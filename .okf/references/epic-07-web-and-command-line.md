---
type: reference
title: EPIC-07 — Buffoonery from the web and the command line
description: "Active EPIC: a `buffoonery` CLI (new crate `buffoonery-cli`) that links the component's adapter functions natively, and the Joker Calculator page published on GitHub Pages; a board link carries a board between the two."
tags: [epic, cli, web, github-pages, active]
timestamp: '2026-10-21T00:00:00Z'
resource: docs/EPIC-07_Web_And_Command_Line.md
---

# What it covers

Two front doors to one engine. The CLI calls `score`, `score_with`, `parse`,
`catalog` and `version` from `component/src/lib.rs`, the same functions the
page reaches through wasm. The page is published from `main` after the
parity tests pass.

# Key rules (see the EPIC)

* **Same adapter, not a copy.** `buffoonery score --json` prints exactly the
  component's JSON; every golden fixture is checked.
* **The table is presentation.** It shows the page's `rows()` set; its
  numbers are `ScoreStep` fields, never CLI sums.
* **`clap` without `suggestions`.** Default `clap` pulls `strsim` (MIT only),
  which `deny.toml` rejects.
* **Static page, no server** (decided 2026-10-20). The engine runs in the
  browser; no HTTP API.
* **Built 2026-10-21:** phases 0–3 and 5, and Phase 4's workflow and footer; see the [CLI decision](/decisions/cli.md). Pages must still be turned on in the repo settings.
* **Numbered 07** because 05 and 06 share cardpack's EPIC number space.
