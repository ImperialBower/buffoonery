---
okf_version: '0.1'
---

# buffoonery

* [Getting started](getting-started.md) - starting point for this bundle.
* [Score keeps fractional mult](decisions/fractional-score.md) - `Score` is `{ chips: u64, mult: f64 }`, floored once in `score()`; why, and what it changed.
* [Joker Calculator page](decisions/web-calculator.md) - why the page lives in `web/`, why Vite, and how it stays free of scoring.
* [Joker shorthand](decisions/joker-shorthand.md) - one token per joker (`BARON+FOIL`, `LUCKYCATx3.25`): the grammar, why it is in the engine, why `MultTimes2Dot`, why `--jokers` replaces.
* [Board line](decisions/board-line.md) - a whole board on one line (`KS KH / KD / BARON+FOIL / FLUSH=2`): `/` by place, `money=20` not `$20`, the one sniff, why `line()` refuses lossy writes.
* [CLI crate](decisions/cli.md) - why the `buffoonery` command is a third crate over the component's adapter, the `clap` license check, and the table and link parity rules.
* [WIT component crate](decisions/wit-component-crate.md) - why the WIT boundary is its own crate, how it builds, shrinks and is tested through `jco`.
* [BoardSpec YAML](decisions/board-spec-yaml.md) - the `yaml` feature, `serde_norway`, the MIT exception, and why `CardSpec` parses by hand.
* [Toolchain choices](decisions/toolchain.md) - which tools the quality gate uses, at which version, and why.
* [Quality gate (make ayce)](processes/quality-gate.md) - the one-time `make setup`, what each phase runs and what green means.
* [EPIC-05 — Joker Calculator](references/epic-05-joker-calculator.md) - complete, closed 2026-10-20 (`docs/EPIC-05_Joker_Calculator.md`); pointer only.
* [EPIC-07 — Web and command line](references/epic-07-web-and-command-line.md) - complete, closed 2026-10-22; the CLI and the live page (`docs/EPIC-07_Web_And_Command_Line.md`); pointer only.
* [EPIC-08 — Joker shorthand](references/epic-08-joker-shorthand.md) - complete, closed 2026-10-22: `BARON+FOIL`, `LUCKYCATx3.25` tokens (`docs/EPIC-08_Joker_Shorthand.md`); pointer only.
* [EPIC-09 — A whole board on one line](references/epic-09-board-line.md) - complete, closed 2026-10-24: `KS KH 2C / KD KC / BARON+FOIL / FLUSH=2` (`docs/EPIC-09_Board_Line.md`); pointer only.
* [The engine](architecture/engine.md) - what buffoonery models, where it lives, and where it came from.
