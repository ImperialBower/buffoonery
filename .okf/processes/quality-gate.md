---
type: process
title: Quality gate (make ayce)
description: The one-time make setup, what each phase of make ayce runs, what green means, and how the security scan also runs on its own and daily in CI.
tags: [quality, ci, process]
timestamp: '2026-10-20T11:40:00Z'
---

# Setup

`make setup` runs once per machine, before the first `make ayce`. It needs
rustup and Node (versions in `.tool-versions`) and checks for both. Then it:

1. adds the `wasm32-wasip2` target (`rustup target add`);
2. installs `cargo-deny` and `cargo-audit` (`cargo install`; an installed
   version is kept);
3. runs `npm ci` in `component/js/` and `web/`;
4. installs Chromium for Playwright (`npx playwright install chromium`,
   with `--with-deps` on Linux, which asks for sudo).

Every step is safe to run again. The devcontainer's `postCreateCommand`
runs it. CI does the same steps inline, per job.

# Phases

`make ayce` is the default Makefile target. It runs, in order:

| Phase | Runs |
|---|---|
| `clean` | `cargo clean` |
| `fmt` | `cargo fmt` |
| `build` | `cargo build --workspace` |
| `test` | `cargo test --workspace` (unit, integration and doc tests) |
| `lint` | `cargo clippy --workspace --all-targets -- -D warnings`, the same for the component on `wasm32-wasip2`, then `cargo deny check` |
| `security-scan` | `./bin/security-scan` (`cargo audit` + `cargo deny check advisories`) |
| `docs` | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` |
| `e2e` | through `web`, `component` and `cli`: builds the component and the `buffoonery` CLI, runs its JS tests, runs the page's Node tests, builds `web/dist`, then the Playwright browser tests in Chromium |

# What green means

Every phase exits 0. Work is not done until `make ayce` is green.

# Security scan

`make security-scan` runs on its own. `bin/security-scan` is the only place
the checks are defined; `.github/workflows/security.yaml` runs it daily, and
`ci.yaml` runs the rest of the gate on every push and pull request.
