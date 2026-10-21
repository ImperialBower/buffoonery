//! Scoring: the four-phase fold that turns a [`BuffoonBoard`] into a [`Score`].
//!
//! Phase 1 sets the base from the hand type and level, phase 2 folds the played
//! cards, phase 3 the held cards, phase 4 the jokers, left to right. Moved here
//! from `board.rs` unchanged (EPIC-05 Story S2), so a step-by-step trace can be
//! threaded through the folds without growing `board.rs` further.

use super::BuffoonBoard;
use crate::prelude::{BCardType, BuffoonCard, BuffoonPile, HandRules, HandType, MPip, Score};
use crate::types::effect::{EffectRegistry, ScoreOp, ScoringContext};
use crate::types::trace::{Phase, ScoreStep, ScoreTrace, StepAt, StepPart, StepSource, apply_step};
use rand::rngs::StdRng;
use rand::{Rng, RngExt, SeedableRng};

/// Balatro's Lucky card grants a flat +20 mult on a successful (1-in-N) roll.
const LUCKY_MULT: usize = 20;

impl BuffoonBoard {
    /// Phase 1 — pre-scoring: establishes the base chips and mult for the
    /// played hand from its [`HandType`] at its current level, as tracked in
    /// [`PokerHands`](crate::prelude::PokerHands). This is the starting [`Score`] before any played-card
    /// (phase 2), held-card (phase 3), or joker (phase 4) contributions.
    ///
    /// A Royal Flush shares the Straight Flush's base and level, matching
    /// Balatro (there is no separate Royal Flush entry to level up).
    ///
    /// From [Detailed Break down of Balatro Scoring System and some tips to optimise your hand scoring.](https://www.reddit.com/r/balatro/comments/1blbexa/detailed_break_down_of_balatro_scoring_system_and/)
    #[must_use]
    pub fn scoring_phase1_pre_scoring(&self) -> Score {
        self.poker_hands
            .get(&self.scoring_hand_type())
            .map_or_else(Score::default, |hand| {
                Score::from_usize(hand.chips, hand.mult)
            })
    }

    /// The hand type the played cards score as, under the board's
    /// [`HandRules`] — so Four Fingers and Shortcut are already accounted for.
    ///
    /// A Royal Flush is normalised to Straight Flush, matching Balatro (there is
    /// no separate Royal Flush entry to level up). Shared by phase 1 and by the
    /// hand-type readers (Card Sharp, `on_hand_played`'s per-round tally) so
    /// there is one answer to "what hand is this" for the whole board — a
    /// second, subtly different copy is exactly how a Royal Flush would come to
    /// count as its own type in one place and not another.
    #[must_use]
    pub fn scoring_hand_type(&self) -> HandType {
        Self::normalise_hand_type(self.played.determine_hand_type_with(self.hand_rules()))
    }

    /// Which played cards score, by position: the cards that make the hand
    /// ([`BuffoonPile::scoring_mask_with`]), plus every Stone card, which
    /// always scores. Under **Splash** every played card scores. The others
    /// are kickers: they add nothing, trigger no per-card joker, and are not
    /// retriggered.
    ///
    /// [`BuffoonPile::scoring_mask_with`]: crate::types::buffoon_pile::BuffoonPile::scoring_mask_with
    #[must_use]
    pub fn scoring_mask(&self) -> Vec<bool> {
        let splash = self
            .jokers
            .iter()
            .any(|joker| !joker.debuffed && matches!(joker.enhancement, MPip::AllPlayedCardsScore));
        if splash {
            return vec![true; self.played.len()];
        }
        let rules = self.hand_rules();
        let hand_type = self.played.determine_hand_type_with(rules);
        self.played
            .scoring_mask_with(hand_type, rules)
            .into_iter()
            .zip(self.played.iter())
            .map(|(scores, card)| scores || card.is_stone())
            .collect()
    }

    /// The played cards that score ([`scoring_mask`](Self::scoring_mask)), in
    /// played order. The per-scored-card jokers read these.
    #[must_use]
    pub fn scored_cards(&self) -> BuffoonPile {
        self.played
            .iter()
            .zip(self.scoring_mask())
            .filter(|(_, scores)| *scores)
            .map(|(card, _)| *card)
            .collect()
    }

    pub(super) fn normalise_hand_type(hand_type: HandType) -> HandType {
        match hand_type {
            HandType::RoyalFlush => HandType::StraightFlush,
            other => other,
        }
    }

    /// Phase 2 — played-hand scoring: folds each played card into the running
    /// `score`. Every card adds its chip value (base rank + flat `Chips`
    /// enhancement, via [`BuffoonCard::get_chips`]) plus any per-card plus-effects
    /// from its own enhancement (conditional chips / mult, via
    /// [`BuffoonCard::calculate_plus`]); those two paths handle disjoint `MPip`
    /// variants, so nothing is counted twice.
    ///
    /// Takes the running score (rather than returning an independent
    /// contribution) so a custom played card's ×mult multiplies the score
    /// accumulated so far — including the phase-1 base — in card order.
    ///
    /// [`BuffoonCard::get_chips`]: crate::types::buffoon_card::BuffoonCard::get_chips
    /// [`BuffoonCard::calculate_plus`]: crate::types::buffoon_card::BuffoonCard::calculate_plus
    #[must_use]
    pub fn scoring_phase2_dealt_hand_scoring(&self, running: Score) -> Score {
        self.fold_played_cards::<StdRng>(running, None, None, None)
    }

    /// The single played-card fold behind phase 2. Each card adds its chips and
    /// built-in additive effects, then its special enhancement resolves:
    /// a Lucky card rolls (if `rng`), a Glass card applies its ×mult, a
    /// `MPip::Custom` card is looked up (if `registry`). Built-in cards are
    /// unaffected by the options. With a `trace`, each of the 3 ops per pass
    /// (effect, special, edition) is recorded as a step.
    // Glass's ×mult factor is a small literal from the card data; the same
    // allow the other ×mult seams carry (`builtin_held_op`, `joker_x_mult`).
    #[allow(clippy::cast_precision_loss)]
    fn fold_played_cards<R: Rng + ?Sized>(
        &self,
        running: Score,
        mut rng: Option<&mut R>,
        registry: Option<&EffectRegistry>,
        mut trace: Option<&mut Vec<ScoreStep>>,
    ) -> Score {
        let mut score = running;
        let scoring = self.scoring_mask();
        // The card's place among the scoring cards: Hanging Chad retriggers
        // the first card that scores, not the first card played.
        let mut position = 0;

        for (index, card) in self.played.iter().enumerate() {
            // A kicker scores nothing and leaves no step.
            if !scoring.get(index).copied().unwrap_or(false) {
                continue;
            }
            let source = StepSource::Card { card: *card, index };
            // Only a Lucky card rolls, and only when there is an RNG to roll.
            let rolls =
                matches!(card.enhancement, MPip::Lucky(odds, _) if odds > 0) && rng.is_some();
            // A card is scored once, plus once more for each retrigger a joker
            // grants it (Hack: each played 2-5; Hanging Chad: the first card).
            // Retriggering re-runs the whole per-card contribution, so a
            // retriggered Lucky card rolls again — matching Balatro. With no
            // retrigger joker this is a single pass.
            for retrigger in 0..=self.played_retriggers(position, card) {
                let at = |part, rolled| StepAt {
                    phase: Phase::Played,
                    source,
                    part,
                    retrigger,
                    rolled,
                };
                // Every played card contributes its built-in chips/mult first,
                // then its special (probabilistic / custom) effect resolves.
                score = apply_step(
                    &mut trace,
                    score,
                    &Self::builtin_played_op(card),
                    at(StepPart::Effect, false),
                );

                let special = match card.enhancement {
                    MPip::Lucky(mult_odds, _) if mult_odds > 0 => {
                        // A 1-in-`mult_odds` roll wins on outcomes `0..wins`;
                        // Oops! All 6s doubles `wins` (capped at certainty).
                        let wins = self.probability_numerator().min(mult_odds);
                        rng.as_deref_mut().map_or(ScoreOp::Nothing, |rng| {
                            if rng.random_range(0..mult_odds) < wins {
                                ScoreOp::AddMult(LUCKY_MULT)
                            } else {
                                ScoreOp::Nothing
                            }
                        })
                    }
                    // Glass card: ×n mult when scored. Multiplicative, so it
                    // cannot ride the additive `calculate_plus` path — it scales
                    // the running score at this card's position, like a held
                    // Steel card does in phase 3. The destruction half
                    // (1-in-`_odds` after the hand) is still data only:
                    // `on_round_end_with_rng` rolls *joker* destruction, and
                    // destroying a played card needs the deck-mutation seam
                    // wired into a round loop that does not exist yet.
                    MPip::Glass(mult, _odds) => ScoreOp::TimesMult(mult as f64),
                    MPip::Custom(id) => self.custom_op(*card, id, registry),
                    _ => ScoreOp::Nothing,
                };
                score = apply_step(&mut trace, score, &special, at(StepPart::Special, rolls));

                // The card's edition scores at its own position, after its
                // chips/mult — so a Polychrome ×1.5 multiplies the running score
                // here (the Glass shape), and a retriggered card re-applies its
                // edition each pass, matching Balatro.
                score = apply_step(
                    &mut trace,
                    score,
                    &card.edition.score_op(),
                    at(StepPart::Edition, false),
                );
            }
            position += 1;
        }

        score
    }

