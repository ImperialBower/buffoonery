# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **`ScoreTrace`: the score, step by step** (EPIC-05 Story 1).
  `BuffoonBoard::score_traced()`, `score_traced_with_seed`,
  `score_traced_with_rng` and `score_traced_with_registry` return every
  `ScoreOp` the folds applied, in order, as `ScoreStep`s: phase, source
  (hand, card or joker, with its position), part (effect, special, edition),
  retrigger pass, the op, whether it fired, whether it was rolled, and the
  running score after it. The total always equals the matching `score*`
  call, and replaying the ops from an empty `Score` reproduces every step.
  The `buffoon` example prints the trace.
- `ScoreOp` implements `Display` (`+4 Mult`, `+80 Chips`, `×3 Mult`) and
  gains `ScoreOp::is_noop()`.

### Changed

- **Scoring lives in `src/types/board/scoring.rs`** (EPIC-05 Story S2), moved
  out of `board.rs` word for word. No public API or behavior change.
- **Breaking: `Score` keeps fractional mult** (EPIC-05 Story S1). `Score` is
  now `{ chips: u64, mult: f64 }`. `multi_mult` no longer rounds up after each
  ×mult, so ×1.5 on 3 mult is 4.5, and `score()` rounds down once and returns
  `f64`, as Balatro does. `Score::new` takes `(u64, f64)`; the new
  `Score::from_usize` takes the engine's `usize` counts. `Score` no longer
  derives `Eq`, `Hash` or `Ord`, and `BuffoonBoard` no longer derives `Eq`.
  `ScoreOp::TimesMult` and the engine's x-mult math are `f64` (were `f32`),
  and `BuffoonBoard::round_score` is `f64`. `ScoreOp::Add` serializes mult as
  a float (`{"chips":5,"mult":3.0}`). Eleven tests that pinned rounded-up
  mults were re-baselined (for example Ramen ×1.01 now gives mult 1.01, not
  2); the `buffoon` example still scores 11254.
- **Requires cardpack 0.11.2.** cardpack 0.11.2 dropped a stale wasm-only
  `getrandom` 0.3 pin, so `buffoonery` no longer gets two copies of
  `getrandom`. The `getrandom` and `r-efi` skips in `deny.toml` are removed.

### Added

- **The engine**, copied from cardpack's `funky` module at cardpack commit
  `599a621` (last engine change there: `c762538`), with no git history. Paths
  changed (`cardpack::preludes::funky::*` → `buffoonery::prelude::*`); type
  names and behavior did not. 477 lib tests pass unchanged, and the `buffoon`
  example still scores 11254.
- Repository scaffold (dev-playbook, rust): `make ayce` quality gate, CI,
  daily security scan, `.okf/` knowledge bundle.
