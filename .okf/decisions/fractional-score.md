---
type: decision
title: Score keeps fractional mult
description: "Score is { chips: u64, mult: f64 }; nothing rounds until score(), which floors once. No Eq/Hash/Ord. Engine x-mult math is f64, not f32."
tags: [scoring, score, decision, epic-05]
timestamp: '2026-10-19T00:00:00Z'
resource: src/types/score.rs
---

# Decision

`Score` (`src/types/score.rs`) is `{ chips: u64, mult: f64 }` (EPIC-05 Story
S1, decision 0f, 2026-10-19).

| Choice | Why |
|---|---|
| `mult: f64`, no rounding between steps | Balatro keeps fractional mult. The old `multi_mult` rounded up after every ×mult, so ×1.01 on 1 mult gave 2, and two ×1.5 rounded twice. |
| `chips: u64`, saturating adds | Balatro's chips are whole numbers. `usize` is 32 bits on wasm32, and late-game scores pass 4.29 × 10⁹. |
| `score() -> f64`, floored once | Balatro also computes in floats and floors the hand score. `f64` holds scores past `u64::MAX`. |
| No `Eq`, `Hash`, `Ord` | Floats cannot derive them. The old derived `Ord` sorted by chips first, which never meant "higher score". Compare `score()` values. |
| Engine x-mult math is `f64` (was `f32`) | In `f32`, Ramen's ×1.99 is 1.99000000954; the error can move a floored score by one. |
| `Score::from_usize(chips, mult)` | The engine counts card chips, hand levels and `+mult` in `usize`. This is the one place they become a `Score`. |

`BuffoonBoard::round_score` is `f64` for the same reason. `BuffoonBoard`
no longer derives `Eq`.

# Consequences

* The pinned `ScoreOp` JSON changed on purpose: `Add` is now
  `{"chips":5,"mult":3.0}`. `AddChips`, `AddMult` and `TimesMult` keep their
  shape.
* Eleven tests that asserted a rounded-up mult were re-baselined (Hologram
  ×1.25, Baseball ×1.5, Ramen ×1.01, Constellation, Vampire, Ancient Joker,
  Smeared + Ancient, Steel Joker, held Steel, joker Polychrome). The
  `buffoon` example still scores 11254: its hand has no fractional step.
* Not done here: `blind_target` is still `usize`. Blind targets past
  4.29 × 10⁹ would overflow on wasm32.

# Citations

* cardpack `docs/EPIC-05_Joker_Calculator.md`, Story S1 and open question 1.
* [The engine](/architecture/engine.md).
