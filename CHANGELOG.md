# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- **Requires cardpack 0.11.2.** cardpack 0.11.2 dropped a stale wasm-only
  `getrandom` 0.3 pin, so `buffoonery` no longer gets two copies of
  `getrandom`. The `getrandom` and `r-efi` skips in `deny.toml` are removed.

### Added

- **The engine**, copied from cardpack's `funky` module at cardpack commit
  `599a621` (last engine change there: `c762538`), with no git history. Paths
  changed (`cardpack::preludes::funky::*` → `buffoonery::prelude::*`); type
  names and behavior did not. 477 lib tests pass unchanged, and the `buffoon`
  example still scores 11254.
- Repository scaffold (dev-playbook, rust): `make ayce` quality gate, CI,
  daily security scan, `.okf/` knowledge bundle.
