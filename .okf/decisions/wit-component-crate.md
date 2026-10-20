---
type: decision
title: The WIT component is its own workspace crate, tested through jco
description: "component/ (buffoonery-component, unpublished) holds the WIT boundary so the engine crate carries no wit-bindgen or JSON code; it builds with plain cargo for wasm32-wasip2, is shrunk by jco opt, and is tested in Node through the jco-transpiled module."
tags: [wit, wasm, component, decision, epic-05]
timestamp: '2026-10-20T12:22:00Z'
resource: component/
---

# Decision

The WIT component (EPIC-05 Story 4) is a separate crate in a Cargo
workspace: `component/`, package `buffoonery-component`, `publish = false`.

* **Adapter, not engine.** It owns the formats; it exports `score`, `score-with` (added in Milestone D: `score` with `score-options`, an optional `u64` seed so chances roll, and custom effects, each an id and a `ScoreOp` as JSON that the adapter registers as a fixed-op `Effect`; +21 KiB; it replaced the short-lived `score-seeded`), `parse` (added in Story 8: a spec as JSON or YAML in, JSON out), `catalog` and `version`. The seed and the effects are arguments, not `BoardSpec` fields: they say how to score a board, not what is on it, as the engine's `EffectRegistry` sits beside the board. A `ScoreOp` crosses as a JSON string because WIT has no recursive types (`Seq`). `score` reads a `BoardSpec`
  as JSON (input starting with `{`) or YAML, and writes the `ScoreTrace` as
  JSON. The engine crate gained only `buffoonery::VERSION`.
* **Plain cargo builds it.** `cargo build -p buffoonery-component --target
  wasm32-wasip2 --profile component` emits a component directly, so
  `cargo component` is not needed. `wit-bindgen` is a `wasm32`-only
  dependency; the exports are plain Rust functions, so native tests call them.
* **The `component` profile** (`opt-level = "z"`, LTO, one codegen unit,
  `panic = "abort"`, strip) is a named profile so the engine's `release`
  build does not change.
* **Tested through `jco`, in Node**, not `wasmtime`: the transpiled ES module
  is what a browser loads. `component/js/` pins `@bytecodealliance/jco` and
  `preview2-shim` exactly; Node's version is in `.tool-versions`.

# Why not a feature on the engine crate

A `component` feature would put `wit-bindgen`, `serde_json` and a `cdylib`
crate type into the published engine. The domain-kernel split says the
adapters own the formats. cardpack's self-dev-dependency trap
([cardpack decision](https://github.com/ImperialBower/cardpack.rs/blob/main/.okf/decisions/examples-flag-free-alias.md))
does not apply: a separate member is not a dependency of the engine, and the
member's path dependency carries a version, so `wildcards` stays quiet.

# Gotchas (load-bearing)

* **Plain `wasm-opt` cannot read a component.** Use `jco opt`, which runs
  `wasm-opt` on the core modules inside it.
* **Name the wasm features.** `wasm-opt -Oz` alone rejects `memory.copy`;
  `-all` emits typed function references the component validator rejects.
  The `opt` script in `component/js/package.json` lists the features Rust
  uses: bulk memory (and `-opt`), non-trapping float-to-int, sign-ext,
  mutable globals, multivalue, reference types.
* **Generated code trips one clippy lint**, `same_length_and_capacity`; it is
  allowed on the `bindings` module only.
* **Installed `wasmtime` 24 cannot invoke component exports** from the CLI.

# Size

494 KiB after `jco opt -Oz` (178 KiB gzipped; 6 KiB of room left), against a 500 KiB budget that
`golden.test.mjs` checks. Figures in `component/README.md`.
