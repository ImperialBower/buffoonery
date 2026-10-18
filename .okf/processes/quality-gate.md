---
type: process
title: Quality gate (make ayce)
description: What each phase of make ayce runs, what green means, and how the security scan also runs on its own and daily in CI.
tags: [quality, ci, process]
timestamp: '2026-10-18T22:10:40Z'
---

# Phases

`make ayce` is the default Makefile target. It runs, in order:

| Phase | Runs |
|---|---|
| `clean` | `cargo clean` |
| `fmt` | `cargo fmt` |
| `build` | `cargo build` |
| `test` | `cargo test` (unit, integration and doc tests) |
| `lint` | `cargo clippy --all-targets -- -D warnings`, then `cargo deny check` |
| `security-scan` | `./bin/security-scan` (`cargo audit` + `cargo deny check advisories`) |
| `docs` | `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` |

# What green means

Every phase exits 0. Work is not done until `make ayce` is green.

# Security scan

`make security-scan` runs on its own. `bin/security-scan` is the only place
the checks are defined; `.github/workflows/security.yaml` runs it daily, and
`ci.yaml` runs the rest of the gate on every push and pull request.
