---
type: architecture
title: The engine — Balatro-style cards and scoring
description: What buffoonery models (jokers, planets, tarots, spectrals as MPip data; four-phase scoring; shop, vouchers, editions, blinds), where it lives, and where it came from.
tags: [engine, balatro, scoring, architecture]
timestamp: '2026-10-19T18:00:00Z'
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
  ([decision](/decisions/fractional-score.md)). `score_traced()` and its
  seeded/registry twins return a `ScoreTrace` (`src/types/trace.rs`): one
  `ScoreStep` per `ScoreOp` applied. Every fold applies ops through
  `trace::apply_step`, so tracing cannot change a number.
* **Decks** (`src/decks/`) — basic decks, 112 jokers (`Joker::ALL_JOKERS`;
  8 `Blank` with reasons in `Joker::BLANK_WITH_REASON`), planets, tarots,
  spectrals. `src/decks/joker_catalog.rs` names every joker and maps its
  ident (the const name) to the card: `Joker::catalog()`, `by_ident`,
  `name`, `info`.
* **Boards as data** (`src/types/board_spec.rs`) — `BoardSpec` (Rust, JSON,
  or YAML behind the default `yaml` feature) → `BuffoonBoard::try_from`,
  with a `BoardSpecError` naming any bad field
  ([decision](/decisions/board-spec-yaml.md)).
* **Economy** — `Shop`, `Voucher`, editions, blinds with boss blinds.
* **WIT component** (`component/`, its own crate) — the engine behind
  `imperialbower:solver/calculator` on `wasm32-wasip2`: `score`, `catalog`,
  `version`, strings in and out. The engine crate has no WIT or JSON code
  ([decision](/decisions/wit-component-crate.md)).
* **Joker Calculator** (`web/`, Vite) — the page over the component.
  `web/src/model.js` maps page state to a `BoardSpec` and the trace to the
  page; `main.js` only draws ([decision](/decisions/web-calculator.md)).
* **Debuffed jokers** do nothing while scoring: effect and edition are no-op
  steps, and they add no retriggers, rule changes or odds. Readers outside
  scoring (boss disabling, draws, rerolls) still ignore the flag.
* **Golden fixtures** (`fixtures/golden/`) — `BoardSpec` YAML + `ScoreTrace`
  JSON pairs read by `tests/golden.rs`, the component's native tests, and its
  Node tests through `jco`.

# Origin

Copied on 2026-10-18 from cardpack's `funky` module at cardpack commit
`599a621` (last engine change there: `c762538`), with no git history. The
engine's earlier history, and its design documents (EPIC-01 and children,
EPIC-06 Funky Extraction), stay in the cardpack repo. EPIC-05 moved here on
2026-10-19 ([EPIC-05](/references/epic-05-joker-calculator.md)). Cardpack's docs:
<https://github.com/ImperialBower/cardpack.rs/tree/main/docs>.

# Next

EPIC-05's stories run here: a fractional `Score` (done, S1), moving the scoring folds out
of `board.rs` (done, S2), a `ScoreTrace` (done, Story 1), a `BoardSpec` written in Rust or YAML (done, Story 3), a
joker catalog (done, Story 2), golden fixtures (done, Story 5), a WIT component (built, Story 4; its browser test moved to Story 8), the calculator page (Story 6), its hand editor (Story 7) and the browser parity test (Story 8): Milestone C is built. Milestone D (polish) is next.

# Try it

```shell
cargo run --example buffoon      # four-phase scoring, phase by phase (scores 11254)
cargo run --example funky_tour   # seeded tour: rounds, editions, shop, spectrals
```
