# EPIC-10: Every suit at once — Wild cards, Stone cards, Smeared (SUITS)

Oct 24, 2026 · @Christoph · *Written against `main` at `fdac48e`. Numbered in the shared cardpack / buffoonery space, after EPIC-09. Defect-driven: two `BACKLOG.md` bugs, plus three that the survey for them found.*

## Summary

- **Builds:** one rule for "is this card of that suit": a Wild card is every suit, a Stone card is none, and Smeared Joker merges red and black. Every suit check in scoring asks it. Also: `blind_target` becomes as wide as the score it is compared with.
- **Why:** today a Wild card is only its printed suit (`MPip::is_wild` is never called), so `AH KH 9H 4H 2S+WILD` is not a Flush. A Stone card's hidden suit pays Wrathful Joker and passes Blackboard. Smeared Joker widens a flush but not Greedy Joker. The calculator gives wrong numbers for all of these.
- **Shape:** `BuffoonCard::is_suit(suit, rules)` in the engine, named after Balatro's `Card:is_suit`. Flush sizing and the flush's scoring cards count Wild cards; the three suit-reading joker effects call `is_suit`. `blind_target: usize` becomes `f64`.
- **Proves it:** golden fixtures 21 (a Wild flush) and 22 (Stone and Wild against suit jokers) score to hand-checked totals; the 20 existing traces do not change.
- **Status:** In progress on `epic10-suits`. Phases 0 (`is_suit`), 1 (flushes) and 2 (suit jokers) are built.

---

## Context

The engine reads a card's suit in seven places. None of them knows about Wild cards. Surveyed 2026-10-24 at `fdac48e`.

- **A Wild card is data nobody reads.** The Lovers puts `MPip::Wild(PipType::Suit)` on a card (`src/decks/tarot.rs:97`). `MPip::is_wild` (`src/types/mpip.rs:429`) exists and is never called. The board line spells it `7D+WILD` (EPIC-09).
- **Flush sizing.** `count_largest_same_suit` (`src/types/buffoon_pile.rs:356`) asks cardpack's `combos_by_suit` over `detectable()` cards. cardpack's card keeps only suit and rank, so the enhancement is gone before the count. `count_largest_same_suit_with` (`:367`) counts red and black under Smeared. Every flush test (`has_flush_with`, `:517`, and so Flush, Straight Flush, Royal Flush, Flush House, Flush Five, Crafty, Droll, The Tribe, Séance) reaches these two.
- **The flush's scoring cards.** `flush_mask` (`src/types/buffoon_pile.rs:289`) groups cards by suit (or colour under Smeared), finds the first group with `flush_len` cards, and marks that group as scoring. A Wild card of another printed suit would not be marked even if the flush counted it.
- **Stone cards are suitless by mask, not by data.** `is_stone` (`src/types/buffoon_card.rs:195`) and `detectable` (`src/types/buffoon_pile.rs:345`) keep Stone cards out of hand detection. A Stone card still has a suit inside (Spades, `src/decks/basic.rs:198`), and two places read it: the suit jokers and Blackboard.
- **The suit jokers.** Greedy, Lusty, Wrathful and Gluttonous Joker (`src/decks/joker.rs:387`, `:402`, `:417`, `:432`) carry `MPip::MultPlusOnSuit(3, suit)` (`src/types/mpip.rs:102`). `calculate_plus_mult` (`src/types/buffoon_card.rs:160`) pays when `self.suit.index == suit` (`:163`): no Wild, no Smeared, and a scored Stone card pays Wrathful. `calculate_plus_mult_with` (`src/types/buffoon_pile.rs:121`) has the `HandRules` but does not pass them down (`:132`).
- **Blackboard.** `MultTimesIfHeldAllSuits(3, ['S', 'C'])` (`src/decks/joker.rs:1100`, `src/types/mpip.rs:157`) is checked in `joker_x_mult` (`src/types/board/scoring.rs:702`) as `suits.contains(&card.suit.index)` over held cards. A held Wild Heart fails it; a held Stone card passes it.
- **Ancient Joker.** `MultTimesPerScoredAncientSuit` (`src/types/board/scoring.rs:731`) calls `suit_matches(card_suit: char, target, rules)` (`:573`), which knows Smeared but takes a `char`, so it cannot see Wild or Stone.
- **Smeared Joker** (`src/decks/joker.rs:1945`) sets `rules.smeared` (`src/types/board/scoring.rs:561`). `HandRules::smeared` is documented as "when sizing a flush" (`src/types/buffoon_pile.rs:32`).
- **No helper exists.** There is no `is_suit` or `has_suit`; each site does its own comparison.
- **Playing cards are never debuffed.** `BuffoonCard.debuffed` is read for jokers only; the suit-debuff bosses are not modelled (`src/types/blind.rs:14`); `CardSpec` has no `debuffed` field. So the debuffed-Wild rule does not arise yet.
- **No golden fixture holds a Wild card, a Stone card next to a suit joker, or Smeared next to a suit joker.** Fixtures 06 and 15 have Greedy Joker with plain cards only. So the 20 existing traces must not change; that is a test.
- **`blind_target` is `usize`** (`src/types/board.rs:88`). On wasm32 that is 32 bits, so a target past 4.29 × 10⁹ cannot be held. `round_score` beside it is `f64` (`:80`), and `round_is_won` casts (`:967`). `BoardSpec` has no target, so only Rust callers meet this; EPIC-05 deferred it (`docs/EPIC-05_Joker_Calculator.md:198`).

