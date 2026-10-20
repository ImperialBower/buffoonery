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
- **Joker catalog** (EPIC-05 Story 2): the Balatro name of all 112 jokers,
  and a stable identifier (the const name, e.g. `"JOLLY_JOKER"`).
  `Joker::catalog()` lists a `JokerInfo` (ident, name, rarity, cost, effect,
  blank reason) per joker; `Joker::by_ident`, `Joker::ident`, `Joker::name`
  and `Joker::info` look one up. An edited joker keeps its name and ident.
  The `buffoon` example's trace shows joker names.
- **`BoardSpec`: a board as data** (EPIC-05 Story 3). Write a board in Rust,
  JSON or YAML (hand levels, played and held cards, jokers by ident with
  optional effect/edition/debuff overrides, money, hands, discards, blind)
  and build it with `BuffoonBoard::try_from(spec)`. Bad input is a
  `BoardSpecError` that names the field, e.g.
  `jokers[1]: unknown joker ident "NOT_A_JOKER"`. `BoardSpec::from_yaml` and
  `to_yaml` are behind the new default-on `yaml` feature (`serde_norway`
  0.9.42, as cardpack uses); `deny.toml` allows MIT for its parser,
  `unsafe-libyaml-norway`, by name.
- `PokerHands::set_level(hand_type, level)`.
- `ScoreOp` implements `Display` (`+4 Mult`, `+80 Chips`, `×3 Mult`) and
  gains `ScoreOp::is_noop()`.
- **Golden fixtures** (EPIC-05 Story 5): 14 boards in
  `fixtures/golden/` as `BoardSpec` YAML, each with the `ScoreTrace` JSON the
  engine must return. `tests/golden.rs` checks them;
  `BLESS=1 cargo test --test golden` rewrites the JSON after a deliberate
  scoring change. They are the shared contract for the WIT component and the
  calculator.
- **WIT component** (EPIC-05 Story 4): the new workspace crate
  `component/` (`buffoonery-component`, not published) builds a
  `wasm32-wasip2` component exporting `imperialbower:solver/calculator`:
  `score` (a `BoardSpec` as JSON or YAML in, a `ScoreTrace` as JSON out),
  `catalog` and `version`. `make component` shrinks it with `jco opt`,
  transpiles it with `jco`, and runs the golden fixtures through it in Node;
  CI does the same. 467 KiB after optimization (166 KiB gzipped).
- `buffoonery::VERSION`, the crate version.
- **The Joker Calculator page** (EPIC-05 Story 6), in `web/`: pick a hand,
  slot jokers, override their effects or debuff them; every number comes
  from the WIT component's trace. `make web` tests and builds it.
- **The calculator's hand editor** (EPIC-05 Story 7): choose up to 5 played
  and 8 held cards by rank and suit; the engine finds the hand type, and a
  level editor sets any hand's level. The breakdown groups its rows by the
  card or joker that made them.
- **Boards in links, and the browser parity test** (EPIC-05 Story 8). The
  component exports `parse`: a `BoardSpec` as JSON or YAML in, the same spec
  as JSON out. The calculator opens a board from `#board=` in its link and
  keeps the link up to date. `make e2e` opens every golden fixture in the
  page with Playwright and checks every table cell against its trace.
- **Copy and paste boards** (EPIC-05 Milestone D): "Copy board" (JSON),
  "Copy link", and "Paste a board" (YAML or JSON) in the calculator.
- **Joker edition picker** (EPIC-05 Milestone D): each joker card in the
  calculator has an edition menu (Foil, Holographic, Polychrome, Negative).
- **`make setup`**: one-time setup for `make ayce` (the wasm target,
  `cargo-deny`, `cargo-audit`, npm packages, and Chromium for the browser
  tests). The devcontainer runs it.
- **Seeded scoring and custom effects in the component and the calculator**
  (EPIC-05 Milestone D): the WIT export `score-with(spec, options)` rolls
  Lucky cards and Misprint from an optional `u64` seed, and scores each
  `MPip::Custom(id)` with a `ScoreOp` given as JSON. The calculator has a
  seed field, marks rolled steps, has a "Custom" effect kind where you type
  an effect (`+20 Chips, ×2 Mult`), and keeps the seed and the effects in
  the link.
- `BuffoonBoard::score_traced_with_rng_and_registry` and
  `score_traced_with_seed_and_registry`: chances and custom effects in one
  traced score.
- A joker carrying `MPip::Chips(n)` adds n chips. No Balatro joker carries
  it; the calculator's "+ Chips" override does.

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

### Fixed

- A debuffed joker no longer scores: its effect and edition are no-op trace
  steps, and it adds no retriggers, hand-rule changes or odds. Before, the
  `debuffed` flag was never read while scoring.

### Added

- **The engine**, copied from cardpack's `funky` module at cardpack commit
  `599a621` (last engine change there: `c762538`), with no git history. Paths
  changed (`cardpack::preludes::funky::*` → `buffoonery::prelude::*`); type
  names and behavior did not. 477 lib tests pass unchanged, and the `buffoon`
  example still scores 11254.
- Repository scaffold (dev-playbook, rust): `make ayce` quality gate, CI,
  daily security scan, `.okf/` knowledge bundle.