    /// How many *additional* times a scoring played card is scored, summed
    /// over the board's retrigger jokers. 0 for a board with none (the common
    /// case), so the played-card fold is byte-identical when no retrigger joker
    /// is held. `index` is the card's position among the **scoring** cards,
    /// used by position-based retriggers (Hanging Chad fires only on the first
    /// scoring card).
    ///
    /// Jokers are walked with their `joker_state` slot, since the round-state
    /// retriggers read a counter: Seltzer retriggers only while its 10 hands
    /// are unspent.
    fn played_retriggers(&self, index: usize, card: &BuffoonCard) -> usize {
        self.jokers
            .iter()
            .enumerate()
            .filter(|(_, joker)| !joker.debuffed)
            .map(|(slot, joker)| {
                let counter = self.joker_state.get(slot).copied().unwrap_or(0);
                match joker.enhancement {
                    MPip::RetriggerPlayedRanks(n, ranks) if ranks.contains(&card.rank.index) => n,
                    MPip::RetriggerPlayedFaces(n) if self.is_face_card(card) => n,
                    MPip::RetriggerFirstPlayed(n) if index == 0 => n,
                    // Dusk: every played card, but only on the round's last hand.
                    MPip::RetriggerPlayedCardsInFinalRound if self.is_final_hand() => 1,
                    // Seltzer: every played card, for its first `hands` hands.
                    // `counter` is hands *completed*, so the hand that spends the
                    // last one still retriggers and `melt_emptied_jokers` removes
                    // the joker immediately after it.
                    MPip::RetriggerAllPlayedForHands(n, hands)
                        if usize::try_from(counter.max(0)).unwrap_or(usize::MAX) < hands =>
                    {
                        n
                    }
                    _ => 0,
                }
            })
            .sum()
    }

    /// Built-in played-card contribution: base rank chips (+ flat `Chips`) plus
    /// the card's own additive plus-effects, as one additive [`ScoreOp`].
    fn builtin_played_op(card: &BuffoonCard) -> ScoreOp {
        ScoreOp::Add(Score::from_usize(card.get_chips(), 0) + card.calculate_plus(card))
    }

    /// Resolves a `MPip::Custom(id)` card/joker to the [`ScoreOp`] its registered
    /// [`Effect`](crate::types::effect::Effect) returns, or
    /// [`ScoreOp::Nothing`] if there is no registry entry. Shared by all three
    /// phase folds.
    fn custom_op(
        &self,
        source: BuffoonCard,
        id: u32,
        registry: Option<&EffectRegistry>,
    ) -> ScoreOp {
        registry
            .and_then(|r| r.get(id))
            .map_or(ScoreOp::Nothing, |effect| {
                let ctx = ScoringContext {
                    board: self,
                    source,
                };
                effect.score(&ctx)
            })
    }

    /// Phase 3 — held-card effects: applies the ×mult contributions of cards
    /// held **in hand** (not played) to the running `score`. The canonical case
    /// is a Steel card (`MPip::STEEL` = `MultTimes1Dot(15)` = ×1.5 mult while
    /// held); `MultTimes(n)` gives a flat ×n.
    ///
    /// Unlike the additive phases, held effects multiply, so this takes the
    /// score accumulated so far (phases 1 + 2) and returns it transformed. With
    /// no held cards it is the identity.
    #[must_use]
    pub fn scoring_phase3_effects_in_hand(&self, running: Score) -> Score {
        self.fold_held_cards(running, None, None)
    }

    /// The single held-card fold behind phase 3. Built-in Steel/`MultTimes`
    /// cards apply their ×mult; a `MPip::Custom` held card is resolved through
    /// `registry` (if any). With a `trace`, each application is a step.
    fn fold_held_cards(
        &self,
        running: Score,
        registry: Option<&EffectRegistry>,
        mut trace: Option<&mut Vec<ScoreStep>>,
    ) -> Score {
        let mut score = running;

        // Mime retriggers held-card abilities: each held card's op applies
        // `1 + held_retriggers` times, so a retriggered Steel card gives ×1.5
        // twice. 0 for a board with no held-retrigger joker (the common case),
        // leaving the held fold byte-identical.
        let retriggers = self.held_retriggers();
        for (index, card) in self.in_hand.iter().enumerate() {
            let op = match card.enhancement {
                MPip::Custom(id) => self.custom_op(*card, id, registry),
                _ => Self::builtin_held_op(card),
            };
            for retrigger in 0..=retriggers {
                let at = StepAt {
                    phase: Phase::Held,
                    source: StepSource::Card { card: *card, index },
                    part: StepPart::Effect,
                    retrigger,
                    rolled: false,
                };
                score = apply_step(&mut trace, score, &op, at);
            }
        }

        score
    }

    /// How many *additional* times every held card's ability fires, summed over
    /// the board's held-retrigger jokers (Mime). Unlike `played_retriggers` this
    /// is card-independent — Mime retriggers all held cards alike.
    fn held_retriggers(&self) -> usize {
        self.jokers
            .iter()
            .filter(|joker| !joker.debuffed)
            .map(|joker| match joker.enhancement {
                MPip::RetriggerCardsInHand(n) => n,
                _ => 0,
            })
            .sum()
    }

    /// Built-in held-card contribution: Steel / `MultTimes` give a ×mult, as a
    /// [`ScoreOp`]; anything else is inert while held.
    #[allow(clippy::cast_precision_loss)]
    fn builtin_held_op(card: &BuffoonCard) -> ScoreOp {
        match card.enhancement {
            MPip::MultTimes1Dot(n) => ScoreOp::TimesMult(n as f64 / 10.0),
            MPip::MultTimes(n) => ScoreOp::TimesMult(n as f64),
            _ => ScoreOp::Nothing,
        }
    }

    /// Phase 4 — joker scoring: applies each joker to the running `score`, left
    /// to right. Order matters: a `+mult` joker followed by a `×mult` joker
    /// scores differently than the reverse, so jokers must be folded into one
    /// running score rather than summed independently.
    ///
    /// Additive jokers contribute via [`BuffoonPile::calculate_plus`] (chips /
    /// +mult); multiplicative jokers scale the running mult — unconditional
    /// (`MultTimes(n)` = ×n, `MultTimes1Dot(n)` = ×n/10) or hand-conditional
    /// (`MultTimesOn{Pair,Trips,4OfAKind,Straight,Flush}` — The Duo/Trio/Family/
    /// Order/Tribe — which fire when the played hand *contains* that category).
    ///
    /// Note: Loyalty Card (`MultTimesEveryXHands`) is not applied yet — it
    /// needs a hands-played counter on the board.
    ///
    /// [`BuffoonPile::calculate_plus`]: crate::types::buffoon_pile::BuffoonPile::calculate_plus
    #[must_use]
    pub fn scoring_phase4_joker_scoring(&self, running: Score) -> Score {
        self.fold_jokers::<StdRng>(running, None, None, None)
    }