**Confirmed with the CLI on 2026-10-24** (`buffoonery score --board`):

| Board line | Today | Should be |
| --- | --- | --- |
| `AH KH 9H 4H 2S+WILD` | High Card, 16 × 1 = 16 | a Flush |
| `AS AD KS+STONE / / WRATHFULJOKER` | Pair, 82 × 8 = 656 (the Stone card pays +3) | 82 × 5 = 410 |
| `AS AD / QH+WILD JS / BLACKBOARD` | Pair, 32 × 2 = 64 (Blackboard does not fire) | 32 × 6 = 192 |

**What Balatro does.** From the author's and the survey's memory of Balatro's `Card:is_suit` (not checked against its source in this EPIC): a Stone card is no suit; a Wild card is every suit; under Smeared, Hearts ≡ Diamonds and Spades ≡ Clubs for every suit check, not only flushes; otherwise the printed suit. A debuffed Wild card loses its wildness for flushes and has no suit for other checks. The first three are the rules this EPIC encodes.

**This EPIC does not:**

- add playing-card debuffs or the suit-debuff bosses (The Club, The Goad, The Head, The Window). `is_suit` documents the debuffed-Wild rule for when they come.
- implement the suit jokers still missing (Castle, Flower Pot, Seeing Double, The Idol, Rough Gem, Bloodstone, Arrowhead, Onyx Agate; `src/decks/joker.rs:2170`). When they come, they call `is_suit`.
- change how a Wild card ranks, pairs or makes a straight. A Wild card is a suit wildcard only.
- touch `MPip::Wild(PipType::Rank)` or any other `Wild` variant. Only `Wild(PipType::Suit)` is wild.
- add ante progression or a blind table. `blind_target` stays caller-set.
- change any existing golden trace, `parse` output, page link or board line.

---

## Status

As of 2026-10-24, `main` at `fdac48e`. Nothing is built.

| Component | Status |
| --- | --- |
| Phase 0 — `BuffoonCard::is_suit` | **Complete** |
| Phase 1 — flushes count Wild cards (sizing and scoring cards) | **Complete** |
| Phase 2 — the suit jokers, Blackboard and Ancient Joker call `is_suit` | **Complete** |
| Phase 3 — `blind_target` is `f64` | Planned |
| Phase 4 — golden fixtures 21 and 22 | Planned |
| Phase 5 — docs, OKF, CHANGELOG | Planned |

---

## Goals

