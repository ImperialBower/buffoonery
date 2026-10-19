---
okf_version: '0.1'
---

# buffoonery

* [Getting started](getting-started.md) - starting point for this bundle.
* [Score keeps fractional mult](decisions/fractional-score.md) - `Score` is `{ chips: u64, mult: f64 }`, floored once in `score()`; why, and what it changed.
* [Toolchain choices](decisions/toolchain.md) - which tools the quality gate uses, at which version, and why.
* [Quality gate (make ayce)](processes/quality-gate.md) - what each phase runs and what green means.
* [EPIC-05 — Joker Calculator](references/epic-05-joker-calculator.md) - the active EPIC (`docs/EPIC-05_Joker_Calculator.md`); pointer only.
* [The engine](architecture/engine.md) - what buffoonery models, where it lives, and where it came from.
