# buffoonery-component

The buffoonery engine as a WebAssembly component (EPIC-05 Story 4). It exports
the `calculator` interface in [`wit/world.wit`](wit/world.wit):

| Export | In | Out |
| --- | --- | --- |
| `score` | a `BoardSpec` as JSON or YAML | its `ScoreTrace` as JSON, or an error naming the bad field |
| `score-with` | a `BoardSpec` as JSON or YAML, and `score-options`: an optional `u64` seed (a `BigInt` in JavaScript) and a list of custom effects (`id`, and a `ScoreOp` as JSON) | its `ScoreTrace`: chances (Lucky cards, Misprint) rolled from the seed, rolled steps marked `"rolled": true`; each `Custom(id)` card or joker applies its effect's op |
| `parse` | a `BoardSpec` as JSON or YAML | the same spec as JSON (shape check only) |
| `catalog` | — | every joker as JSON |
| `version` | — | the engine version |

This crate is an adapter. It owns the formats and does no scoring. It is not
published.

## Build and test

```bash
rustup target add wasm32-wasip2
make component
```

`make component` builds `target/wasm32-wasip2/component/buffoonery_component.wasm`,
shrinks it with `jco opt` (`wasm-opt -Oz`), turns it into an ES module with
`jco transpile` (into `js/pkg/`), and runs `js/golden.test.mjs`: every
[golden fixture](../fixtures/golden/README.md) against the built component.
`cargo test -p buffoonery-component` runs the same fixtures natively.

## Size

Measured 2026-10-20, Rust 1.99.1, `--profile component`:

| File | Raw | gzip -9 |
| --- | --- | --- |
| `buffoonery_component.wasm` | 596 KiB | 178 KiB |
| after `jco opt -Oz` | 488 KiB | 175 KiB |

The budget is 500 KiB raw after `jco opt`. Most of the size is the engine, the
YAML parser and the joker data. `score-with` added 21 KiB: 8 KiB for the
seeded RNG (`StdRng`), 13 KiB for reading a `ScoreOp` from JSON and the
effect registry. That leaves 12 KiB of room. cardpack's `i18n` and `colored-display`
features are off.