- **One suit rule.** Every suit check in scoring goes through one function. A rule fixed there is fixed everywhere, including for jokers not yet written.
- **Wild is every suit.** A Wild card makes a flush with any four (or, under Four Fingers, three) cards of one suit, scores as part of it, and pays every suit joker.
- **Stone is no suit.** A Stone card pays no suit joker and fails Blackboard.
- **Smeared everywhere.** Smeared Joker merges suits for the suit jokers and Blackboard, as it already does for flushes and Ancient Joker.
- **No silent change.** The 20 existing traces stay byte-for-byte; the two new fixtures say, in their comments, what changed and why.
- **A wide target.** `blind_target` holds any score the engine can reach.

## Scope

1. **`is_suit(card, suit, rules)`**, in this order: a Stone card is no suit (false); a Wild card (`MPip::Wild(PipType::Suit)`) is every suit (true); the printed suit matches (true); under `rules.smeared`, Hearts ≡ Diamonds and Spades ≡ Clubs (true); otherwise false.
2. **Flush sizing.** The largest group is, over the four suits, the count of non-Stone cards for which `is_suit` holds. A Wild card counts in every suit's group, so it is counted once per group, not once per hand. Under Smeared the groups are red and black, and a Wild card is in both.
3. **The flush's scoring cards.** When a hand is a Flush (or a stronger flush hand), the scoring cards are every non-Stone card for which `is_suit(flush suit)` holds: the natural cards of that suit and every Wild card.
4. **Which suit is the flush suit.** The first suit, in the order the played cards lie, whose group reaches `flush_len`. If only Wild cards are left to decide (every card is Wild), all of them score and the suit does not matter. This keeps today's tie rule for natural cards.
5. **The suit jokers** pay per scored card for which `is_suit(their suit)` holds. **Blackboard** fires when every held card `is_suit` Spades or Clubs (still true for an empty hand). **Ancient Joker** counts scored cards for which `is_suit(ancient suit)` holds. All three use the hand's `HandRules`, so Smeared reaches them.
6. **`blind_target: f64`**, `0.0` for an untargeted round. `round_is_won` is `blind_target > 0.0 && round_score >= blind_target`, with no cast.
7. **Nothing else moves.** Hand types without a flush, rank checks, straights, pairs and Stone scoring are unchanged.

---

## Domain map

| Domain concept | Code construct | Status |
| --- | --- | --- |
| A card's suit | `BuffoonCard.suit` | ✅ |
| "Is this card of that suit?" | none; seven ad-hoc comparisons | ❌ |
| A Wild card | `MPip::Wild(PipType::Suit)` (`src/types/mpip.rs:398`) | 🟡 data only |
| A Stone card has no suit | `detectable` (`src/types/buffoon_pile.rs:345`) | 🟡 hand detection only |
| Smeared merges suits | `HandRules::smeared` (`src/types/buffoon_pile.rs:34`) | 🟡 flushes and Ancient only |
| A flush and its scoring cards | `count_largest_same_suit_with`, `flush_mask` | 🟡 no Wild |
| A suit joker | `MPip::MultPlusOnSuit` (`src/types/buffoon_card.rs:163`) | 🟡 printed suit only |
| A blind target | `blind_target: usize` (`src/types/board.rs:88`) | 🟡 32-bit on wasm32 |

---

## Design

### `BuffoonCard::is_suit`

`src/types/buffoon_card.rs`, beside `is_stone` (`:195`):

```rust
impl BuffoonCard {
    /// Whether this card counts as `suit` (`'S'`, `'H'`, `'D'`, `'C'`) under
    /// `rules`, as Balatro's `Card:is_suit` decides: a Stone card is no suit;
    /// a Wild card is every suit; under Smeared Joker, Hearts ≡ Diamonds and
    /// Spades ≡ Clubs; otherwise the printed suit.
    ///
    /// Playing cards are never debuffed yet. When the suit-debuff bosses
    /// arrive: a debuffed Wild card is its printed suit in a flush check and
    /// no suit elsewhere.
    #[must_use]
    pub fn is_suit(&self, suit: char, rules: HandRules) -> bool;

    /// Whether this card is a Wild card: `MPip::Wild(PipType::Suit)`.
    #[must_use]
    pub const fn is_wild(&self) -> bool;
}
```

