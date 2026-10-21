# EPIC-07: Buffoonery from the web and the command line (DOORS)

Oct 20, 2026 · @Christoph · *Written against branch `epic05-custom-effects` at `f54016d`. Numbered in the shared cardpack / buffoonery space: EPIC-05 moved here from cardpack, and EPIC-06 (Funky Extraction) stays in cardpack.*

## Summary

- **Builds:** two front doors to the same engine: a `buffoonery` command-line tool, and the Joker Calculator page published on the web.
- **Why:** the page exists but only runs on a developer's machine (`make dev`), and there is no way to score a board from a shell, a script or CI.
- **Shape:** both doors call the same adapter functions in `component/src/lib.rs`. The CLI links them natively; the page loads them as wasm. A board link carries a board from one door to the other.
- **Proves it:** for every golden fixture, `buffoonery score --json` equals the fixture's `.trace.json`, and a link printed by `buffoonery link` opens the same table in the browser.
- **Status:** Built 2026-10-21, except what needs the live site: Pages must be turned on in the repo settings (task 15), then the live page checked (task 17).

---

## Context

The engine already has one door. The CLI and the hosting are the missing parts.

- **The adapter is plain Rust.** `component/src/lib.rs` has `score` (`:38`), `score_with` with `ScoreOptions` (`:47`, `:100`), `parse` (`:133`), `catalog` (`:156`) and `version` (`:170`). They take and return strings. The WIT bindings are a thin wrapper, built only on `wasm32` (`component/src/lib.rs:177`). The native tests already call these functions directly (`component/tests/golden.rs`), so a native program can use them too.
- **Input is JSON or YAML.** `read_spec` (`component/src/lib.rs:138`) reads JSON when the text starts with `{`, else YAML through `BoardSpec::from_yaml` (`src/types/board_spec.rs:201`).
- **The page works, locally.** `web/` scores every board through the component and does no scoring of its own (`.okf/decisions/web-calculator.md`). `make web` builds `web/dist`; `make dev` serves it (`Makefile`, targets `web` and `dev`). Vite uses `base: './'` (`web/vite.config.js:6`) so the build works from a GitHub Pages project path. **No workflow publishes it.** `.github/workflows/` has only `ci.yaml` and `security.yaml`.
- **Boards already travel by link and by clipboard.** The page reads `#seed=…&effects=[…]&board=…` (`boardHash`, `web/src/model.js:146`; `readHash`, `:171`). "Copy board" writes the spec as JSON (`web/index.html:32`), and "Paste a board" reads YAML or JSON (`web/index.html:38`).
- **The page decides which rows to show.** `rows()` (`web/src/model.js:422`) hides a step unless it changed the score, is a joker's effect, or `showAll` is on.
- **Text forms exist.** `ScoreOp` has `Display` (`src/types/effect.rs:76`), e.g. `+20 Chips`. `Score` has `Display` (`src/types/score.rs:89`), but its form (`Score { 154 chips times 54 mult } = 8316`) is for debugging, not a table cell.
- **There is no binary.** The engine crate is a library. `examples/buffoon.rs` prints one fixed board. Nothing reads a board file from the command line.
- **Supply-chain policy is tight.** `deny.toml:15` allows only `Apache-2.0`, plus named exceptions. A dual `MIT OR Apache-2.0` crate passes. An MIT-only crate needs its own exception.

**This EPIC does not:**

- add a server or HTTP API. "From the web" means the static page, which runs the engine in the browser. No board leaves the user's machine. **Decided 2026-10-20:** static page only; a server was rejected (hosting, cost, boards leave the machine).
- add scoring logic. Neither door computes a number. The engine scores; the adapter translates.
- change the WIT interface (`component/wit/world.wit`). The CLI uses the same five functions the page uses.
- publish the CLI to crates.io. That needs `buffoonery-component` published too (it has `publish = false`, `component/Cargo.toml`). See Open questions.
- build a TUI or an interactive mode. The CLI reads a file and prints.

---

## Status

As of 2026-10-21, branch `epic07-cli` (not yet merged). `make ayce` green.

