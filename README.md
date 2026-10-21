[![Crates.io](https://img.shields.io/crates/v/buffoonery.svg)](https://crates.io/crates/buffoonery)
[![Build and Test](https://github.com/ImperialBower/buffoonery/actions/workflows/ci.yaml/badge.svg)](https://github.com/ImperialBower/buffoonery/actions/workflows/ci.yaml)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.0-4baaaa.svg)](CODE_OF_CONDUCT.md)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](LICENSE-APACHE)
[![AI-BOM](https://img.shields.io/badge/AI--BOM-declared-8A2BE2?style=flat-square)](AI-BOM.md)

---

# buffoonery

![buffoonery_harlequin_flipped_172x248.png](docs/images/grimaldi/buffoonery_harlequin_flipped_172x248.png)

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
make setup
make ayce
```

`make ayce` runs the whole quality gate. `make help` lists every target.

`make setup` runs once. It needs rustup and Node (versions in
`.tool-versions`). It adds the `wasm32-wasip2` target, installs
`cargo-deny` and `cargo-audit`, installs the npm packages, and installs
Chromium for the browser tests. On Linux, Chromium's system libraries need
sudo. The devcontainer runs it for you.

## WebAssembly component

[`component/`](component/README.md) wraps the engine in a WIT interface:
score a board written as JSON or YAML, get the step-by-step trace back as
JSON. The optimized component is 494 KiB (178 KiB gzipped); the budget is
600 KiB.

## Joker Calculator

[`web/`](web/) is a Balatro joker calculator built on the component: pick a
hand, slot in jokers, override or debuff them, and read the score step by
step. It does no scoring of its own.

```shell
make web                 # builds the component, tests the page, builds web/dist
make dev                 # serves it with live reload
make e2e                 # opens every golden fixture in Chromium and checks the table
```

Each push to `main` publishes the page to
<https://imperialbower.github.io/buffoonery/> (`.github/workflows/pages.yaml`),
after the same tests pass.

## Command line

[`cli/`](cli/) is the `buffoonery` command. It calls the same adapter
functions as the page, so it gives the same numbers.

```shell
cargo install --git https://github.com/ImperialBower/buffoonery buffoonery-cli

buffoonery score board.yaml              # the trace as a table
buffoonery score --json board.yaml       # the trace as JSON, as the component returns it
buffoonery score --seed 7 board.yaml     # roll Lucky cards and Misprint from a seed
buffoonery score --all board.yaml        # every step, not only the ones the page shows
buffoonery link board.yaml               # a link that opens the board in the page
buffoonery parse board.yaml              # the board as JSON
buffoonery catalog --find jolly          # the jokers, by ident or name
buffoonery version                       # the engine version
```

A board file is YAML or JSON; `-` or no file reads stdin, so a board copied
from the page scores with `pbpaste | buffoonery score`. Exit codes: 0 done,
1 the board is not valid (the message names the field), 2 bad usage or a
missing file.

```text
$ buffoonery score fixtures/golden/02_pair_additive_jokers.yaml
 #  Source                Detail  Effect             Chips  Mult  Score
 1  Pair (level 1)                +10 Chips +2 Mult     10     2     20
 2  A♠ · played 1                 +11 Chips             21     2     42
 5  A♥ · played 2                 +11 Chips             32     2     64
 8  Jolly Joker · slot 1          +8 Mult               32    10    320
10  Sly Joker · slot 2            +50 Chips             82    10    820
Pair  82 × 10 = 820
```

## What's in the box

- **Formatting:** rustfmt — see [toolchain decisions](.okf/decisions/toolchain.md)
- **Linting:** clippy at pedantic, warnings as errors — see [toolchain decisions](.okf/decisions/toolchain.md)
- **Supply-chain policy:** cargo-deny — see [toolchain decisions](.okf/decisions/toolchain.md)
- **Security scan:** cargo-audit plus cargo-deny advisories, daily in CI — see [toolchain decisions](.okf/decisions/toolchain.md)
- **Docs:** rustdoc with warnings as errors — see [toolchain decisions](.okf/decisions/toolchain.md)
- **Toolchain:** Rust, MSRV pinned in one place per file — see [toolchain decisions](.okf/decisions/toolchain.md)

![grimaldi_clown_214x261.gif](docs/images/grimaldi/grimaldi_clown_214x261.gif)

- Image of [Joseph Grimaldi](https://publicdomainreview.org/essay/the-memoirs-of-joseph-grimaldi/) possibly by Piercy Roberts, 1822