**Why on the card and why `HandRules`.** Every caller has a card and the hand's rules; Smeared is a rule, not a card fact. `HandRules` already lives next to the pile (`src/types/buffoon_pile.rs:24`). **Why `is_suit`.** It is Balatro's name, and the missing jokers will be written from Balatro's text. **Why a card `is_wild`, not `MPip::is_wild`.** `MPip::is_wild` matches every `Wild(_)`; only `Wild(PipType::Suit)` is a suit wildcard. The `MPip` method stays as it is.

`BuffoonBoard::suit_matches` (`src/types/board/scoring.rs:573`) is deleted; Ancient Joker calls `is_suit`.

### Flushes

`src/types/buffoon_pile.rs`. cardpack cannot count Wild cards (its card has no enhancement), so the count is written here:

```rust
/// The suits a flush can be of: four, or the two colours under Smeared.
/// Under Smeared, 'S' stands for black and 'H' for red, and is_suit merges.
fn flush_suits(rules: HandRules) -> &'static [char];

/// Cards of the largest suit group, Wild cards in every group.
pub fn count_largest_same_suit_with(&self, rules: HandRules) -> usize {
    flush_suits(rules)
        .iter()
        .map(|&suit| self.iter().filter(|card| card.is_suit(suit, rules)).count())
        .max()
        .unwrap_or(0)
}
```

`count_largest_same_suit` becomes `count_largest_same_suit_with(HandRules::default())`. `flush_mask` (`:289`) picks the flush suit by Scope rule 4 and marks every card with `is_suit(flush_suit, rules)`. `is_suit` already returns false for Stone, so the `!card.is_stone()` guard (`:301`) folds into it.

**Why not patch `combos_by_suit`.** It is cardpack's, and its card type drops the enhancement. A per-suit count over at most a handful of cards is four short loops.

### The three suit-reading jokers

- `MultPlusOnSuit` (`src/types/buffoon_card.rs:163`): the arm becomes `if self.is_suit(suit, rules)`. `calculate_plus_mult` gains a `_with(enhancer, rules)` twin; the plain one calls it with `HandRules::default()`. `calculate_plus_mult_with` on the pile (`src/types/buffoon_pile.rs:132`) passes its `rules` down.
- Blackboard (`src/types/board/scoring.rs:702`): `suits.iter().any(|&suit| card.is_suit(suit, rules))` for every held card.
- Ancient Joker (`src/types/board/scoring.rs:735`): `card.is_suit(suit, rules)`.

### `blind_target: f64`

`src/types/board.rs:88`. `f64`, not `u64`: it is compared with `round_score: f64` (`:80`), so no cast and no `cast_precision_loss` allow; Balatro's own late-ante targets pass `u64` anyway. `BuffoonBoard` dropped `Eq` in EPIC-05 S1, so a float field costs nothing. **Why not `Option<f64>`.** `0` already means "untargeted" in every doc comment and test; changing the sentinel is a second change.

---

## Work Items

### Phase 0 — `is_suit`

- [x] **0a.** `BuffoonCard::is_wild` and `BuffoonCard::is_suit` (`src/types/buffoon_card.rs:195`), with doc tests. Tests `is_suit__printed_suit`, `is_suit__wild_is_every_suit`, `is_suit__stone_is_no_suit` (a Stone card printed Spades is not a Spade), `is_suit__smeared_merges_colours`, `is_suit__smeared_wild_is_still_every_suit`, `is_wild__only_suit_wild` (`Wild(PipType::Rank)` is not).
- [x] **0b.** `cargo test --workspace` and clippy green. No caller yet, so no trace moves.

### Phase 1 — Flushes

