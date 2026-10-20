---
type: decision
title: The Joker Calculator lives in web/, built with Vite
description: "EPIC-05 open question 3 and the stack: the calculator page is a web/ folder in this repo (GitHub Pages later), bundled by Vite so the jco output and preview2-shim resolve without an import map; model.js is pure and tested in Node against the real component."
tags: [web, vite, calculator, decision, epic-05]
timestamp: '2026-10-19T23:30:00Z'
resource: web/
---

# Decision

* **Where:** `web/` in this repo, not a separate `-web` repo. The page sits
  next to `component/` and `fixtures/golden/`, so the Story 8 parity test
  reads the fixtures directly and one CI job builds and tests both. A
  separate repo would need the component published and the fixtures copied.
  The page can move once the WIT interface settles.
* **Stack:** Vite (`vite` 8.3.4) and plain JS, no framework. Vite resolves
  the bare `@bytecodealliance/preview2-shim` imports in the `jco` output and
  the core `.wasm` URL; an import map would need the shim copied by hand.
  `base: './'` keeps the build working from a GitHub Pages project path.
* **Split:** `web/src/model.js` is pure (state → `BoardSpec`, trace → rows
  and joker cards) and tested in Node against the real component
  (`web/test/model.test.mjs`). `web/src/main.js` only draws. The slot
  controls are built once and updated, so typing keeps focus.
* **Hands are cards (Story 7).** State holds `played` and `held` as
  `{rank, suit}`, plus `levels` (only changed ones go into `hand_levels`).
  The hand type and base come from the trace's first step (`handSummary`);
  the page never names a hand type itself. At least 1 card is played, since
  the engine scores an empty hand as High Card.
* **Rows are grouped by source** (`groupRows`): one `<tbody>` per hand, card
  or joker, keyed `Base`, `Played 0`, `Held 2`, `Joker 1`. Card editors are
  rebuilt only when a card is added or removed.
* **Boards come from the link (Story 8).** `#board=` + a URL-encoded
  `BoardSpec` (JSON or YAML) opens a board; the component's `parse` turns
  YAML into JSON, and `specToState` turns that into page state. State keeps
  what the page cannot edit (card enhancement/edition, joker edition, joker
  effects that are no override kind, `money`, `discards_left`, `hands_left`,
  `blind`). The page rewrites the link as JSON on every edit, with
  `history.replaceState`, so it never adds history entries.
* **Browser parity** (`web/e2e/parity.spec.mjs`, Playwright, Chromium):
  each fixture opens from a link, and the table must equal rows built from
  the fixture's JSON. Expected rows use `rows()` from `model.js`; the
  formatting itself is covered by the Node tests.
* **No scoring in the page.** Its one sum is `floor(chips × mult)` for the
  running score, the same expression as `Score::score()`.

# Load-bearing details

* `npm test` and `npm run build` transpile
  `target/wasm32-wasip2/component/buffoonery_component.opt.wasm`, so
  `make component` (which makes the `.opt.wasm`) runs first; `make web`
  depends on it.
* Vite warns that the `"use components"` directive in the `jco` output may
  not be kept. It is a marker only; the warning is harmless.
* An override is unconditional (`Chips`, `MultPlus`, `MultTimes`,
  `MultTimes1Dot`); ×mult fractions round to tenths.