    /// The single joker-scoring fold that every phase-4 entry point delegates
    /// to. Applies each joker to the running score left-to-right:
    ///
    /// * `MPip::Custom(id)` — resolved through `registry` (if any), else inert;
    /// * `MPip::MultPlusRandomTo(n)` — rolled with `rng` (if any), else inert;
    /// * multiplicative built-ins ([`joker_x_mult`](Self::joker_x_mult)) — ×mult;
    /// * everything else — additive ([`BuffoonPile::calculate_plus`]).
    ///
    /// The `rng`/`registry` options are what distinguish the pure, seeded, and
    /// registry entry points — the fold itself lives here once. With a
    /// `trace`, each joker's effect and edition are recorded as two steps.
    ///
    /// [`BuffoonPile::calculate_plus`]: crate::types::buffoon_pile::BuffoonPile::calculate_plus
    fn fold_jokers<R: Rng + ?Sized>(
        &self,
        running: Score,
        mut rng: Option<&mut R>,
        registry: Option<&EffectRegistry>,
        mut trace: Option<&mut Vec<ScoreStep>>,
    ) -> Score {
        let mut score = running;

        for (index, joker) in self.jokers.iter().enumerate() {
            let at = |part, rolled| StepAt {
                phase: Phase::Joker,
                source: StepSource::Joker {
                    card: *joker,
                    index,
                },
                part,
                retrigger: 0,
                rolled,
            };
            // A debuffed joker keeps its rows, as no-ops: it does nothing.
            // Only Misprint rolls, and only when there is an RNG to roll.
            let rolls = !joker.debuffed
                && matches!(joker.enhancement, MPip::MultPlusRandomTo(n) if n > 0)
                && rng.is_some();
            let op = match joker.enhancement {
                _ if joker.debuffed => ScoreOp::Nothing,
                MPip::Custom(id) => self.custom_op(*joker, id, registry),
                MPip::MultPlusRandomTo(n) if n > 0 => {
                    rng.as_deref_mut().map_or(ScoreOp::Nothing, |rng| {
                        ScoreOp::AddMult(rng.random_range(0..n))
                    })
                }
                _ => {
                    let counter = self.joker_state.get(index).copied().unwrap_or(0);
                    self.counter_joker_op(joker, counter)
                        .unwrap_or_else(|| self.builtin_joker_op(joker))
                }
            };
            score = apply_step(&mut trace, score, &op, at(StepPart::Effect, rolls));

            // The joker's own edition scores at its position, after its effect —
            // so a Polychrome joker ×1.5s the running score once its +mult/×mult
            // has landed, matching Balatro's left-to-right joker order.
            let edition = if joker.debuffed {
                ScoreOp::Nothing
            } else {
                joker.edition.score_op()
            };
            score = apply_step(&mut trace, score, &edition, at(StepPart::Edition, false));
        }

        score
    }

    /// Built-in joker contribution as a [`ScoreOp`]: a ×mult for a (satisfied)
    /// multiplicative joker, otherwise its additive chips/mult.
    fn builtin_joker_op(&self, joker: &BuffoonCard) -> ScoreOp {
        // Board-reading additive jokers (a pure function of the current board).
        match joker.enhancement {
            // Flat +chips, whatever the hand. No Balatro joker carries it; the
            // calculator's "+ Chips" override does (EPIC-05 Story 6).
            MPip::Chips(n) => return ScoreOp::AddChips(n),
            MPip::MultPlusPerJoker(n) => return ScoreOp::AddMult(n * self.jokers.len()),
            MPip::ChipsPerDeckCard(n) => return ScoreOp::AddChips(n * self.deck.len()),
            // Stone Joker: +n chips per Stone card in the full deck.
            MPip::ChipsPerFullDeckStone(n) => {
                return ScoreOp::AddChips(n * self.full_deck_stone_count());
            }
            // Erosion: +n mult per card destroyed from the starting deck.
            MPip::MultPlusPerMissingDeckCard(n) => {
                return ScoreOp::AddMult(n * self.cards_missing_from_deck());
            }
            MPip::ChipsPlusPerScoredFace(n) => {
                let faces = self
                    .scored_cards()
                    .iter()
                    .filter(|card| self.is_face_card(card))
                    .count();
                return ScoreOp::AddChips(n * faces);
            }
            MPip::ChipsMultPlusPerScoredRanks(chips, mult, ranks) => {
                let count = self
                    .scored_cards()
                    .iter()
                    .filter(|card| ranks.contains(&card.rank.index))
                    .count();
                return ScoreOp::Add(Score::from_usize(chips * count, mult * count));
            }
            // Banner: +n chips for each remaining discard (reads round state).
            MPip::ChipsPerRemainingDiscard(n) => {
                return ScoreOp::AddChips(n * self.discards_remaining());
            }
            // Mystic Summit: +n mult only when no discards remain, else inert.
            MPip::MultPlusOnZeroDiscards(n) => {
                return if self.discards_remaining() == 0 {
                    ScoreOp::AddMult(n)
                } else {
                    ScoreOp::Nothing
                };
            }
            // Bull: +n chips per $1 held; debt (negative money) scores nothing.
            MPip::ChipsPerDollar(n) => {
                let dollars = usize::try_from(self.money).unwrap_or(0);
                return ScoreOp::AddChips(n * dollars);
            }
            // Fortune Teller: +n mult per Tarot used this run. A board reader,
            // not a counter — that is what makes it retroactive.
            MPip::MultPlusPerTarotUsedThisRun(n) => {
                return ScoreOp::AddMult(n * self.tarots_used);
            }
            // Gros Michel: +n mult unconditionally. The destruction half of the
            // variant is inert here — it rolls at end of round, which has no
            // hook yet — but the mult is not conditional on it and scores now.
            MPip::MultPlusChanceDestroyed(n, _, _) => return ScoreOp::AddMult(n),
            // Scholar: +chips and +mult for each scored card of the given rank;
            // compounds with the count (+20 chips, +4 mult per scored Ace).
            MPip::MultPlusChipsOnRank(mult, chips, rank) => {
                let count = self
                    .scored_cards()
                    .iter()
                    .filter(|card| card.rank.index == rank)
                    .count();
                return ScoreOp::Add(Score::from_usize(chips * count, mult * count));
            }
            // Raised Fist: +n × the lowest-ranked held card's value to mult
            // (nothing when the hand is empty).
            MPip::MultPlusXOnLowestRankInHand(n) => {
                let lowest = self
                    .in_hand
                    .iter()
                    .map(|card| card.rank.value)
                    .min()
                    .unwrap_or(0);
                return ScoreOp::AddMult(n * lowest);
            }
            _ => {}
        }

        self.joker_x_mult(joker).map_or_else(
            || {
                // Per-card effects (the suit jokers, Fibonacci, Even Steven,
                // Odd Todd) pay for each scored card; the hand-shape ones
                // ("contains a Pair") read the whole played hand.
                let per_card = matches!(
                    joker.enhancement,
                    MPip::MultPlusOnSuit(_, _)
                        | MPip::MultPlusOn5Ranks(_, _)
                        | MPip::ChipsPlusOn5Ranks(_, _)
                );
                let pile = if per_card {
                    &self.scored_cards()
                } else {
                    &self.played
                };
                ScoreOp::Add(pile.calculate_plus_with(joker, self.hand_rules()))
            },
            ScoreOp::TimesMult,
        )
    }

    /// The straight/flush detection rules in force for this board, loosened by
    /// its rule-modifier jokers: **Four Fingers** drops straights and flushes to
    /// four cards; **Shortcut** allows one-gap straights. Vanilla Balatro
    /// ([`HandRules::default`]) when neither is held, so hand typing is
    /// unchanged. Multiple modifiers stack (Four Fingers + Shortcut → a
    /// four-card gapped straight).
    pub(super) fn hand_rules(&self) -> HandRules {
        let mut rules = HandRules::default();
        for joker in self.jokers.iter().filter(|joker| !joker.debuffed) {
            match joker.enhancement {
                MPip::FourFlushAndStraight => {
                    rules.straight_connectors = 3;
                    rules.flush_len = 4;
                }
                MPip::GappedStraight => rules.straight_distance = 2,
                MPip::SmearedSuits => rules.smeared = true,
                _ => {}
            }
        }
        rules
    }

    /// Whether a card of suit `card_suit` counts as the `target` suit.
    ///
    /// Exact, except under **Smeared Joker**, which makes Hearts ≡ Diamonds and
    /// Spades ≡ Clubs — the same merge it already applies to flush sizing, so
    /// Smeared widens what Ancient Joker pays for exactly as it widens a flush.
    fn suit_matches(card_suit: char, target: char, rules: HandRules) -> bool {
        if card_suit == target {
            return true;
        }
        rules.smeared
            && matches!(
                (card_suit, target),
                ('H', 'D') | ('D', 'H') | ('S', 'C') | ('C', 'S')
            )
    }