- [x] **1.** `count_largest_same_suit_with` and `count_largest_same_suit` count by `is_suit` (`src/types/buffoon_pile.rs:356`, `:367`). Tests `flush__four_hearts_and_a_wild_is_a_flush` (`AH KH 9H 4H 2S+WILD`), `flush__four_fingers_three_and_a_wild`, `flush__smeared_wild_joins_either_colour`, `flush__five_wilds_is_a_flush`, `flush__wild_pair_of_kings_is_flush_five` (four `KH` and a `KS+WILD` is Flush Five), `flush__stone_still_counts_for_nothing`, `flush__no_flush_without_enough` (`AH KH 9S 4C 2S+WILD` is not).
- [x] **2.** `flush_mask` by `is_suit` (`:289`). Tests `flush_mask__wild_card_scores_in_the_flush` (the `2S+WILD` is a scoring card), `flush_mask__first_suit_in_card_order_wins`, `flush_mask__off_suit_card_does_not_score` (Four Fingers: `AH KH 9H 2S+WILD 5C`, the `5C` does not score).
- [x] **3.** The 20 golden traces unchanged: `cargo test --test golden` green with no `BLESS`.

### Phase 2 — Suit jokers

- [x] **4.** `calculate_plus_mult_with` on the card; the pile passes `rules` (`src/types/buffoon_card.rs:160`, `src/types/buffoon_pile.rs:132`). Tests `suit_joker__wild_pays_every_suit_joker` (a scored `2S+WILD` pays Greedy, Lusty, Wrathful and Gluttonous), `suit_joker__stone_pays_none` (a scored Stone card no longer pays Wrathful), `suit_joker__smeared_pays_the_other_colour` (Greedy pays a Heart under Smeared).
- [x] **5.** Blackboard by `is_suit` (`src/types/board/scoring.rs:702`). Tests `blackboard__held_wild_counts`, `blackboard__held_stone_fails`, `blackboard__smeared_held_heart_fails` (Smeared merges Hearts with Diamonds, not with Spades), `blackboard__empty_hand_still_fires`.
- [x] **6.** Ancient Joker by `is_suit`; delete `suit_matches` (`:573`). Tests `ancient__wild_counts`, `ancient__stone_does_not`. The existing Ancient Joker tests pass unchanged.
- [x] **7.** The 20 golden traces unchanged.

### Phase 3 — `blind_target`

- [ ] **8.** `blind_target: f64` (`src/types/board.rs:88`, `:256`, `:967`); drop the `cast_precision_loss` allow on `round_is_won`. Update the test sites (`:4626`–`:6814`; `usize::MAX` becomes `f64::MAX`). Test `round_is_won__a_target_past_u32` (target 5 × 10⁹, a round score of 6 × 10⁹ wins).

### Phase 4 — Golden fixtures

- [ ] **9.** `fixtures/golden/21_wild_flush.yaml`: `played: [AH, KH, 9H, 4H, {card: 2S, enhancement: !Wild Suit}]`, jokers Droll (flush +10 mult), Lusty (Hearts +3 per card) and Greedy (Diamonds +3 per card). A Flush; the Wild card scores and pays both Lusty and Greedy. Check the total by hand in the fixture comment.
- [ ] **10.** `fixtures/golden/22_stone_and_wild_suits.yaml`: played `AS AD KS+STONE`, held `QH+WILD JS`, jokers Wrathful and Blackboard. The Stone card pays no Wrathful; the held Wild Heart lets Blackboard fire. Total checked by hand.
- [ ] **11.** `BLESS=1` the two traces, their CLI tables, and `fixtures/cli/lines.txt` (two new rows). The page parity walk covers both with no change. `make e2e` green.

### Phase 5 — Docs and the gate

- [ ] **12.** `.okf/decisions/suit-rule.md` (`type: decision`): the rule, why one helper, why `f64`. `.okf/architecture/engine.md`: the suit rule. `.okf/log.md`, `.okf/index.md`.
- [ ] **13.** `CHANGELOG.md` Unreleased: Fixed (Wild flushes, Wild and Stone against suit jokers, Blackboard and Ancient; Smeared reaches the suit jokers and Blackboard), Changed (breaking: `blind_target` is `f64`; `count_largest_same_suit` counts Wild cards). `BACKLOG.md`: tick the two bugs. `make ayce` green.

