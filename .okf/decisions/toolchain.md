---
type: decision
title: Toolchain choices
description: Which tools buffoonery's quality gate uses, at which version, and why.
tags: [toolchain, quality, rust]
timestamp: '2026-10-18T22:10:40Z'
---

# Decision

| Tool | Version | Why |
|---|---|---|
| Rust | 1.99.1 (MSRV, edition 2024) | Latest stable installed locally. Raised from 1.85.0 (cardpack's MSRV) on 2026-10-18; let-chains are now allowed, which newer clippy's `collapsible_if` asks for. |
| rustfmt | ships with the toolchain | One formatting style, checked in CI. |
| clippy | ships with the toolchain; `all` + `pedantic` in `Cargo.toml` | Pedantic catches API-design and idiom issues `clippy::all` misses; a few justified `#[allow]`s cost less than the bugs it prevents. |
| cargo-deny | installed via `cargo install` (0.20.2 at scaffold) | One tool for four supply-chain concerns: licenses, banned or duplicate crates, advisories, and allowed sources. |
| cargo-audit | installed via `cargo install` (0.22.2 at scaffold) | The dedicated RustSec client; runs next to deny's advisory check because their data paths and update cadence differ. |
| cargo-watch | optional, via `make install-tools` | Powers `make watch` only; not part of the gate. |
| miri | not used | Off by default: slow, and it pays off mainly for unsafe-heavy crates. |
| rustdoc | `RUSTDOCFLAGS="-D warnings"` | Keeps documentation debt from piling up silently. |

# Citations

* `Makefile`, `bin/security-scan`, `deny.toml`, `Cargo.toml`
