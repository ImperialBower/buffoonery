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
score a board written as JSON, YAML or a board line, get the step-by-step trace back as
JSON. The optimized component is 510 KiB (184 KiB gzipped); the budget is
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
buffoonery jokers "BARON+FOIL LUCKYCATx3.25"   # explain a line of joker tokens
buffoonery score board.yaml --jokers "BARON+FOIL"   # score with these jokers instead
buffoonery score --board "AS AH / / JOLLYJOKER"    # score a board line, no file
buffoonery line board.yaml               # the board as one line
buffoonery version                       # the engine version
```

`--jokers` and `--board` work on `score`, `link` and `parse`. `--jokers`
replaces the board's jokers; `--board` gives the whole board instead of a
file.

A board file is YAML, JSON or a board line; `-` or no file reads stdin, so a board copied
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

## Joker shorthand

A joker can be one token, the way a card is `AS`
([EPIC-08](docs/EPIC-08_Joker_Shorthand.md)). A line of tokens is a joker
row, left to right; spaces, tabs or commas separate them.

| Part | Means | Example |
| --- | --- | --- |
| ident | the joker; any case, `_` optional | `LUCKYCAT`, `lucky_cat` |
| `+FOIL` `+HOLO` `+POLY` `+NEG` | edition (long words also read) | `BARON+FOIL` |
| `+Nc` | chips override | `JOKER+50c` |
| `+Nm` | +mult override | `JOKER+8m` |
| `xN` | ×mult override, up to two decimals | `LUCKYCATx3.25` |
| `+OFF` | debuffed | `JOKER+OFF` |

Modifiers come in any order, each kind once. Tokens work in a board file
(`jokers: [BARON+FOIL, LUCKYCATx3.25]`), on the command line (`--jokers`),
and in Rust:

```rust
use buffoonery::prelude::*;

let jokers = parse_jokers("BARON+FOIL LUCKYCATx3.25 JOKER+OFF").unwrap();
assert_eq!(jokers[1].enhancement, Some(MPip::MultTimes2Dot(325)));
assert_eq!(shorthand_line(&jokers).unwrap(), "BARON+FOIL LUCKYCATx3.25 JOKER+OFF");
```

A board is always written back in the map form; a token is input only.

## Board line

A whole board fits on one line ([EPIC-09](docs/EPIC-09_Board_Line.md)).
Four sections, split by `/`, by place: played cards, held cards, jokers,
settings. A section may be empty; trailing sections may be left out.

```text
KS KH 2C / KD KC / BARON+FOIL LUCKYCATx3.25 JOKER+OFF
AH+BONUS JH+MULT 9H 6H 3H / / DROLLJOKER CRAFTYJOKER / FLUSH=2
AS AD AH / 3C 9D / BANNER BULL / money=20 discards=2 blind=WATER
```

| Section | Words | Example |
| --- | --- | --- |
| played, held | a card index, then `+` an enhancement and/or edition | `KS`, `KS+STEEL`, `AH+GLASS+POLY` |
| jokers | joker tokens, as above | `BARON+FOIL` |
| settings | `HAND=level`, `money=N` (may be negative), `hands=N`, `discards=N`, `blind=…` | `FLUSH=2 money=-5` |

The eight enhancements are `BONUS MULT WILD GLASS STEEL STONE GOLD LUCKY`,
each the value its tarot gives. A hand is its name without spaces
(`TWOPAIR`); a blind is `SMALL`, `BIG`, `NEEDLE`, `WATER` or `MANACLE` (`THE`
optional). Case is ignored; `_` is ignored in names. The grammar has no `$`
or `:`, so a line needs one pair of quotes in a shell.

A line works wherever a board is read: a file, stdin, `--board`, the page's
"Paste a board" box and a `#board=` link. `buffoonery line` writes any board
as its line; [`fixtures/cli/lines.txt`](fixtures/cli/lines.txt) holds all 20
golden boards. A board with an effect the line cannot spell (a `Custom`
joker, a `!Chips 50` card) has no line, and `line` says which part.

```rust
use buffoonery::prelude::*;

let spec: BoardSpec = "KS KH 2C / KD KC / BARON+FOIL / FLUSH=2".parse().unwrap();
assert_eq!(spec.held.len(), 2);
assert_eq!(spec.line().unwrap(), "KS KH 2C / KD KC / BARON+FOIL / FLUSH=2");
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