| Component | Status |
| --- | --- |
| Phase 0 — CLI crate layout and argument parser decided | **Complete**: `cli/`, `clap` 4.6.7 without `suggestions`; `cargo deny check` passes |
| Phase 1 — `buffoonery version`, `parse`, `catalog` | **Complete** |
| Phase 2 — `buffoonery score` (JSON and table, seed, custom effects) | **Complete**: all 19 fixtures, JSON and table snapshots |
| Phase 3 — `buffoonery link` (CLI to web bridge) | **Complete**: every fixture through `readHash`; one link in Chromium |
| Phase 4 — the page on GitHub Pages | Workflow and footer **built**; 🔒 Gated: the maintainer turns on Pages (task 15), then the live check (task 17) |
| Phase 5 — docs, OKF, CHANGELOG, `make` targets | **Complete** |

---

## Goals

- **One engine, two doors.** A board scores to the same `ScoreTrace` from the **CLI** and from the **page**. Neither door has scoring arithmetic.
- **Same adapter, not a copy.** The CLI calls the functions in `component/src/lib.rs`. It does not re-implement `read_spec` or the JSON form.
- **Boards move between doors.** A board copied from the page scores in the CLI. A board file opens in the page through a link the CLI prints.
- **Scriptable.** JSON on stdout, errors on stderr, and exit codes a script can check.
- **Published page.** The calculator is at a public URL, rebuilt from `main` after CI is green.
- **The engine stays pure.** No `clap`, no file I/O and no `std::process` in the engine crate. The CLI is a delivery adapter, as the component is.

## Scope

The rules each door must obey.

1. `buffoonery score <FILE>` reads a `BoardSpec` as YAML or JSON. `-` or no file means stdin.
2. `--json` prints exactly what the component's `score` (or `score-with`) returns. No reformatting, no pretty-print, one trailing newline.
3. Without `--json`, the CLI prints a table. It shows the same rows as the page's `rows()`: a step that changed the score, or any joker effect. `--all` shows every step, as the page's `showAll` does.
4. `--seed N` scores with a seed (a `u64`), as the page's seed field does. `--effects FILE` reads custom effects as the JSON list the page puts in its link (`[{"id":1,"op":"…"}]`).
5. An error prints the adapter's message unchanged, e.g. `jokers[0]: unknown joker ident "NOPE"`, to stderr.
6. Exit codes: `0` success, `1` the board or effects are not valid, `2` bad usage (unknown flag, missing file).
7. `buffoonery link <FILE>` prints a page URL whose fragment `readHash` reads back to the same spec, seed and effects.
8. The page is published only from `main`, only after the `component` CI job is green.

---

## Domain map

| Domain concept | Code construct | Status |
| --- | --- | --- |
| A board to score | `BoardSpec` (`src/types/board_spec.rs:48`) | ✅ |
| How the score happened | `ScoreTrace` (`src/types/trace.rs:77`) | ✅ |
| How to score (seed, custom effects) | `ScoreOptions` (`component/src/lib.rs:47`) | ✅ |
| The joker list | `catalog()` (`component/src/lib.rs:156`) | ✅ |
| The browser door | `web/` + the jco-transpiled component | 🟡 works locally, not published |
| A board as a link | `boardHash` / `readHash` (`web/src/model.js:146`, `:171`) | 🟡 JS only |
| The command-line door | `buffoonery` binary | ❌ |
| A trace as text rows | none in Rust; `rows()` in JS (`web/src/model.js:422`) | ❌ |

---

## Design

### The CLI crate

`cli/` (new workspace member), package `buffoonery-cli`, binary `buffoonery`:

```toml
[package]
name = "buffoonery-cli"
publish = false

[[bin]]
name = "buffoonery"
path = "src/main.rs"

[dependencies]
buffoonery = { path = "..", version = "0.0.1" }
buffoonery-component = { path = "../component", version = "0.1.0" }
clap = { version = "4", default-features = false, features = ["std", "derive", "help", "usage", "error-context"] }
serde_json = "1"
```

