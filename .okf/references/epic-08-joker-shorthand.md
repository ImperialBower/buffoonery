---
type: reference
title: EPIC-08 — Joker shorthand
description: "Complete (closed 2026-10-22): a one-token text form for a joker (`BARON+FOIL`, `LUCKYCATx3.25`) parsed in the engine, accepted as a string in `BoardSpec` YAML/JSON, and on the CLI as `jokers` and `--jokers`; adds `MPip::MultTimes2Dot` so ×3.25 is exact."
tags: [epic, shorthand, dsl, joker, cli, complete]
timestamp: '2026-10-22T00:00:00Z'
resource: docs/EPIC-08_Joker_Shorthand.md
---

# What it covers

One joker, one token, the way `66+` is a poker range. `ident { modifier }`:
`+FOIL` / `+HOLO` / `+POLY` / `+NEG` for the edition, `+50c` / `+4m` / `x3.25`
for the override, `+OFF` for debuffed. A line of tokens is a joker row.

# Key rules (see the EPIC)

* **Jokers only** (decided 2026-10-21). A whole-board one-liner and joker
  *sets* are follow-ons.
* **In the engine, no dependencies**, beside `BuffoonCard::from_str("AS")`.
  The CLI and the component call it.
* **The decimal count picks the variant:** `x3` → `MultTimes`, `x3.5` →
  `MultTimes1Dot`, `x3.25` → new `MultTimes2Dot` (hundredths). `MultTimes1Dot`
  is not changed: its YAML shape is a contract.
* **`shorthand()` is a `Result`, never rounds.** A `Custom` or `Lucky`
  override has no token and says so.
* **Serialization stays the map.** Traces, `parse` output and links are
  byte-for-byte unchanged.
* **The page writes hundredths too** for a two-decimal ×mult, so a typed
  ×3.25 is not rounded to ×3.3. (The EPIC expected fixture 20 to break the
  page's round trip; it did not, because the page passes an `MPip` it does
  not know through unchanged.)
* **Complete, closed 2026-10-22** at `e16be87`, all six phases; see the [decision](/decisions/joker-shorthand.md)
  and the EPIC's Implementation notes.
