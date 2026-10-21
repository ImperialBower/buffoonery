---
type: decision
title: The CLI is a third workspace crate over the component's adapter
description: "cli/ (buffoonery-cli, unpublished, binary `buffoonery`) links buffoonery-component natively and calls its score/score_with/parse/catalog/version, so the shell and the page give the same JSON; clap without `suggestions` keeps the MIT-only strsim out; the text table is presentation only."
tags: [cli, decision, epic-07]
timestamp: '2026-10-21T00:00:00Z'
resource: cli/
---

# Decision

The command-line tool (EPIC-07) is a new workspace member, `cli/`
(`buffoonery-cli`, `publish = false`), with one binary, `buffoonery`. It
depends on the engine and on `buffoonery-component`, and calls the
component's plain-Rust adapter functions: `score`, `score_with`, `parse`,
`catalog`, `version`.

# Why

* **Not a `[[bin]]` in the engine.** That would pull `clap` and
  `serde_json` into the engine's build and blur the kernel line the
  [component crate](wit-component-crate.md) drew. The engine's dependency
  list is unchanged.
* **Link the component, do not copy it.** The component is already an
  `rlib`; its `wit-bindgen` dependency exists only on `wasm32`. Linking it
  gives the CLI the exact `read_spec`, JSON form and error text the page
  sees. `buffoonery score --json` prints the component's string unchanged.
  The cost: the CLI cannot go to crates.io before the component does.
  `cargo install --git` works.
* **`clap` 4 without default features.** Its `suggestions` feature pulls
  `strsim`, which is MIT only, and `deny.toml` allows MIT only by name.
  With `std, derive, help, usage, error-context`, `cargo deny check`
  passes. `clap` 4.6 needs Rust 1.85, below the 1.99.1 MSRV.

# Rules that keep the two doors equal

* **The table is presentation.** `cli/src/table.rs` shows the page's
  `rows()` set: a step that changed the score, or a joker's effect;
  `--all` shows every step. Every chips, mult and score cell is a
  `ScoreStep` field; the score is `Score::score()`. Numbers are rounded to
  2 decimals as the page does, without thousands separators.
* **Links match `boardHash`.** `cli/src/link.rs` encodes as
  `encodeURIComponent` does. A web Node test reads every fixture's CLI link
  with the page's `readHash`, and a Playwright test opens one in Chromium.
  `link` scores the board first: the page opens only a board it can score.
* **Snapshots.** `fixtures/cli/*.txt` holds each golden fixture's table;
  `BLESS=1 cargo test -p buffoonery-cli --test golden` rewrites them.

# Consequences

* `make web` (and so `make ayce`) builds the CLI first, and CI's
  `component` job and the Pages workflow run `cargo build -p buffoonery-cli`
  before the page's tests.