```rust
// cli/src/main.rs: parse args, call run, map the result to an exit code.
fn main() -> std::process::ExitCode;

// cli/src/lib.rs: testable without a process.
pub enum Command {
    Score { file: Input, json: bool, all: bool, seed: Option<u64>, effects: Option<Input> },
    Parse { file: Input },
    Catalog { json: bool, find: Option<String> },
    Link { file: Input, seed: Option<u64>, effects: Option<Input>, base: String },
    Version,
}

pub enum Input { Stdin, Path(PathBuf) }

pub enum CliError {
    Usage(String),   // exit 2
    Io(String),      // exit 2: the file named on the line cannot be read
    Board(String),   // exit 1: the adapter's message, unchanged
}

/// Runs one command. Writes to `out`; the caller prints errors.
pub fn run(command: &Command, stdin: &mut dyn Read, out: &mut dyn Write) -> Result<(), CliError>;
```

**Why a third crate, not a `[[bin]]` in the engine.** A binary in the engine crate would pull `clap` and `serde_json` into its build and blur the kernel line that the component crate drew (`.okf/decisions/wit-component-crate.md`). A separate member keeps the engine's dependency list unchanged.

**Why depend on `buffoonery-component`, not copy it.** The component crate is already an `rlib` as well as a `cdylib` (`component/Cargo.toml`, `[lib]`), and its native tests already link it. Linking it gives the CLI the exact `read_spec`, JSON form and error text the page sees. A copy would drift. The cost: the CLI cannot go to crates.io until the component can. `cargo install --git` works, because it builds the whole workspace.

**Why `clap` with trimmed features.** Default `clap` pulls `strsim` (MIT only, for "did you mean" hints), which `deny.toml:15` rejects. Without the `suggestions` feature, `clap` and its tree are `MIT OR Apache-2.0`. Phase 0 checks this with `cargo deny check`. If it still fails, the fallback is a hand-written parser over `std::env::args`: five subcommands and five flags are small.

### The text table

`cli/src/table.rs` (new):

```rust
/// One printed row of a trace, the same rows the page shows.
pub struct Row { pub source: String, pub part: String, pub op: String, pub chips: u64, pub mult: f64, pub score: u64 }

/// The rows the page's `rows()` shows: a step that changed the score, or a
/// joker effect; every step when `all` is true.
pub fn rows(trace: &ScoreTrace, all: bool) -> Vec<Row>;

/// Fixed-width columns, plus a total line: `Pair  32 × 18 = 576`.
pub fn render(rows: &[Row], trace: &ScoreTrace) -> String;
```

```text
$ buffoonery score fixtures/golden/02_pair_additive_jokers.yaml
Source            Part     Op                  Chips   Mult   Score
Pair (level 1)    effect   +10 Chips +2 Mult      10      2      20
A♠                effect   +11 Chips              21      2      42
A♥                effect   +11 Chips              32      2      64
Jolly Joker       effect   +8 Mult                32     10     320
Sly Joker         effect   +50 Chips              82     10     820
Pair                                              82     10     820
```

*The numbers are fixture 02's trace (`fixtures/golden/02_pair_additive_jokers.trace.json`). The column layout is a sketch; task 8's snapshots fix it.*

The table reads the `ScoreTrace` as a Rust value: the CLI parses the adapter's JSON back with `serde_json`, so the table and `--json` come from one call. Op text comes from `ScoreOp`'s `Display` (`src/types/effect.rs:76`). Joker names come from `Joker::name` (the catalog, `src/decks/joker_catalog.rs`).

**Why the table lives in the CLI, not the engine.** It is presentation. The page has its own (`web/src/model.js:422`). Two renderers of the same trace are fine; two scorers are not. The parity rule is on the numbers: every `chips`, `mult` and `score` cell is a field of a `ScoreStep`, never a sum the CLI makes. The one sum is the score column, `floor(chips × mult)`, the same as `Score::score()` and the page.

### The link bridge

`cli/src/link.rs` (new):

```rust
/// The page URL that opens this board: `{base}#seed=…&effects=…&board=…`.
/// The same fragment as `boardHash` in `web/src/model.js`.
pub fn board_link(base: &str, spec_json: &str, seed: Option<u64>, effects_json: Option<&str>) -> String;
```

`spec_json` is `parse`'s output, so a YAML file becomes the JSON the page expects. The default `base` is the Pages URL (`https://imperialbower.github.io/buffoonery/`); `--base http://localhost:5173/` points it at `make dev`.

