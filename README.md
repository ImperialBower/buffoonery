[![Crates.io](https://img.shields.io/crates/v/buffoonery.svg)](https://crates.io/crates/buffoonery)
[![Build and Test](https://github.com/ImperialBower/buffoonery/actions/workflows/ci.yaml/badge.svg)](https://github.com/ImperialBower/buffoonery/actions/workflows/ci.yaml)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.0-4baaaa.svg)](CODE_OF_CONDUCT.md)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](LICENSE-APACHE)
[![AI-BOM](https://img.shields.io/badge/AI--BOM-declared-8A2BE2?style=flat-square)](AI-BOM.md)

---

# buffoonery

![buffoonery_card_stacked_172x248.gif](docs/images/grimaldi/buffoonery_card_stacked_flipped_172x248.gif)

Foundation for a Balatro-style score calculator to help learn optimized play
and math. It models jokers, editions, boards and scoring on top of [cardpack](https://crates.io/crates/cardpack),
and it is growing a step-by-step score trace, boards you can write as YAML,
and a WebAssembly (WIT) component so other programs, like a joker calculator
in the browser, can use it. The engine started life as cardpack's `funky`
feature.

## How to use

```shell
git clone https://github.com/ImperialBower/buffoonery.git
cd buffoonery
make ayce
```

`make ayce` runs the whole quality gate. `make help` lists every target.

`make ayce` also builds the WebAssembly component and tests it. That step
needs the `wasm32-wasip2` target and Node (see `.tool-versions`):

```shell
rustup target add wasm32-wasip2
```

## WebAssembly component

[`component/`](component/README.md) wraps the engine in a WIT interface:
score a board written as JSON or YAML, get the step-by-step trace back as
JSON. The optimized component is 467 KiB (166 KiB gzipped); the budget is
500 KiB.

## What's in the box

- **Formatting:** rustfmt — see [toolchain decisions](.okf/decisions/toolchain.md)
- **Linting:** clippy at pedantic, warnings as errors — see [toolchain decisions](.okf/decisions/toolchain.md)
- **Supply-chain policy:** cargo-deny — see [toolchain decisions](.okf/decisions/toolchain.md)
- **Security scan:** cargo-audit plus cargo-deny advisories, daily in CI — see [toolchain decisions](.okf/decisions/toolchain.md)
- **Docs:** rustdoc with warnings as errors — see [toolchain decisions](.okf/decisions/toolchain.md)
- **Toolchain:** Rust, MSRV pinned in one place per file — see [toolchain decisions](.okf/decisions/toolchain.md)

![grimaldi_clown_214x261.gif](docs/images/grimaldi/grimaldi_clown_214x261.gif)

- Image of [Joseph Grimaldi](https://publicdomainreview.org/essay/the-memoirs-of-joseph-grimaldi/) possibly by Piercy Roberts, 1822