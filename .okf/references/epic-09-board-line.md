---
type: reference
title: EPIC-09 — A whole board on one line
description: "Planned (written 2026-10-23): a one-line text form for a whole board (`KS KH 2C / KD KC / BARON+FOIL / FLUSH=2`), parsed in the engine, read by the component's one sniff, and on the CLI as `--board` and `line`."
tags: [epic, shorthand, dsl, board, cli, planned]
timestamp: '2026-10-23T00:00:00Z'
resource: docs/EPIC-09_Board_Line.md
---

# What it covers

The follow-on that [EPIC-08](epic-08-joker-shorthand.md) named. Four
`/`-separated sections, by place: played cards, held cards, jokers (EPIC-08's
token line, unchanged), settings (`FLUSH=2 money=20 hands=3 discards=2
blind=WATER`).

# Key rules (see the EPIC)

* **Card tokens** are the index plus an enhancement word (`BONUS`, `MULT`,
  `WILD`, `GLASS`, `STEEL`, `STONE`, `GOLD`, `LUCKY`, each the tarot's fixed
  `MPip`) and an edition word: `KS+STEEL`, `AH+GLASS+POLY`.
* **Settings are `key=value`.** `money=20`, not `$20`: `$2` is a shell
  variable in a double-quoted string.
* **One sniff.** `read_spec` reads a line when the text is one line with no
  `:`; a YAML board always has a `:`. The CLI, the page's paste box and
  `#board=` links get the line with no WIT change.
* **`line()` refuses lossy writes**, as `shorthand()` does in EPIC-08.
* **Every golden fixture must round-trip** through its line to the same
  trace; `fixtures/cli/lines.txt` pins the 20 lines.
