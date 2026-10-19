---
type: architecture
title: The engine — Balatro-style cards and scoring
description: What buffoonery models (jokers, planets, tarots, spectrals as MPip data; four-phase scoring; shop, vouchers, editions, blinds), where it lives, and where it came from.
tags: [engine, balatro, scoring, architecture]
timestamp: '2026-10-19T00:00:00Z'
---

# What it is

A [Balatro](https://www.playbalatro.com/)-style card engine, built on
[cardpack](https://crates.io/crates/cardpack)'s card model (`Pip`, `PipType`,
`FrenchSuit`, `BasicCard`, `BasicPile`, `Ranged`). It exists to power a
Balatro **score solver** and to allow **custom decks and jokers**. Every
`types/*` file derives serde.

# Key constructs

* **`BuffoonCard`** (`src/types/buffoon_card.rs`) — cardpack `Pip`s for suit
  and rank, plus `card_type: BCardType`, `enhancement: MPip`, `edition`,
  `resell_value`, `debuffed`.
* **`MPip`** (`src/types/mpip.rs`) — a 142-variant enum that makes card
  effects **data, not code**; interpreted at scoring time.
* **`BuffoonPile`** — implements cardpack's `Ranged`, so combinatorics and
  hand detection reuse cardpack's engine.
* **Scoring** (`src/types/board/scoring.rs`, split out of `board.rs`) — `BuffoonBoard::score()` /
  `score_with_seed()`: base hand, played cards, held cards, then jokers left
  to right; never panics. Open effects go through `EffectRegistry`
  (`src/types/effect.rs`). `Score` keeps fractional mult and floors once
  ([decision](/decisions/fractional-score.md)).
* **Decks** (`src/decks/`) — basic decks, 112 jokers (`Joker::ALL_JOKERS`;
  8 `Blank` with reasons in `Joker::BLANK_WITH_REASON`), planets, tarots,
  spectrals.
* **Economy** — `Shop`, `Voucher`, editions, blinds with boss blinds.

# Origin

Copied on 2026-10-18 from cardpack's `funky` module at cardpack commit
`599a621` (last engine change there: `c762538`), with no git history. The
engine's earlier history, and its design documents (EPIC-01 and children,
EPIC-06 Funky Extraction), stay in the cardpack repo. EPIC-05 moved here on
2026-10-19 ([EPIC-05](/references/epic-05-joker-calculator.md)). Cardpack's docs:
<https://github.com/ImperialBower/cardpack.rs/tree/main/docs>.

# Next

EPIC-05's stories run here: a fractional `Score` (done, S1), moving the scoring folds out
of `board.rs` (done, S2), a `ScoreTrace`, a `BoardSpec` written in Rust or YAML, a
joker catalog, and a WIT component.

# Try it

```shell
cargo run --example buffoon      # four-phase scoring, phase by phase (scores 11254)
cargo run --example funky_tour   # seeded tour: rounds, editions, shop, spectrals
```
