---
type: decision
title: A joker is one token, read and written by the engine
description: "`BARON+FOIL`, `LUCKYCATx3.25`, `JOKER+50c+OFF`: a second spelling of `JokerSpec`, parsed in `src/types/joker_shorthand.rs` with no new dependency; `MPip::MultTimes2Dot` makes ×3.25 exact; `shorthand()` is a `Result`; output stays the map; `--jokers` replaces a board's jokers."
tags: [shorthand, joker, decision, epic-08]
timestamp: '2026-10-22T00:00:00Z'
resource: src/types/joker_shorthand.rs
---

# Decision

A joker can be written as one token (EPIC-08):

```text
token    = ident { modifier }
ident    = letter { letter | digit | "_" }   ; any case; "_" is ignored
modifier = "+" edition | "+" integer "c" | "+" integer "m" | "x" number | "+OFF"
edition  = FOIL | HOLO | POLY | NEG (or the long word)
number   = integer [ "." digit [ digit ] ]
```

The parser and writer live in the engine: `parse_joker`, `parse_jokers`,
`shorthand_line`, `FromStr for JokerSpec`, `JokerSpec::shorthand`. A
`BoardSpec` in YAML or JSON reads a joker as a token or a map, and always
writes the map. The CLI has `buffoonery jokers LINE` and `--jokers LINE`.

# Why

* **In the engine.** It is a spelling of a domain value with no I/O, like
  `BuffoonCard::from_str("AS")`. The CLI and the component call it, so there
  is one parser.
* **`MultTimes2Dot`, not a change to `MultTimes1Dot`.** Boards on disk say
  `!MultTimes1Dot 15`; changing its unit to hundredths would make them ×0.15.
  Not an `f64` either: `MPip` is `Eq`/`Hash` data in `const` tables.
  Hundredths is as far as Balatro shows (Lucky Cat grows by 0.25).
* **The decimal count picks the variant**, and `shorthand()` writes it back
  the same way (`x3`, `x3.0`, `x3.50`), so a token never rounds or loses
  information.
* **`shorthand()` is a `Result`, not `Display`.** A `Lucky`, `Custom` or
  other override has no token; so has `edition: Some(None)`. Writing a token
  that dropped them would lie.
* **Ident lookup has no table of its own.** It compares against
  `Joker::catalog()` with underscores dropped and case ignored. Two catalog
  tests pin what this needs: no two idents collide without underscores, and
  no ident has an `X` before a digit or a dot.
* **A hand-written `Deserialize`,** on `CardSpec`'s pattern. An untagged enum
  would buffer the input and lose the map form's YAML tags.
* **Output stays the map**, so traces, `parse` output and page links did not
  change.
* **`--jokers` replaces the file's jokers.** Appending would give two sources
  and an order question.

# Consequences

* The page reads `MultTimes2Dot` and writes the fewest decimals a typed
  ×mult needs; a two-decimal ×mult is no longer rounded to tenths.
* A whole-board one-liner and joker *sets* are follow-ons.