---

## Test Plan

- `flush__four_hearts_and_a_wild_is_a_flush` — the headline bug.
- `flush_mask__wild_card_scores_in_the_flush` — the Wild card is not only counted but scored.
- `flush__wild_pair_of_kings_is_flush_five` — the strongest flush hand reaches Wild cards too.
- `suit_joker__wild_pays_every_suit_joker`, `suit_joker__stone_pays_none`, `suit_joker__smeared_pays_the_other_colour` — the three suit-joker defects.
- `blackboard__held_wild_counts`, `blackboard__held_stone_fails` — Blackboard's two defects.
- `is_suit__*` — the rule itself, one assertion per clause.
- `golden__traces_match` with no `BLESS` after Phases 1 and 2 — no existing board moved.
- Fixtures 21 and 22 — the fixes, end to end, in the engine, the component, the CLI and the page.
- `round_is_won__a_target_past_u32` — the wide target.

## Key Files

| File | Role |
| --- | --- |
| `src/types/buffoon_card.rs` | `is_wild`, `is_suit`, `calculate_plus_mult_with` |
| `src/types/buffoon_pile.rs` | Flush sizing and `flush_mask` by `is_suit`; rules passed to the suit jokers |
| `src/types/board/scoring.rs` | Blackboard and Ancient Joker by `is_suit`; `suit_matches` deleted |
| `src/types/board.rs` | `blind_target: f64` |
| `fixtures/golden/21_*`, `22_*`, `fixtures/cli/` | New fixtures, tables, lines |
| `CHANGELOG.md`, `BACKLOG.md`, `.okf/` | Docs |

## Reuse (do NOT recreate)

- `HandRules` (`src/types/buffoon_pile.rs:24`) — Smeared and `flush_len` already travel in it.
- `is_stone` (`src/types/buffoon_card.rs:195`) — the first clause of `is_suit`.
- `has_flush_with` (`src/types/buffoon_pile.rs:517`) and the hand cascade — unchanged; they get Wild cards through the count.
- `scoring_mask_with` (`src/types/buffoon_pile.rs:254`) — unchanged; it gets Wild cards through `flush_mask`.
- The golden walk (`tests/golden.rs`), the component, CLI and page parity tests — fixtures 21 and 22 are covered by walking.

## Compatibility

- **Preserves** the 20 golden traces, every `parse` output, page link and board line; the WIT interface; `MPip::is_wild`.
- **Adds** `BuffoonCard::is_suit`, `BuffoonCard::is_wild`, `BuffoonCard::calculate_plus_mult_with`, fixtures 21 and 22.
- **Changes** scores for boards with a Wild card, with a Stone card next to Wrathful Joker or Blackboard, or with Smeared next to a suit joker or Blackboard. These were wrong. `count_largest_same_suit` counts Wild cards.
- **Breaks** `blind_target`'s type (`usize` → `f64`). Unreleased crate (crates.io has 0.0.1), so it goes into 0.1.0's notes.

## Dependencies

- **Blocks:** the missing suit jokers (Castle, Flower Pot, Seeing Double, The Idol, Rough Gem, Bloodstone, Arrowhead, Onyx Agate); the suit-debuff bosses; release 0.1.0 (a breaking type change should land first).
- **Built on:** EPIC-05 (`HandRules`, Four Fingers, Smeared, the golden set), EPIC-09 (`+WILD` in a board line).
- **Related:** `BACKLOG.md` "Wild cards do not count toward a flush" and "`blind_target` is `usize`".

## Open questions

1. **The flush suit when two suits reach `flush_len` only with Wild cards.** Under Four Fingers, `AH 2S+WILD 3S+WILD 4D+WILD 9S` reaches four in Hearts and in Spades. Today's rule (first in card order) picks Hearts, so `9S` does not score. Balatro may pick by a fixed suit order. Recommend: keep card order and pin it with `flush_mask__first_suit_in_card_order_wins`; revisit if a Balatro trace disagrees.
2. **Should the page say "Wild" on a card?** It shows the `MPip` text, `Wild(Suit)`. Recommend: not in this EPIC.

