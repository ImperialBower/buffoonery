---
type: decision
title: One suit rule, BuffoonCard::is_suit
description: "Every suit check in scoring asks `BuffoonCard::is_suit(suit, rules)`: a Stone card is no suit, a Wild card is every suit, else the printed suit, and Smeared merges red and black. Flushes, the suit jokers, Blackboard and Ancient Joker use it; `blind_target` became `f64` in the same EPIC."
tags: [suit, wild, stone, smeared, scoring, decision, epic-10]
timestamp: '2026-10-24T00:00:00Z'
resource: src/types/buffoon_card.rs
---

# Decision

`BuffoonCard::is_suit(suit, rules)` (EPIC-10) decides whether a card counts
as a suit (`'S'`, `'H'`, `'D'`, `'C'`). The checks run in this order:

1. A Stone card is no suit (`is_stone`).
2. A Wild card is every suit (`is_wild`: `MPip::Wild(PipType::Suit)` only).
3. The printed suit matches.
4. Under Smeared Joker (`HandRules::smeared`), Hearts and Diamonds are one
   suit, and Spades and Clubs are one suit.

Every suit read in scoring calls it:

* Flush sizing (`BuffoonPile::count_largest_same_suit_with`) and the flush's
  scoring cards (`flush_mask`). A Wild card is in every suit's group. The
  flush suit is the first natural card's suit, in card order, that reaches
  `flush_len`; a Wild card's printed suit never picks it.
* The suit jokers (`MPip::MultPlusOnSuit`, through
  `BuffoonCard::calculate_plus_mult_with`).
* Blackboard (`MultTimesIfHeldAllSuits`) and Ancient Joker
  (`MultTimesPerScoredAncientSuit`). `BuffoonBoard::suit_matches` is gone.

`BuffoonBoard::blind_target` is `f64`, the type of `round_score`.

# Why

* **One helper.** Before EPIC-10 there were seven suit reads, each comparing
  the printed suit by hand. None knew Wild; three knew Smeared; two let a
  Stone card's hidden suit through. One function means a rule fixed once is
  fixed for every reader, and the suit jokers not yet built (Castle, Flower
  Pot, Seeing Double, …) get it for free.
* **Balatro's name.** Balatro's own code calls it `Card:is_suit`; the
  missing jokers will be written from Balatro's text.
* **On the card, with `HandRules`.** Every caller has a card and the hand's
  rules. Smeared is a rule, not a card fact.
* **Not `MPip::is_wild`.** It matches every `Wild(_)`, a rank wildcard too.
* **Not cardpack's `combos_by_suit`.** cardpack's card has no enhancement,
  so the Wild mark is gone before it counts.
* **`blind_target: f64`, not `u64` or `Option<f64>`.** It is compared with
  `round_score: f64`, so no cast; `0.0` already meant "untargeted"
  everywhere. On wasm32 a `usize` stopped at 4.29 × 10⁹.

# Consequences

* Golden fixtures `21_wild_flush` (2,272) and `22_stone_and_wild_suits`
  (1,230) pin the fixes; the 20 older traces did not change.
* Not modelled yet: playing-card debuffs. When the suit-debuff bosses come,
  a debuffed Wild card is its printed suit in a flush check and no suit
  elsewhere; `is_suit` documents this.