    /// Whether `card` counts as a face card for the face-reading jokers. Kings,
    /// Queens and Jacks always do; **Pareidolia** makes *every* card a face.
    pub(super) fn is_face_card(&self, card: &BuffoonCard) -> bool {
        self.all_cards_are_faces() || matches!(card.rank.index, 'K' | 'Q' | 'J')
    }

    /// Whether Pareidolia is on the board (every card is treated as a face).
    fn all_cards_are_faces(&self) -> bool {
        self.jokers
            .iter()
            .any(|joker| !joker.debuffed && matches!(joker.enhancement, MPip::AllCardsAreFaces))
    }

    /// How many cards in the run's full deck carry a Steel enhancement.
    ///
    /// Steel is modelled as `MultTimes1Dot`, matching what
    /// [`builtin_held_op`](Self::builtin_held_op) treats as Steel while held.
    fn full_deck_steel_count(&self) -> usize {
        self.full_deck
            .iter()
            .filter(|card| matches!(card.enhancement, MPip::MultTimes1Dot(_)))
            .count()
    }

    /// How many cards in the run's full deck are Stone cards.
    fn full_deck_stone_count(&self) -> usize {
        self.full_deck
            .iter()
            .filter(|card| matches!(card.enhancement, MPip::Stone(_)))
            .count()
    }

    /// How many cards the full deck is **below** its starting size — the count
    /// of cards destroyed over the run. Saturates at 0, so a deck grown past
    /// its starting size (DNA, Séance) scores nothing rather than wrapping.
    fn cards_missing_from_deck(&self) -> usize {
        self.starting_deck_size.saturating_sub(self.full_deck.len())
    }

    /// Winning outcomes for a 1-in-N probability roll.
    ///
    /// 1 normally, doubled per **Oops! All 6s** on the board (each doubles
    /// listed probabilities). The caller caps it at the roll's denominator so
    /// it never exceeds certainty.
    pub(super) fn probability_numerator(&self) -> usize {
        let oops = self
            .jokers
            .iter()
            .filter(|joker| !joker.debuffed && matches!(joker.enhancement, MPip::DoubleOdds))
            .count();
        // 2^oops, saturating so a pathological joker count can't overflow-shift.
        1usize
            .checked_shl(u32::try_from(oops).unwrap_or(u32::MAX))
            .unwrap_or(usize::MAX)
    }

    /// The ×mult factor a joker applies to the running score given the played
    /// hand, or `None` if it is not a (satisfied) multiplicative joker — in
    /// which case it is handled additively. Hand-conditional jokers use the
    /// "contains" predicates (e.g. `has_pair` is true for two pair / trips /
    /// full house / quads), matching Balatro.
    #[allow(clippy::cast_precision_loss)]
    fn joker_x_mult(&self, joker: &BuffoonCard) -> Option<f64> {
        let played = &self.played;
        // The straight/flush conditionals (The Order, The Tribe) honour the
        // board's rule modifiers, so Four Fingers / Shortcut let them fire on a
        // four-card or gapped hand just as they widen the base hand type.
        let rules = self.hand_rules();
        let factor = match joker.enhancement {
            // Cavendish (`MultTimesChanceDestroyed`) rides the plain ×n arm:
            // its destruction half rolls at end of round
            // (`on_round_end_with_rng`), never gating the mult — the Gros
            // Michel compound shape, on the ×mult side.
            MPip::MultTimes(n) | MPip::MultTimesChanceDestroyed(n, _, _) => n as f64,
            MPip::MultTimes1Dot(n) => n as f64 / 10.0,
            MPip::MultTimesOnPair(n) if played.has_pair() => n as f64,
            MPip::MultTimesOnTrips(n) if played.has_trips() => n as f64,
            MPip::MultTimesOn4OfAKind(n) if played.has_4_of_a_kind() => n as f64,
            MPip::MultTimesOnStraight(n) if played.has_straight_with(rules) => n as f64,
            MPip::MultTimesOnFlush(n) if played.has_flush_with(rules) => n as f64,
            MPip::MultTimesPerScoredRank(n, ranks) => {
                // ×n for each scored card of a matching rank; the factor
                // compounds, e.g. two Kings and a Queen with Triboulet = ×2³.
                let matches = self
                    .scored_cards()
                    .iter()
                    .filter(|card| ranks.contains(&card.rank.index))
                    .count();
                (0..matches).fold(1.0, |acc, _| acc * n as f64)
            }
            MPip::MultTimesPerHeldRank(tenths, rank) => {
                // ×(tenths/10) for each held card of `rank`; compounds. Baron.
                let held = self
                    .in_hand
                    .iter()
                    .filter(|card| card.rank.index == rank)
                    .count();
                let per = tenths as f64 / 10.0;
                (0..held).fold(1.0, |acc, _| acc * per)
            }
            MPip::MultTimesPlusPerFullDeckSteel(tenths) => {
                // Steel Joker: ×1 base, gaining ×(tenths/10) per Steel card in
                // the full deck. Additive in the factor, unlike the compounding
                // per-card jokers above: two Steel with ×0.2 is ×1.4, not ×1.44.
                let steel = self.full_deck_steel_count();
                1.0 + (tenths * steel) as f64 / 10.0
            }
            MPip::MultTimesPerUncommonJoker(tenths) => {
                // ×(tenths/10) per Uncommon joker on the board; compounds. Baseball Card.
                let uncommon = self
                    .jokers
                    .iter()
                    .filter(|j| j.card_type == BCardType::UncommonJoker)
                    .count();
                let per = tenths as f64 / 10.0;
                (0..uncommon).fold(1.0, |acc, _| acc * per)
            }
            MPip::MultTimesIfHeldAllSuits(n, suits)
                if self
                    .in_hand
                    .iter()
                    .all(|card| suits.contains(&card.suit.index)) =>
            {
                // Blackboard: vacuously true (×n) when the hand is empty.
                n as f64
            }
            // Card Sharp: ×n if this hand type has already been played this
            // round. `hands_by_type_this_round` is bumped by `on_hand_played`,
            // which fires *after* a hand scores — so during the round's second
            // Pair the tally reads 1, and `>= 1` is the test. (Balatro bumps
            // before its joker pass and so tests `> 1`; same semantics, and
            // getting it backwards makes Card Sharp fire on the first play.)
            MPip::MultTimesOnRepeatedHandThisRound(n)
                if self
                    .hands_by_type_this_round
                    .get(&self.scoring_hand_type())
                    .copied()
                    .unwrap_or(0)
                    >= 1 =>
            {
                n as f64
            }
            // Ancient Joker: ×(tenths/10) per scored card of the run's current
            // ancient suit; compounds, like its per-card ×mult neighbours. No
            // suit rolled yet means no matches, i.e. ×1 — inert rather than
            // zeroing.
            MPip::MultTimesPerScoredAncientSuit(tenths) => {
                let matches = self.ancient_suit.map_or(0, |suit| {
                    self.scored_cards()
                        .iter()
                        .filter(|card| Self::suit_matches(card.suit.index, suit, rules))
                        .count()
                });
                let per = tenths as f64 / 10.0;
                (0..matches).fold(1.0, |acc, _| acc * per)
            }
            // Joker Stencil: ×n per empty joker slot, "Joker Stencil included" —
            // i.e. it counts its own occupied slot as if it were empty. Every
            // Stencil on the board adds that +1, so the rule reduces to
            // `slots − (jokers that are not Stencils)`. A Negative joker takes
            // no slot, so it fills none ([`BuffoonBoard::slots_taken`]).
            //
            // The `> 0` gate is on **literally** empty slots, not the inclusive
            // count: a full board applies nothing at all. With one Stencil that
            // is unobservable (the inclusive count is exactly ×1 — identity —
            // when the gate closes), but with two on a full board the two
            // disagree, and the gate wins.
            MPip::MultTimesOnEmptyJokerSlots(n) => {
                let empty = self
                    .joker_slots
                    .saturating_sub(Self::slots_taken(&self.jokers));
                if empty == 0 {
                    return None;
                }
                let stencils = self
                    .jokers
                    .iter()
                    .filter(|j| matches!(j.enhancement, MPip::MultTimesOnEmptyJokerSlots(_)))
                    .count();
                (n * (empty + stencils)) as f64
            }
            _ => return None,
        };
        Some(factor)
    }