## Verification

```bash
cargo test --workspace
cargo test --doc
cargo clippy --workspace --all-targets -- -D warnings
cargo build --no-default-features
cargo test --test golden
cargo run -p buffoonery-cli -- score --board "AH KH 9H 4H 2S+WILD / / DROLLJOKER LUSTYJOKER GREEDYJOKER"
cargo run -p buffoonery-cli -- score --board "AS AD KS+STONE / QH+WILD JS / WRATHFULJOKER BLACKBOARD"
make ayce
```

Exit criteria:

1. `AH KH 9H 4H 2S+WILD` is a Flush, and the Wild card scores.
2. A Wild card pays every suit joker; a Stone card pays none and fails Blackboard; Smeared reaches the suit jokers and Blackboard.
3. The 20 existing golden traces are byte-for-byte unchanged; fixtures 21 and 22 match their hand-checked totals everywhere.
4. `blind_target` holds 5 × 10⁹ on any target.
5. `make ayce` is green.

## Implementation notes

Written as each phase lands. Design vs actual:

1. **A private `colour` helper** (Phase 0). `is_suit` merges suits under Smeared through `colour(suit)` in `src/types/buffoon_card.rs`: `'R'` for Hearts and Diamonds, `'B'` for Spades and Clubs, `None` for any other suit character. So a tarot or joker card, whose "suit" is not one of the four, never matches under Smeared. `is_wild` is `const`; `is_suit` is not, because `Option::is_some_and` is not `const`. 6 unit tests, 2 doc tests; 688 lib tests.
2. **The flush suit: natural cards decide first** (Phase 1). `flush_mask` tries the printed suits of the natural (not Stone, not Wild) cards in card order, then the suits of a private `flush_suits(rules)` (`S H D C`, or `S H` for black and red under Smeared). The first suit whose group reaches `flush_len` is the flush. So a Wild card's printed suit never picks the flush: `QS 5S+W 7S+W 9D+W AH` under Four Fingers is Spades because `QS` comes first, and `AH 5S+W 7S+W 9D+W QS` is Hearts. This answers Open question 1 with card order, as recommended. The Open question's own example (`AH 2S 3S 4D 9S`) is a Straight Flush under Four Fingers, so the test uses ranks with no straight.
3. **`count_largest_same_suit` no longer calls cardpack** (Phase 1). It is `count_largest_same_suit_with(HandRules::default())`, and both count with a private `count_suit`. `is_suit` drops Stone cards, so neither needs `detectable()`. It had no caller outside `has_flush_with`. 10 unit tests (one, `flush__stone_still_counts_for_nothing`, passed before the change: it guards the Stone rule), 2 doc tests; 698 lib tests. The 20 golden traces did not change. `score --board "AH KH 9H 4H 2S+WILD"` is now `Flush 71 × 4 = 284`.
4. **The Phase 2 tests are board tests** (Phase 2). They live in `src/types/board.rs`, next to the Blackboard and Ancient Joker tests, so they run the whole path: `builtin_joker_op` → `BuffoonPile::calculate_plus_with` → `BuffoonCard::calculate_plus_mult_with`. A test helper `wild_at` puts The Lovers on one card. Two tests passed before the change and are guards: `blackboard__smeared_held_heart_fails` and `blackboard__empty_hand_still_fires`. The seven that failed gave the EPIC's wrong numbers (82 × 8, 7 × 4, 15 × 3 for a held Stone card, …). 9 tests, 1 doc test; 707 lib tests. The 20 golden traces did not change. `score --board` gives `AS AD KS+STONE / / WRATHFULJOKER` → `Pair 82 × 5 = 410` and `AS AD / QH+WILD JS / BLACKBOARD` → `Pair 32 × 6 = 192`.
5. **`HandRules` docs** (Phase 2). The struct and its `smeared` field said Smeared works "for flushes"; they now say every suit check.
