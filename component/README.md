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
| `buffoonery_component.wasm` | 603 KiB | 180 KiB |
| after `jco opt -Oz` | 494 KiB | 178 KiB |

The budget is 600 KiB raw after `jco opt`, checked by `js/golden.test.mjs`.
It was 500 KiB until 2026-10-20, when the component reached 494 KiB. Raised
on purpose, not cut: most of the size is YAML, and YAML is a feature.

Where the code goes (measured 2026-10-20 on an unstripped build, before
`jco opt`; 502 KiB of code, plus 88 KiB of data):

| Pulled in by | Code |
| --- | --- |
| YAML (`serde_norway`, `unsafe-libyaml-norway`) | 180 KiB |
| JSON (`serde_json`) | 115 KiB |
| The engine and cardpack | 92 KiB |
| Other std / core / alloc | 58 KiB |
| Formatting (`fmt`) | 27 KiB |
| Unicode tables | 22 KiB |
| Other serde glue | 9 KiB |

A component with no YAML (buffoonery's `yaml` feature off, JSON in only) is
295 KiB after `jco opt` (114 KiB gzipped). That is the lever if size ever
matters more than YAML in the page: the paste box, links and fixtures would
take JSON only. Smaller levers: panic messages carry about 6 KiB of
file paths (`--remap-path-prefix` would shorten them).

Growth so far: `score-with` added 21 KiB (8 KiB for the seeded RNG,
`StdRng`; 13 KiB for reading a `ScoreOp` from JSON and the effect
registry). Scoring only the cards that make the hand (kickers) added 6 KiB.
cardpack's `i18n` and `colored-display` features are off.