    /// Combined score for the currently played hand — the full four-phase
    /// pipeline, in Balatro order:
    ///
    /// 1. base hand chips/mult,
    /// 2. played-card chips,
    /// 3. held-card ×mult (Steel, …),
    /// 4. joker contributions.
    ///
    /// The final chips × mult is `score().score()`. Each phase folds into one
    /// running score, so the Balatro-significant ordering (held ×mult before
    /// jokers, and jokers left-to-right) is preserved. This never panics, so a
    /// solver can call it for any board.
    ///
    /// NOTE: this is deterministic — probabilistic effects (Lucky, Misprint)
    /// contribute their floor of zero here; use
    /// [`score_with_seed`](Self::score_with_seed) to roll them. State-dependent
    /// effects (economy, discards/hands remaining) still fall through to zero.
    #[must_use]
    pub fn score(&self) -> Score {
        let base = self.scoring_phase1_pre_scoring();
        let after_cards = self.scoring_phase2_dealt_hand_scoring(base);
        let held = self.scoring_phase3_effects_in_hand(after_cards);

        self.scoring_phase4_joker_scoring(held)
    }

    /// Like [`score`](Self::score), but realizes the crate's **probabilistic**
    /// effects with a `u64` seed — deterministic per seed, so a solver can
    /// reproduce a roll or sample the outcome distribution over many seeds.
    ///
    /// Currently rolled: Lucky cards (1-in-N → +20 mult, phase 2) and the
    /// Misprint joker (`MultPlusRandomTo(n)` → +random(0..n) mult, phase 4).
    /// Everything else scores identically to [`score`](Self::score); in
    /// particular the pure `score()` is the guaranteed floor (no procs).
    #[must_use]
    pub fn score_with_seed(&self, seed: u64) -> Score {
        self.score_with_rng(&mut StdRng::seed_from_u64(seed))
    }

    /// Like [`score_with_seed`](Self::score_with_seed), but drives the
    /// probabilistic effects from the caller's RNG.
    #[must_use]
    pub fn score_with_rng<R: Rng + ?Sized>(&self, rng: &mut R) -> Score {
        let base = self.scoring_phase1_pre_scoring();
        let after_cards = self.scoring_phase2_dealt_hand_scoring_with_rng(base, rng);
        let held = self.scoring_phase3_effects_in_hand(after_cards);

        self.scoring_phase4_joker_scoring_with_rng(held, rng)
    }

    /// Phase 2 with probabilistic played-card effects (Lucky cards) rolled from
    /// `rng`, threaded through the same played-card fold as
    /// [`scoring_phase2_dealt_hand_scoring`](Self::scoring_phase2_dealt_hand_scoring).
    #[must_use]
    pub fn scoring_phase2_dealt_hand_scoring_with_rng<R: Rng + ?Sized>(
        &self,
        running: Score,
        rng: &mut R,
    ) -> Score {
        self.fold_played_cards(running, Some(rng), None, None)
    }

    /// Phase 4 with probabilistic joker effects rolled, threaded through the
    /// same left-to-right fold as
    /// [`scoring_phase4_joker_scoring`](Self::scoring_phase4_joker_scoring) so
    /// a random `+mult` (Misprint) still lands in joker order and interacts
    /// correctly with later ×mult jokers.
    #[must_use]
    pub fn scoring_phase4_joker_scoring_with_rng<R: Rng + ?Sized>(
        &self,
        running: Score,
        rng: &mut R,
    ) -> Score {
        self.fold_jokers(running, Some(rng), None, None)
    }

    /// Like [`score`](Self::score), but resolves `MPip::Custom(id)` jokers
    /// through a mod-supplied [`EffectRegistry`] — the extension point that lets
    /// a mod add scoring behaviour without editing funky source.
    ///
    /// Built-in effects score exactly as in [`score`](Self::score); a custom
    /// card or joker is scored by looking up its id in `registry` and applying
    /// the [`ScoreOp`] its [`Effect`] returns. Unregistered ids contribute
    /// nothing.
    ///
    /// Custom effects are resolved in every phase they can occur — **played
    /// cards** (phase 2), **held cards** (phase 3) and **jokers** (phase 4) —
    /// via the same [`ScoringContext`]/[`ScoreOp`] pattern.
    ///
    /// [`Effect`]: crate::types::effect::Effect
    /// [`ScoreOp`]: crate::types::effect::ScoreOp
    #[must_use]
    pub fn score_with_registry(&self, registry: &EffectRegistry) -> Score {
        let base = self.scoring_phase1_pre_scoring();
        let after_cards = self.fold_played_cards::<StdRng>(base, None, Some(registry), None);
        let held = self.fold_held_cards(after_cards, Some(registry), None);

        self.scoring_phase4_joker_scoring_with_registry(held, registry)
    }

    /// Phase 4, resolving `MPip::Custom(id)` jokers through `registry`. Built-in
    /// jokers fold in exactly as [`scoring_phase4_joker_scoring`] does.
    ///
    /// [`scoring_phase4_joker_scoring`]: Self::scoring_phase4_joker_scoring
    #[must_use]
    pub fn scoring_phase4_joker_scoring_with_registry(
        &self,
        running: Score,
        registry: &EffectRegistry,
    ) -> Score {
        self.fold_jokers::<StdRng>(running, None, Some(registry), None)
    }

    /// Like [`score`](Self::score), but returns every step: one per
    /// [`ScoreOp`] applied, starting with the base hand. Its `total` equals
    /// `score()`, and replaying the steps' ops from an empty [`Score`] gives
    /// each step's `after`.
    #[must_use]
    pub fn score_traced(&self) -> ScoreTrace {
        self.trace_with::<StdRng>(None, None)
    }

    /// Like [`score_with_seed`](Self::score_with_seed), but returns every
    /// step. Rolled steps (Lucky, Misprint) carry `rolled: true`.
    #[must_use]
    pub fn score_traced_with_seed(&self, seed: u64) -> ScoreTrace {
        self.score_traced_with_rng(&mut StdRng::seed_from_u64(seed))
    }

    /// Like [`score_with_rng`](Self::score_with_rng), but returns every step.
    #[must_use]
    pub fn score_traced_with_rng<R: Rng + ?Sized>(&self, rng: &mut R) -> ScoreTrace {
        self.trace_with(Some(rng), None)
    }

    /// Like [`score_with_registry`](Self::score_with_registry), but returns
    /// every step.
    #[must_use]
    pub fn score_traced_with_registry(&self, registry: &EffectRegistry) -> ScoreTrace {
        self.trace_with::<StdRng>(None, Some(registry))
    }

    /// Like [`score_traced_with_seed`](Self::score_traced_with_seed) and
    /// [`score_traced_with_registry`](Self::score_traced_with_registry) at
    /// once: chances roll from `seed`, and `MPip::Custom(id)` effects resolve
    /// through `registry`.
    #[must_use]
    pub fn score_traced_with_seed_and_registry(
        &self,
        seed: u64,
        registry: &EffectRegistry,
    ) -> ScoreTrace {
        self.score_traced_with_rng_and_registry(&mut StdRng::seed_from_u64(seed), registry)
    }

