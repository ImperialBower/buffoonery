---
type: reference
title: EPIC-10 — Every suit at once (Wild, Stone, Smeared)
description: "Built 2026-10-24 on `epic10-suits` (all six phases): one `BuffoonCard::is_suit` rule (Stone is no suit, Wild is every suit, Smeared merges colours) used by flushes, the suit jokers, Blackboard and Ancient Joker; `blind_target` becomes `f64`."
tags: [epic, defect, suit, wild, scoring, built]
timestamp: '2026-10-24T00:00:00Z'
resource: docs/EPIC-10_Suit_Correctness.md
---

# What it covers

Five scoring defects with one cause: every suit check compares the printed
suit by hand.

* A Wild card does not count toward a flush (`AH KH 9H 4H 2S+WILD` is High
  Card).
* A Wild card pays only its printed suit's joker.
* A Stone card's hidden suit pays Wrathful Joker and passes Blackboard.
* Smeared Joker reaches flushes and Ancient Joker, but not the suit jokers
  or Blackboard.
* `blind_target` is `usize`, 32 bits on wasm32.

# Key rules (see the EPIC)

* **`is_suit(card, suit, rules)`**: Stone → no; Wild → yes; printed suit →
  yes; Smeared colour → yes; else no. Named after Balatro's `Card:is_suit`.
* **A Wild card counts in every suit's flush group** and scores in the
  flush.
* **The 20 existing golden traces must not change**; fixtures 21 and 22 pin
  the fixes.
* **`blind_target: f64`**, the type of `round_score`.

# Status

All six phases built on `epic10-suits` (2026-10-24). See the
[suit rule decision](/decisions/suit-rule.md).