The other direction needs no code. "Copy board" writes JSON, and `read_spec` reads JSON. So `pbpaste | buffoonery score -` works today, once the CLI exists.

### The page on the web

`.github/workflows/pages.yaml` (new):

```yaml
on:
  push:
    branches: [main]
permissions:
  pages: write
  id-token: write
jobs:
  build:   # the same steps as ci.yaml's `component` job, then upload web/dist
  deploy:  # actions/deploy-pages
```

**Why GitHub Pages.** The page is static: HTML, JS and one `.wasm`. Pages is free, already planned (`.okf/decisions/web-calculator.md`, "GitHub Pages later"), and `base: './'` (`web/vite.config.js:6`) is set for it. A server would add hosting, cost and a place where boards leave the machine, for no gain.

**Why rebuild, not reuse CI's build.** The `component` job (`.github/workflows/ci.yaml:79`) does not keep `web/dist`. The Pages workflow builds it again and runs the Playwright parity test before upload, so a broken page never deploys.

The page footer already names the engine version (`web/src/main.js:406`). Phase 4 adds one line under it: how to install the CLI.

---

## Work Items

### Phase 0 — Prerequisites

- [x] **0a.** Add `cli` to `[workspace] members` (`Cargo.toml:4`). Create `cli/Cargo.toml` with `rust-version = "1.99.1"`, the same `[lints.clippy]` block as `component/Cargo.toml`, and `publish = false`.
- [x] **0b.** Add `clap` with the trimmed features. Run `cargo deny check`. If a license fails, take the hand-written parser instead and say so in the CHANGELOG.
- [x] **0c.** Check the version-consistency rule (`CLAUDE.md`): `.tool-versions`, `rust-version`, the CI MSRV entry (`.github/workflows/ci.yaml:23`) and the devcontainer agree. The new crate must not raise the MSRV.
- [x] **0d.** `cargo build --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` are green.

### Phase 1 — `version`, `parse`, `catalog`

- [x] **1.** `cli/src/lib.rs`: `Command`, `Input`, `CliError`, `run`. `cli/src/main.rs`: `clap` derive, then `run`, then the exit code.
- [x] **2.** `version` prints `buffoonery_component::version()`. Test `run_version_prints_engine_version`.
- [x] **3.** `parse <FILE>` prints `parse()` output. Tests `run_parse_yaml_prints_json`, `run_parse_reads_stdin`.
- [x] **4.** `catalog` prints a table (ident, name, rarity, cost); `--json` prints `catalog()`; `--find jolly` filters by name, ignoring case. Tests `run_catalog_json_equals_component`, `run_catalog_find_filters`.
- [x] **5.** Errors: a missing file is exit 2, a bad board is exit 1 with the adapter's text. Tests `run_missing_file_is_io_error`, `run_bad_joker_is_board_error`.

### Phase 2 — `score`

- [x] **6.** `score <FILE> --json`: calls `score`, or `score_with` when `--seed` or `--effects` is given. Test `cli/tests/golden.rs`: for every `fixtures/golden/*.yaml`, the output equals the `.trace.json` (reuse the fixture walk from `component/tests/golden.rs:18`).
- [x] **7.** `cli/src/table.rs`: `rows` and `render`. Tests `rows_hides_noop_card_steps`, `rows_keeps_unfired_joker_effects`, `rows_all_keeps_every_step`, `render_total_line_matches_score`.
- [x] **8.** Table snapshots: `fixtures/cli/*.txt`, one per golden fixture, rewritten with `BLESS=1` as the golden traces are. Test `table_matches_snapshot`.
- [x] **9.** `--seed` and `--effects`: tests `score_seed_matches_score_with` (same JSON as the component with the same seed) and `score_effects_reads_page_format` (the effects JSON the page writes in its link).
- [x] **10.** `cli/tests/process.rs`: run the built binary (`env!("CARGO_BIN_EXE_buffoonery")`) for exit codes 0, 1 and 2 and for stdin.

### Phase 3 — `link`

