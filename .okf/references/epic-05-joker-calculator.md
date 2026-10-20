---
type: reference
title: EPIC-05 — Joker Calculator integration
description: "The active EPIC: make the Balatro Joker Calculator a thin display over this engine — fractional Score (S1), scoring split out (S2), a ScoreTrace, a BoardSpec in Rust or YAML, a joker catalog, and a WIT component."
tags: [epic, scoring, trace, wit, yaml, active]
timestamp: '2026-10-19T12:00:00Z'
resource: docs/EPIC-05_Joker_Calculator.md
---

# What it covers

The calculator stops doing its own math. Every number it shows comes from
`BuffoonBoard`, through a step-by-step `ScoreTrace`, behind a WIT component.
Written in cardpack on 2026-10-18 (Milestone 0 ran there) and moved here on
2026-10-19 with its design bundle (`docs/design/balatro-joker-calculator/`).

# Key rules (see the EPIC)

* **One trace step per `ScoreOp::apply`**, not per card: a played card applies
  up to 3 ops per pass, a joker 2.
* **The joker catalog is generated from `Joker::ALL_JOKERS`**, never kept by hand.
* **YAML sets data, Rust adds behavior.**
* `Score` keeps fractional mult ([decision](/decisions/fractional-score.md)).
* **`fixtures/golden/` is the shared contract** (Story 5): the Rust tests, the
  WIT component and the browser parity test read the same YAML/JSON pairs.
  `BLESS=1 cargo test --test golden` rewrites them; check every changed
  number by hand, because a blessed file records the engine, not Balatro.

# Status

The live status is the `## Status` table in `docs/EPIC-05_Joker_Calculator.md`;
this page does not track it.

This concept is a pointer, not a copy — the EPIC is authoritative.