    /// Like [`score_traced_with_rng`](Self::score_traced_with_rng) and
    /// [`score_traced_with_registry`](Self::score_traced_with_registry) at
    /// once: chances roll from `rng`, and `MPip::Custom(id)` effects resolve
    /// through `registry`. The RNG is drawn in the same order as in
    /// [`score_with_rng`](Self::score_with_rng), so the rolls match a seeded
    /// score of the same board.
    #[must_use]
    pub fn score_traced_with_rng_and_registry<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        registry: &EffectRegistry,
    ) -> ScoreTrace {
        self.trace_with(Some(rng), Some(registry))
    }

    /// The traced pipeline: the base step, then the same three folds the
    /// untraced entry points use, each given the step list. The RNG is drawn in
    /// the same order as [`score_with_rng`](Self::score_with_rng) (played
    /// cards, then jokers), so a seed gives the same total either way.
    fn trace_with<R: Rng + ?Sized>(
        &self,
        mut rng: Option<&mut R>,
        registry: Option<&EffectRegistry>,
    ) -> ScoreTrace {
        let hand_type = self.scoring_hand_type();
        let level = self
            .poker_hands
            .get(&hand_type)
            .map_or(0, |hand| hand.level);
        let mut steps = Vec::new();

        let base = apply_step(
            &mut Some(&mut steps),
            Score::default(),
            &ScoreOp::Add(self.scoring_phase1_pre_scoring()),
            StepAt {
                phase: Phase::Base,
                source: StepSource::Hand { hand_type, level },
                part: StepPart::Effect,
                retrigger: 0,
                rolled: false,
            },
        );
        let played = self.fold_played_cards(base, rng.as_deref_mut(), registry, Some(&mut steps));
        let held = self.fold_held_cards(played, registry, Some(&mut steps));
        let total = self.fold_jokers(held, rng, registry, Some(&mut steps));

        ScoreTrace {
            hand_type,
            steps,
            total,
        }
    }

    /// The scoring contribution of a counter joker given its accumulator, or
    /// `None` if `joker` is not a counter joker (so scoring falls through to
    /// `builtin_joker_op`). Arms are added per joker in later tasks, at which
    /// point they will read board state through `&self`.
    #[allow(clippy::unused_self)]
    fn counter_joker_op(&self, joker: &BuffoonCard, counter: i32) -> Option<ScoreOp> {
        match joker.enhancement {
            MPip::GainMultPerHandLessDiscard(rate) => {
                let net = counter.max(0);
                #[allow(clippy::cast_sign_loss)]
                Some(ScoreOp::AddMult(rate * net as usize))
            }
            MPip::GainChipsPerCardCountHand(rate, _n) =>
            {
                #[allow(clippy::cast_sign_loss)]
                Some(ScoreOp::AddChips(rate * counter.max(0) as usize))
            }
            // Spare Trousers gains +rate mult per two-pair hand; Flash Card per
            // reroll; Red Card per pack skipped. Different events grow the
            // counter (`growth_delta` keeps them apart), but the read is one
            // additive rule, so it is written once.
            MPip::GainMultPerTwoPairHand(rate)
            | MPip::MultPlusPerReroll(rate)
            | MPip::MultPlusPerPackSkipped(rate) =>
            {
                #[allow(clippy::cast_sign_loss)]
                Some(ScoreOp::AddMult(rate * counter.max(0) as usize))
            }
            MPip::GainChipsPerStraightHand(rate) =>
            {
                #[allow(clippy::cast_sign_loss)]
                Some(ScoreOp::AddChips(rate * counter.max(0) as usize))
            }
            MPip::LoseMultTimesPerDiscard(base, per) => {
                #[allow(clippy::cast_precision_loss)]
                let raw = (per as f64).mul_add(-f64::from(counter.max(0)), base as f64) / 100.0;
                Some(ScoreOp::TimesMult(raw.max(1.0)))
            }
            MPip::LoseChipsPerHand(base, per) => {
                #[allow(clippy::cast_sign_loss)]
                let hands = counter.max(0) as usize;
                Some(ScoreOp::AddChips(base.saturating_sub(per * hands)))
            }
            // Popcorn: Ice Cream's decay on the mult side, per round rather
            // than per hand. Floors at 0 — the round that empties it also
            // destroys it (`melt_emptied_jokers`).
            MPip::LoseMultPerRound(base, per) => {
                #[allow(clippy::cast_sign_loss)]
                let rounds = counter.max(0) as usize;
                Some(ScoreOp::AddMult(base.saturating_sub(per * rounds)))
            }
            // Yorick: ×1 per 23 cards discarded. The accumulator counts cards,
            // so the factor steps only on each completed block of `per`.
            MPip::GainMultTimesPerDiscardedCards(rate, per) if per > 0 => {
                #[allow(clippy::cast_sign_loss)]
                let blocks = counter.max(0) as usize / per;
                Some(ScoreOp::TimesMult(Self::gain_x_mult(rate, blocks)))
            }
            // Hologram (×0.25 per card added to the deck), Canio (×1 per face
            // destroyed) and Vampire (×0.1 per enhanced card played) differ only
            // in which event grows them — `growth_delta` keeps them apart. The
            // *read* is one rule, so it is written once.
            MPip::GainMultTimesPerCardAdded(rate)
            | MPip::GainMultTimesPerFaceDestroyed(rate)
            | MPip::GainMultTimesPerEnhancedPlayed(rate)
            | MPip::GainMultTimesPerPlanetUsed(rate)
            | MPip::GainMultTimesOnNonBossBlindDestroyingJoker(rate) => {
                #[allow(clippy::cast_sign_loss)]
                let ticks = counter.max(0) as usize;
                Some(ScoreOp::TimesMult(Self::gain_x_mult(rate, ticks)))
            }
            _ => None,
        }
    }

    /// The ×mult factor of a "this joker gains ×`rate`/100 mult per event"
    /// counter that has ticked `count` times: `1 + (rate/100) × count`.
    ///
    /// **Additive, not compounding** — the Steel Joker rule
    /// ([`MPip::MultTimesPlusPerFullDeckSteel`]), which is what Balatro's
    /// "gains ×N Mult" jokers do: Hologram at four cards added is ×2, not
    /// ×0.25⁴. Base ×1 falls out of `count == 0`, so an ungrown counter joker
    /// is inert rather than zeroing the mult.
    #[allow(clippy::cast_precision_loss)]
    fn gain_x_mult(rate: usize, count: usize) -> f64 {
        (rate as f64 / 100.0).mul_add(count as f64, 1.0)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod funky__types__board__scoring_tests {
    use super::*;
    use crate::bcards;
    use crate::decks::basic::Deck;
    use crate::decks::basic::card as basic;
    use crate::decks::joker::card;
    use crate::prelude::{BuffoonPile, Edition};
    use crate::types::draws::Draws;
    use crate::types::effect::Effect;
    use crate::types::trace::{Phase, ScoreTrace, StepPart, StepSource};

    fn board_playing(index: &str) -> BuffoonBoard {
        let mut board = BuffoonBoard::new(Draws::new(4, 3), Deck::basic_buffoon_pile());
        board.played = bcards!(index);
        board
    }

    fn enhanced(card: BuffoonCard, enhancement: MPip) -> BuffoonCard {
        BuffoonCard {
            enhancement,
            ..card
        }
    }

    fn with_jokers(index: &str, jokers: &[BuffoonCard]) -> BuffoonBoard {
        let mut board = board_playing(index);
        for joker in jokers {
            board.push_joker(*joker);
        }
        board
    }

    /// The `buffoon` example's board: a Royal Flush, a held Steel King, and
    /// four hand-conditional jokers.
    fn buffoon_example_board() -> BuffoonBoard {
        let mut board = with_jokers(
            "AS KS QS JS TS",
            &[
                card::CRAZY_JOKER,
                card::DROLL_JOKER,
                card::DEVIOUS_JOKER,
                card::CRAFTY_JOKER,
            ],
        );
        board.in_hand = BuffoonPile::from(vec![enhanced(basic::KING_HEARTS, MPip::STEEL)]);
        board
    }

    fn acceptance_flush_board() -> BuffoonBoard {
        with_jokers(
            "2H 5H 8H TH KH",
            &[
                card::JOKER,
                card::DROLL_JOKER,
                card::CRAFTY_JOKER,
                card::CAVENDISH,
            ],
        )
    }

    fn lucky_ace_board() -> BuffoonBoard {
        let mut board = board_playing("2S");
        board.played = BuffoonPile::from(vec![enhanced(basic::ACE_SPADES, MPip::Lucky(5, 15))]);
        board
    }

    /// Boards that cover each kind of step: base, played (with a retrigger,
    /// a special and an edition), held (with a retrigger), and jokers (with a
    /// failed condition, a ×mult, a roll and an edition).
    fn golden_boards() -> Vec<BuffoonBoard> {
        let mut steel_mime = with_jokers("2S 5D 8C TS KH", &[card::MIME]);
        steel_mime.in_hand = BuffoonPile::from(vec![enhanced(basic::KING_HEARTS, MPip::STEEL)]);

        let mut glass_poly = board_playing("AS AD QC JS TH");
        glass_poly.played = BuffoonPile::from(vec![
            enhanced(basic::ACE_SPADES, MPip::Glass(2, 4)),
            basic::ACE_DIAMONDS.with_edition(Edition::Polychrome),
        ]);

        vec![
            buffoon_example_board(),
            acceptance_flush_board(),
            with_jokers("AS AD QC JS TH", &[card::DROLL_JOKER]),
            with_jokers("2S 3D 8C TS KH", &[card::HACK]),
            steel_mime,
            glass_poly,
            with_jokers(
                "2S 5D 8C TS KH",
                &[
                    card::JOKER.with_edition(Edition::Polychrome),
                    card::MISPRINT,
                ],
            ),
            lucky_ace_board(),
        ]
    }

    /// Every step's `after` is its `op` applied to the step before it, starting
    /// from an empty score: the trace replays to the same numbers.
    fn assert_replays(trace: &ScoreTrace) {
        let mut score = Score::default();
        for (i, step) in trace.steps.iter().enumerate() {
            score = step.op.apply(score);
            assert_eq!(score, step.after, "step {i} does not replay: {step:?}");
        }
        assert_eq!(trace.steps.last().map(|s| s.after), Some(trace.total));
    }

    fn effect_rows(trace: &ScoreTrace, phase: Phase) -> Vec<String> {
        trace
            .steps
            .iter()
            .filter(|s| s.phase == phase && s.part == StepPart::Effect)
            .map(|s| s.op.to_string())
            .collect()
    }

    // ---- EPIC-05 Story 1 acceptance -----------------------------------------

    #[test]
    fn score_traced__flush_joker_rows_in_order() {
        let trace = acceptance_flush_board().score_traced();
        assert_eq!(
            effect_rows(&trace, Phase::Joker),
            vec!["+4 Mult", "+10 Mult", "+80 Chips", "×3 Mult"]
        );
    }

    #[test]
    fn score_traced__a_failed_condition_is_a_row_that_did_not_fire() {
        // Droll Joker pays on a flush; a Pair is not one.
        let trace = with_jokers("AS AD QC JS TH", &[card::DROLL_JOKER]).score_traced();
        let droll = trace
            .steps
            .iter()
            .find(|s| s.phase == Phase::Joker && s.part == StepPart::Effect)
            .unwrap();
        assert!(!droll.fired, "{droll:?}");
        assert!(droll.op.is_noop());
    }

    #[test]
    fn scoring_phase4__a_chips_joker_adds_flat_chips() {
        // The calculator's "+ Chips" override (EPIC-05 Story 6) writes
        // `MPip::Chips(n)` onto a joker; it adds n chips whatever the hand.
        let board = with_jokers("AS", &[enhanced(card::JOKER, MPip::Chips(50))]);
        let trace = board.score_traced();
        let step = trace
            .steps
            .iter()
            .find(|s| s.phase == Phase::Joker && s.part == StepPart::Effect)
            .unwrap();

        assert_eq!(step.op, ScoreOp::AddChips(50));
        assert_eq!(trace.total, Score::new(5 + 11 + 50, 1.0));
    }

    fn debuffed(joker: BuffoonCard) -> BuffoonCard {
        BuffoonCard {
            debuffed: true,
            ..joker
        }
    }

    #[test]
    fn scoring_phase4__a_debuffed_joker_scores_nothing() {
        let polychrome = card::JOKER.with_edition(Edition::Polychrome);
        let board = with_jokers("AS AH", &[debuffed(polychrome)]);
        let trace = board.score_traced();
        let jokers: Vec<_> = trace
            .steps
            .iter()
            .filter(|s| s.phase == Phase::Joker)
            .collect();

        assert_eq!(jokers.len(), 2, "the rows stay, so the table can show them");
        assert!(jokers.iter().all(|s| s.op == ScoreOp::Nothing && !s.fired));
        assert_eq!(trace.total, board_playing("AS AH").score_traced().total);
    }

    #[test]
    fn played_retriggers__a_debuffed_joker_retriggers_nothing() {
        let board = with_jokers("2S 2H", &[debuffed(card::HACK)]);
        assert!(board.score_traced().steps.iter().all(|s| s.retrigger == 0));
    }

    #[test]
    fn hand_rules__a_debuffed_joker_changes_no_rule() {
        let working = with_jokers("2H 5H 8H TH", &[card::FOUR_FINGERS]);
        assert_eq!(working.score_traced().hand_type, HandType::Flush);

        let board = with_jokers("2H 5H 8H TH", &[debuffed(card::FOUR_FINGERS)]);
        assert_eq!(board.score_traced().hand_type, HandType::HighCard);
    }

    // ---- Shape ---------------------------------------------------------------

    #[test]
    fn score_traced__starts_with_the_base_step() {
        let board = board_playing("AS AD QC JS TH");
        let trace = board.score_traced();
        let base = &trace.steps[0];

        assert_eq!(trace.hand_type, HandType::Pair);
        assert_eq!(base.phase, Phase::Base);
        assert_eq!(
            base.source,
            StepSource::Hand {
                hand_type: HandType::Pair,
                level: 1
            }
        );
        assert_eq!(base.after, board.scoring_phase1_pre_scoring());
        assert!(base.fired);
    }

    /// A played card makes 3 steps per pass: effect, special, edition.
    #[test]
    fn score_traced__three_steps_per_played_card_pass() {
        let trace = board_playing("2S").score_traced();
        let parts: Vec<_> = trace
            .steps
            .iter()
            .filter(|s| s.phase == Phase::Played)
            .map(|s| (s.part, s.retrigger))
            .collect();
        assert_eq!(
            parts,
            vec![
                (StepPart::Effect, 0),
                (StepPart::Special, 0),
                (StepPart::Edition, 0)
            ]
        );
        assert_eq!(trace.steps.len(), 4, "base + 3");
    }

    /// Hack retriggers a played 2: the card's three steps repeat with
    /// `retrigger: 1`.
    #[test]
    fn score_traced__a_retrigger_repeats_the_card_steps() {
        let trace = with_jokers("2S", &[card::HACK]).score_traced();
        let played: Vec<_> = trace
            .steps
            .iter()
            .filter(|s| s.phase == Phase::Played)
            .map(|s| (s.part, s.retrigger))
            .collect();
        assert_eq!(
            played,
            vec![
                (StepPart::Effect, 0),
                (StepPart::Special, 0),
                (StepPart::Edition, 0),
                (StepPart::Effect, 1),
                (StepPart::Special, 1),
                (StepPart::Edition, 1),
            ]
        );
    }

    /// Mime retriggers a held Steel card: one held step per application.
    #[test]
    fn score_traced__held_steps_one_per_application() {
        let mut board = with_jokers("2S", &[card::MIME]);
        board.in_hand = BuffoonPile::from(vec![enhanced(basic::KING_HEARTS, MPip::STEEL)]);
        let held: Vec<_> = board
            .score_traced()
            .steps
            .into_iter()
            .filter(|s| s.phase == Phase::Held)
            .collect();

        assert_eq!(held.len(), 2);
        assert_eq!((held[0].retrigger, held[1].retrigger), (0, 1));
        assert!(held.iter().all(|s| s.op == ScoreOp::TimesMult(1.5)));
        assert!(matches!(held[0].source, StepSource::Card { index: 0, .. }));
    }

    /// A joker makes 2 steps: its effect, then its edition.
    #[test]
    fn score_traced__two_steps_per_joker() {
        let trace = with_jokers("2S", &[card::JOKER.with_edition(Edition::Foil)]).score_traced();
        let joker: Vec<_> = trace
            .steps
            .iter()
            .filter(|s| s.phase == Phase::Joker)
            .collect();

        assert_eq!(joker.len(), 2);
        assert_eq!(joker[0].part, StepPart::Effect);
        assert_eq!(joker[0].op.to_string(), "+4 Mult");
        assert_eq!(joker[1].part, StepPart::Edition);
        assert_eq!(joker[1].op.to_string(), "+50 Chips");
        assert!(matches!(
            joker[0].source,
            StepSource::Joker { index: 0, card } if card == card::JOKER.with_edition(Edition::Foil)
        ));
    }

    // ---- Rolls ---------------------------------------------------------------

    #[test]
    fn score_traced__unseeded_lucky_is_the_unrolled_floor() {
        let trace = lucky_ace_board().score_traced();
        let special = trace
            .steps
            .iter()
            .find(|s| s.phase == Phase::Played && s.part == StepPart::Special)
            .unwrap();
        assert!(!special.rolled);
        assert_eq!(special.op, ScoreOp::Nothing);
    }

    #[test]
    fn score_traced_with_seed__lucky_and_misprint_are_rolled() {
        let mut board = lucky_ace_board();
        board.push_joker(card::MISPRINT);
        for seed in 0..16 {
            let trace = board.score_traced_with_seed(seed);
            let rolled: Vec<_> = trace
                .steps
                .iter()
                .filter(|s| s.rolled)
                .map(|s| (s.phase, s.part))
                .collect();
            assert_eq!(
                rolled,
                vec![
                    (Phase::Played, StepPart::Special),
                    (Phase::Joker, StepPart::Effect)
                ],
                "seed {seed}"
            );
        }
    }

    // ---- Invariants ----------------------------------------------------------

    #[test]
    fn score_traced__total_matches_score_on_every_golden_board() {
        for (i, board) in golden_boards().iter().enumerate() {
            let trace = board.score_traced();
            assert_eq!(trace.total, board.score(), "board {i}");
            assert_eq!(trace.hand_type, board.scoring_hand_type(), "board {i}");
            assert_replays(&trace);
        }
    }

    #[test]
    fn score_traced_with_seed__total_matches_score_with_seed() {
        for (i, board) in golden_boards().iter().enumerate() {
            for seed in 0..8 {
                let trace = board.score_traced_with_seed(seed);
                assert_eq!(
                    trace.total,
                    board.score_with_seed(seed),
                    "board {i} seed {seed}"
                );
                assert_replays(&trace);
            }
        }
    }

    struct DoubleMult;
    impl Effect for DoubleMult {
        fn score(&self, _ctx: &ScoringContext<'_>) -> ScoreOp {
            ScoreOp::TimesMult(2.0)
        }
    }

    #[test]
    fn score_traced_with_registry__total_matches_score_with_registry() {
        let mut registry = EffectRegistry::new();
        registry.register(7, DoubleMult);

        let mut board = with_jokers("AS AD QC JS TH", &[enhanced(card::JOKER, MPip::Custom(7))]);
        board.in_hand = BuffoonPile::from(vec![enhanced(basic::KING_HEARTS, MPip::Custom(7))]);

        let trace = board.score_traced_with_registry(&registry);
        assert_eq!(trace.total, board.score_with_registry(&registry));
        assert_replays(&trace);
        let custom_steps = trace
            .steps
            .iter()
            .filter(|s| s.op == ScoreOp::TimesMult(2.0))
            .count();
        assert_eq!(custom_steps, 2, "the held card and the joker");
    }

    /// The played indexes that score, from the trace's first-pass card steps.
    fn scored_indexes(trace: &ScoreTrace) -> Vec<usize> {
        trace
            .steps
            .iter()
            .filter(|s| s.phase == Phase::Played && s.part == StepPart::Effect && s.retrigger == 0)
            .filter_map(|s| match s.source {
                StepSource::Card { index, .. } => Some(index),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn scoring__a_kicker_scores_nothing() {
        let board = board_playing("KS KD 4C");
        let trace = board.score_traced();
        assert_eq!(scored_indexes(&trace), vec![0, 1]);
        assert_eq!(board.scoring_mask(), vec![true, true, false]);
        // Pair 10 × 2, plus 10 per King; the 4 is a kicker.
        assert_eq!(board.score(), Score::new(30, 2.0));
        assert_eq!(trace.total, board.score());
    }

    #[test]
    fn scoring__splash_scores_every_played_card() {
        let board = with_jokers("KS KD 4C", &[card::SPLASH]);
        assert_eq!(scored_indexes(&board.score_traced()), vec![0, 1, 2]);
        assert_eq!(board.score(), Score::new(34, 2.0));

        let off = with_jokers("KS KD 4C", &[debuffed(card::SPLASH)]);
        assert_eq!(scored_indexes(&off.score_traced()), vec![0, 1]);
    }

    #[test]
    fn scoring__a_stone_card_always_scores() {
        let mut board = board_playing("KS KD");
        board.played.push(basic::STONE_CARD);
        assert_eq!(scored_indexes(&board.score_traced()), vec![0, 1, 2]);
        assert_eq!(board.score(), Score::new(80, 2.0));
    }

    #[test]
    fn scoring__an_ace_low_straight_is_a_straight() {
        let board = board_playing("AS 2D 3C 4H 5S");
        let trace = board.score_traced();
        assert_eq!(trace.hand_type, HandType::Straight);
        assert_eq!(scored_indexes(&trace), vec![0, 1, 2, 3, 4]);
        // Straight 30 × 4, plus 11 + 2 + 3 + 4 + 5.
        assert_eq!(board.score(), Score::new(55, 4.0));
    }

    #[test]
    fn played_retriggers__hanging_chad_retriggers_the_first_scoring_card() {
        let trace = with_jokers("4C KS KD", &[card::HANGING_CHAD]).score_traced();
        let retriggered: Vec<usize> = trace
            .steps
            .iter()
            .filter(|s| s.part == StepPart::Effect && s.retrigger > 0)
            .filter_map(|s| match s.source {
                StepSource::Card { index, .. } => Some(index),
                _ => None,
            })
            .collect();
        // Hanging Chad retriggers twice; both land on the first scoring card.
        assert_eq!(retriggered, vec![1, 1]);
    }

    /// The effect op of the joker in slot `index`.
    fn joker_effect(board: &BuffoonBoard, index: usize) -> ScoreOp {
        board
            .score_traced()
            .steps
            .into_iter()
            .find(|s| {
                s.part == StepPart::Effect
                    && matches!(s.source, StepSource::Joker { index: i, .. } if i == index)
            })
            .map(|s| s.op)
            .unwrap()
    }

    /// The "contains a Flush / Straight" +chips and +mult jokers see the hand
    /// the way the base hand type does: Four Fingers, Shortcut and Smeared
    /// widen it. Debuff the rule joker and they do not fire.
    #[test]
    fn scoring_phase4__hand_shape_jokers_follow_the_hand_rules() {
        let cases = [
            (
                "2H 5H 8H TH KS",
                card::FOUR_FINGERS,
                card::CRAFTY_JOKER,
                Score::new(80, 0.0),
            ),
            (
                "2H 5D 8H TD KD",
                card::SMEARED_JOKER,
                card::DROLL_JOKER,
                Score::new(0, 10.0),
            ),
            (
                "3S 5D 7C 9H JS",
                card::SHORTCUT,
                card::CRAZY_JOKER,
                Score::new(0, 12.0),
            ),
            (
                "2S 3D 4C 5H 9S",
                card::FOUR_FINGERS,
                card::DEVIOUS_JOKER,
                Score::new(100, 0.0),
            ),
        ];
        for (hand, rule, joker, delta) in cases {
            let board = with_jokers(hand, &[rule, joker]);
            assert_eq!(
                joker_effect(&board, 1),
                ScoreOp::Add(delta),
                "{hand} with {rule}"
            );
            let off = with_jokers(hand, &[debuffed(rule), joker]);
            assert!(
                joker_effect(&off, 1).is_noop(),
                "{hand} with {rule} debuffed"
            );
        }
    }

    #[test]
    fn scoring_phase4__per_scored_card_jokers_skip_kickers() {
        // Greedy Joker: +3 mult per scored Diamond. The 4D is a kicker.
        let trace = with_jokers("KS KD 4D", &[card::GREEDY_JOKER]).score_traced();
        let greedy = trace
            .steps
            .iter()
            .find(|s| s.phase == Phase::Joker)
            .unwrap();
        assert_eq!(greedy.op, ScoreOp::Add(Score::new(0, 3.0)));
    }

    #[test]
    fn score_traced_with_rng_and_registry__rolls_and_resolves_custom_effects() {
        let mut registry = EffectRegistry::new();
        registry.register(7, DoubleMult);
        let mut board = lucky_ace_board();
        board.push_joker(card::MISPRINT);
        board.push_joker(enhanced(card::JOKER, MPip::Custom(7)));

        for seed in 0..8 {
            let rng = &mut StdRng::seed_from_u64(seed);
            let trace = board.score_traced_with_rng_and_registry(rng, &registry);
            let seeded = board.score_traced_with_seed(seed);
            assert_replays(&trace);
            let rolled = |t: &ScoreTrace| -> Vec<ScoreOp> {
                t.steps
                    .iter()
                    .filter(|s| s.rolled)
                    .map(|s| s.op.clone())
                    .collect()
            };
            assert_eq!(
                rolled(&trace),
                rolled(&seeded),
                "seed {seed} rolls the same"
            );
            let last = trace
                .steps
                .iter()
                .rfind(|s| s.part == StepPart::Effect)
                .unwrap();
            assert_eq!(last.op, ScoreOp::TimesMult(2.0), "seed {seed}");
            assert_eq!(trace.total, seeded.total.multi_mult(2.0), "seed {seed}");
        }

        assert_eq!(
            board.score_traced_with_seed_and_registry(5, &registry),
            board.score_traced_with_rng_and_registry(&mut StdRng::seed_from_u64(5), &registry),
        );

        let empty = EffectRegistry::new();
        let trace = board.score_traced_with_rng_and_registry(&mut StdRng::seed_from_u64(3), &empty);
        assert_eq!(
            trace,
            board.score_traced_with_seed(3),
            "no registry entry: as seeded"
        );
    }
}