- [x] **11.** `cli/src/link.rs`: `board_link`. Percent-encode as JavaScript's `encodeURIComponent` does. Unit tests `board_link_board_only`, `board_link_with_seed_and_effects`, `board_link_encodes_like_encode_uri_component`.
- [x] **12.** Cross-door test in `web/test/model.test.mjs`: for every golden fixture, run `target/debug/buffoonery link --base x <fixture>`, feed the fragment to `readHash`, and check the spec equals `parse()` of the fixture. `make web` builds the CLI first.
- [x] **13.** Browser test in `web/e2e/parity.spec.mjs`: open one CLI-made link and check the table, as the fixture tests do.

### Phase 4 — the page on GitHub Pages

- [x] **14.** `.github/workflows/pages.yaml`: on push to `main`, build the component, run `npm test` and the Playwright tests, then upload and deploy `web/dist`.
- [ ] **15.** Enable Pages in the repo settings, source "GitHub Actions". This is a settings change; the maintainer does it.
- [x] **16.** Footer line in `web/src/main.js:406`: "Also on the command line: `cargo install --git https://github.com/ImperialBower/buffoonery buffoonery-cli`".
- [ ] **17.** Check the live URL in a browser: the opening board scores, a `#board=` link opens, no sideways scroll at 390 px.

### Phase 5 — Docs and the gate

- [x] **18.** `README.md`: a "Command line" section (install, the five commands, one example) and the live page URL.
- [x] **19.** `Makefile`: `cli` target (`cargo build -p buffoonery-cli`); `web` depends on it for task 12. `make ayce` already runs the CLI tests through `cargo test --workspace`.
- [x] **20.** `.okf/decisions/cli.md` (`type: decision`): why a third crate, why it links the component crate, the `clap` license check, the table parity rule. Append to `.okf/log.md`. Add to `.okf/index.md`.
- [x] **21.** `CHANGELOG.md` entry. `make ayce` green.

---

## Test Plan

- `cli/tests/golden.rs::score_json_equals_golden` — every fixture, `--json` output byte-equals its `.trace.json`. Pins "same engine, same numbers".
- `cli/tests/golden.rs::table_matches_snapshot` — every fixture's table equals `fixtures/cli/NN.txt`. Pins the text form; a scoring change fails here and in the trace test.
- `table::rows_*` — the row filter equals the page's rule (changed the score, or a joker effect, or `--all`).
- `score_seed_matches_score_with` — the seed path gives the component's seeded JSON.
- `score_effects_reads_page_format` — custom effects in the page's link format score.
- `cli/tests/process.rs` — exit codes 0, 1, 2; stdin with `-`; error text on stderr, nothing on stdout.
- `board_link_*` — the fragment form matches `boardHash`.
- `web/test/model.test.mjs` "CLI links open" — `readHash(cli link)` equals the fixture spec, for every fixture.
- `web/e2e/parity.spec.mjs` "a CLI link opens" — one CLI link, cell by cell in Chromium.
- Pages workflow — the Playwright run gates the deploy.

## Key Files

| File | Role |
| --- | --- |
| `cli/Cargo.toml` | new crate `buffoonery-cli`, binary `buffoonery` |
| `cli/src/main.rs` | argument parsing and exit codes |
| `cli/src/lib.rs` | `Command`, `run`, `CliError` |
| `cli/src/table.rs` | trace → text rows |
| `cli/src/link.rs` | board → page URL |
| `cli/tests/golden.rs`, `cli/tests/process.rs` | parity and process tests |
| `fixtures/cli/` | table snapshots |
| `.github/workflows/pages.yaml` | builds, tests and publishes the page |
| `web/src/main.js` | footer line |
| `web/test/model.test.mjs`, `web/e2e/parity.spec.mjs` | cross-door tests |
| `Cargo.toml`, `Makefile`, `README.md`, `.okf/` | workspace, targets, docs |

## Reuse (do NOT recreate)

