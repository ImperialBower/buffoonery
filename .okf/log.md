# Update Log

## 2026-10-18
* **Creation**: Scaffolded the buffoonery bundle with `okf_init.py` — see [getting started](getting-started.md).
* Repository scaffolded by dev-playbook (rust): library, edition 2024, MSRV 1.85.0, Apache-2.0. Toolchain and quality-gate concepts added. The engine itself arrives next, copied from cardpack's `funky` module (cardpack `docs/EPIC-06_Funky_Extraction.md`, item 0c).
* **Engine copied in** from cardpack's `funky` module (cardpack `599a621`; last engine change `c762538`), no git history. Paths rewritten to `cardpack::prelude` / `crate::prelude`; behavior unchanged: 477 lib tests pass, `buffoon` scores 11254, `make ayce` green on stable and on 1.85.0. `deny.toml` allows `Unicode-3.0` for `unicode-ident`. Crate-wide `#![allow(dead_code)]` carried over from cardpack (two known sites, listed in `src/lib.rs`). New [engine](architecture/engine.md) concept.
* **Rust bumped 1.85.0 → 1.99.1** (latest stable installed). Updated `.tool-versions`, `Cargo.toml` `rust-version`, `ci.yaml` MSRV matrix entry, and the devcontainer `postCreateCommand`. Clippy 1.99 `collapsible_if` fired 5 times (`src/types/board.rs` ×3, `src/types/hands.rs` ×2); fixed with let-chains via `cargo clippy --fix`. The MSRV is no longer cardpack's 1.85.0.
