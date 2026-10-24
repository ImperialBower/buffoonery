---
type: decision
title: A whole board on one line, read by the one sniff
description: "`KS KH 2C / KD KC / BARON+FOIL / FLUSH=2`: four `/` sections by place (played, held, jokers, settings), parsed in `src/types/board_line.rs` with no new dependency; card tokens are word enhancements; settings are `key=value`; the component's `read_spec` spots a line, so every door reads it with no WIT change; `line()` refuses lossy writes."
tags: [shorthand, board, decision, epic-09]
timestamp: '2026-10-24T00:00:00Z'
resource: src/types/board_line.rs
---

# Decision

A whole board can be written as one line (EPIC-09):

```text
board    = section [ "/" section [ "/" section [ "/" section ] ] ]
played, held: card = index { "+" (enhancement | edition) }
jokers:       EPIC-08 joker tokens
settings:     HAND=n | money=[-]n | hands=n | discards=n | blind=WORD
enhancement = BONUS MULT WILD GLASS STEEL STONE GOLD LUCKY
```

The parser and writer live in the engine: `FromStr for BoardSpec`,
`BoardSpec::line`, `BoardLineError`, `parse_card`, `CardSpec::shorthand`.
The component's `read_spec` reads a line when the text is one line with no
`:`. The CLI has `--board LINE` and `buffoonery line`.

# Why

* **`/` by place, no labels.** `/` is in no token and needs no shell
  quoting. A label such as `played:` would bring back the `:` the sniff
  relies on, and the order is the order of the table.
* **Settings are `key=value`, and money is `money=20`, not `$20`.** In a
  double-quoted shell string `$2` is a variable, so `"… $20"` becomes
  `"… 0"`. A `key=value` word can never look like a card or a joker.
* **One error for a bad key and a bad hand.** `speed=3` and `FLSH=2` cannot
  be told apart, so both are `UnknownSetting`.
* **Card enhancements are eight words, each the tarot's value**, read from
  the tarot constants so the two cannot disagree. A card with any other
  enhancement has no token.
* **Only the 52 playing cards.** `BuffoonCard::from_str` also reads tarots
  and the blank card `__`; a card token does not.
* **The sniff, not a new WIT function.** One sniff is EPIC-07's rule. A
  non-empty YAML board always holds a `:`, so no text that read before reads
  differently now. The CLI, the paste box and links get the line for free.
* **`line()` is a `Result` and never rounds**, as `shorthand()` in EPIC-08:
  a `Custom` joker, a `!Chips 50` card, `edition: None`, or a card map with
  nothing set has no line, and the error names the section and position.
* **One error vocabulary.** `ShorthandError` and `LineError` grew (they were
  unreleased) instead of gaining twins.

# Consequences

* `fixtures/cli/lines.txt` pins every golden board's line; the engine, the
  CLI and the page tests all read it or check against it.
* A bad line through the component reads `parse: played[1]: "KX": unknown
  card`, as a bad YAML token does.
* The component grew from 494 to 510 KiB (budget 600).
* Open: a page link that carries the line instead of JSON, and a "copy
  line" button (EPIC-09 open questions 1 and 2).