- `component/src/lib.rs:38,100,133,156,170` — `score`, `score_with`, `parse`, `catalog`, `version`. The CLI calls these. It does not read YAML or write JSON on its own.
- `component/src/lib.rs:47,61` — `ScoreOptions` and `CustomEffect`, built from `--seed` and `--effects`.
- `src/types/effect.rs:76` — `ScoreOp`'s `Display` for the op column.
- `src/decks/joker_catalog.rs` — `Joker::name` for joker rows.
- `component/tests/golden.rs:14-31` — the fixture walk and expected-trace reader.
- `web/src/model.js:146,171` — `boardHash` / `readHash`, the link form the CLI must match.
- `web/src/model.js:422` — `rows()`, the row filter the table must match.
- `fixtures/golden/` — the 19 boards both doors are checked against.

## Compatibility

- **Preserves** the engine's public API and dependency list, the WIT interface, the component size (the CLI is not in the wasm build), and every page behavior.
- **Adds** a workspace member `buffoonery-cli` (unpublished), a binary `buffoonery`, a Pages workflow, and table snapshots.
- **Breaks** nothing. A downstream user of the `buffoonery` crate sees no change.

## Dependencies

- **Built on:** EPIC-05 (the trace, `BoardSpec`, the component, the page, the golden fixtures, links, seeds and custom effects).
- **Related:** cardpack EPIC-06 (Funky Extraction), which made this repo the solver foundation.
- **Blocks:** nothing yet. A solver that searches boards would use the CLI's `--json` for batch runs.

## Open questions

1. **CLI on crates.io?** It needs `buffoonery-component` published (or its adapter functions moved to a published crate). Recommend: wait. Install from git until a user asks for `cargo install buffoonery-cli`.
2. **Release binaries?** A GitHub Release with built binaries for macOS, Linux and Windows would skip the Rust toolchain for users. Recommend: a follow-on EPIC, after the CLI's commands settle.

## Verification

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check
cargo run -p buffoonery-cli -- version
cargo run -p buffoonery-cli -- score fixtures/golden/02_pair_additive_jokers.yaml
cargo run -p buffoonery-cli -- score --json fixtures/golden/02_pair_additive_jokers.yaml \
  | diff - fixtures/golden/02_pair_additive_jokers.trace.json
cargo run -p buffoonery-cli -- link fixtures/golden/02_pair_additive_jokers.yaml
make ayce
```

Exit criteria:

1. Every golden fixture gives the same JSON from the CLI as from the component, and the same table as its snapshot.
2. A CLI link opens in the page and shows the same rows; a board copied from the page scores in the CLI.
3. The page is live at the Pages URL, deployed only after the parity tests pass.
4. The engine crate's dependency list and the component's size are unchanged.
5. `make ayce` is green.

## Implementation notes

Written 2026-10-21, while Phase 4 waits on the live site. Design vs actual so far:

1. **`--json` is checked as JSON, not byte for byte, against the fixture files.** `--json` prints the component's string unchanged (compact JSON), but the `.trace.json` files are pretty-printed. `score_json_equals_golden` compares parsed values; `no_file_reads_stdin` checks the bytes against `buffoonery_component::score`. The Verification block's `diff` therefore differs in white space only.
2. **The table follows the page's row text,** not the sketch: columns `#`, Source, Detail, Effect, Chips, Mult, Score; sources read `A♠ · played 1` and `Jolly Joker · slot 1`; the total line is `Pair  82 × 10 = 820`. Numbers round to 2 decimals, as the page does (`10.125` → `10.13`), without thousands separators.
3. **`link` scores the board before it prints,** because the page opens only a board it can score; a bad board is exit 1. `--effects` takes each op as an object (the page's link form) or as JSON text; the link always writes objects.
4. **Board and effects cannot both come from stdin** (exit 2).
5. **The footer line is static HTML** (`web/index.html`, `#cli-note`), not `main.js`. A Playwright test checks the page does not scroll sideways at 320 px: without `overflow-wrap` on the install command it scrolled 31 px.
6. **The binary has `doc = false`:** it is named `buffoonery`, like the engine library, and the two docs collided in `target/doc/buffoonery`.
7. **CI and Pages build the CLI** before the page's tests, because a web Node test and a Playwright test run `target/debug/buffoonery link`.

Test counts at `make ayce`: 38 CLI tests (29 unit, 3 golden, 6 process); the page's Node tests 27 (was 26); Playwright 28 (was 26).
