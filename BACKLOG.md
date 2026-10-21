# Backlog

> Refreshed by the `/backlog` skill on 2026-10-20, at `main` `4b337a5`. An index,
> not a plan: each item links to where the work is described. Items tagged 🤖
> were proposed by automation; they are suggestions, not commitments.
> Technical debt detail lives in [docs/TECHNICAL_DEBT.md](docs/TECHNICAL_DEBT.md).

Sources scanned: `docs/EPIC-*.md`, `CHANGELOG.md` (Unreleased), source comments
(`TODO`/`FIXME`/`HACK`/`XXX`: none found; every `HACK` hit is the Hack joker),
"known gap" notes in `src/`, GitHub issues (none open), open PRs (none).

## EPICs and features

- [ ] **EPIC-07 — the CLI and the page on the web ("DOORS")** — Planned, 0 of 6
  phases. A `buffoonery` command-line tool (`version`, `parse`, `catalog`,
  `score`, `link`) over the same adapter as the page, and the calculator
  published on GitHub Pages. Static page only, no server (decided 2026-10-20).
  Phase 4 needs a maintainer to enable Pages in the repo settings.
  ([docs/EPIC-07_Web_And_Command_Line.md](docs/EPIC-07_Web_And_Command_Line.md))
- [ ] **Release `buffoonery` 0.1.0** — crates.io has 0.0.1; `CHANGELOG.md`
  Unreleased holds 27 entries, including breaking changes (fractional `Score`,
  `ScoreOp` JSON). ([CHANGELOG.md](CHANGELOG.md))
- [ ] **EPIC-05 open questions, deferred** — risk 6 (trace size with stacked
  retriggers), risk 10 (typed WIT records instead of JSON strings), risk 11
  (YAML is a second contract; renames need a version bump).
  ([EPIC-05 Open questions](docs/EPIC-05_Joker_Calculator.md))

## Bugs and engine gaps

Found by the automated review (detail in `docs/TECHNICAL_DEBT.md`); each was
confirmed by reading the code:

- [ ] 🤖 **Four of a Kind base is 50 × 7; Balatro's is 60 × 7.**
  (`src/types/hands.rs:76`)
- [ ] 🤖 **Planet cards level hands with base values, not level-up values**
  (Jupiter +35/+4 instead of +15/+2; also Saturn, Earth, Mars).
  (`src/decks/planet.rs:147-182`)
- [ ] 🤖 **Stone cards can make Two Pair and Full House.**
  (`src/types/buffoon_pile.rs:542`, `:598`)
- [ ] 🤖 **Joker Stencil counts Negative jokers as filling a slot.**
  (`src/types/board/scoring.rs:750`)

Known and tracked before:

- [ ] **`blind_target` is `usize`** — overflows past 4.29 × 10⁹ on wasm32
  (32-bit `usize`), while `Score` is already wide. (`src/types/board.rs:88`;
  EPIC-05 S1 follow-up, `docs/EPIC-05_Joker_Calculator.md:194`)
- [ ] **Wild cards do not count toward a flush** — `MPip::Wild` exists and
  `MPip::is_wild` is never called; flush sizing ignores it.
  (`src/types/mpip.rs:425`, `src/types/buffoon_pile.rs` `count_largest_same_suit_with`)
- [ ] **8 Blank jokers** — DNA, To Do List, Mail-In Rebate, Trading Card,
  Reserved Parking, Lucky Cat, Matador, Diet Cola; each with its blocker in
  `Joker::BLANK_WITH_REASON`. (`src/decks/joker.rs:306`)
- [ ] **Run-level tarots are no-ops** — Death, Judgement, The Hermit, The Wheel
  of Fortune. (`src/types/board.rs:531`)
- [ ] **Hiker bumps once per hand, not per trigger** — waits on scoring
  becoming mutating. (`src/types/board.rs:2195`)
- [ ] **Card and joker destruction is data only** — Glass card breakage
  (`src/types/board/scoring.rs:184`); Gros Michel's end-of-round roll
  (`src/types/board/scoring.rs:496`).

## Tech debt

- [ ] **EPIC-05 task 0i: stale comment** — says Joker Stencil is not applied;
  it is. Loyalty Card still is not. (`src/types/board/scoring.rs:361`)
- [ ] **File paths in the `.wasm`** — panic messages carry ~6 KiB of build
  paths, including the builder's home folder. `--remap-path-prefix` would
  shorten them. (`component/README.md`, Size)
- [ ] More, including automated review findings: see
  [docs/TECHNICAL_DEBT.md](docs/TECHNICAL_DEBT.md).

## Docs and notes

- [ ] **EPIC-05 status table is stale** — Stories 7–8 and Milestone D still
  say "not yet merged"; all merged in PRs #3 and #4. Mark EPIC-05 Complete.
  (`docs/EPIC-05_Joker_Calculator.md:18-35`)
- Created Stone cards have a fixed base, a deliberate divergence from
  Balatro. (`src/decks/basic.rs:193`)
- Component size: 494 of 600 KiB; YAML is 40% of it. The JSON-only lever is
  documented. (`component/README.md`, Size)

## Recently completed

EPIC-05 (Joker Calculator): Stories S1–S2 and 1–8, Milestone D (export,
edition picker, seeded mode, custom effects), risks 7 and 8 (kickers,
Ace-low straights), and the Four Fingers / Shortcut bonus-joker gap. Merged
in PRs #1–#4.
