---
type: decision
title: BoardSpec YAML — feature, crate, and the hand-written CardSpec parser
description: "YAML is the default-on `yaml` feature using serde_norway 0.9.42 (cardpack's YAML crate); CardSpec deserializes by hand because #[serde(untagged)] cannot carry YAML tags; MPip's YAML shape is pinned."
tags: [yaml, serde, board-spec, decision, epic-05]
timestamp: '2026-10-19T00:00:00Z'
resource: src/types/board_spec.rs
---

# Decision

`BoardSpec` (`src/types/board_spec.rs`, EPIC-05 Story 3) is plain serde
types. YAML loading (`BoardSpec::from_yaml` / `to_yaml`) is the `yaml`
feature, on by default.

| Choice | Why |
|---|---|
| `serde_norway` 0.9.42, optional | The YAML crate and version cardpack's own `yaml` feature uses, so a graph with both never has two YAML stacks. |
| `yaml` is a feature, default on | The domain-kernel rule: formats are adapters. `--no-default-features` builds and tests with no YAML code (checked 2026-10-19: 535 tests). |
| `deny.toml` allows MIT for `unsafe-libyaml-norway` only | It is the YAML parser under `serde_norway` and is MIT only. Allowing MIT by crate name keeps every other new dependency under review. |
| `CardSpec` has a hand-written `Deserialize` | `#[serde(untagged)]` buffers its input, and the buffer cannot carry a YAML tag: `enhancement: !MultTimes1Dot 15` failed with "untagged and internally tagged enums do not support enum input". The visitor takes a string as `CardSpec::Index` and hands a map straight to the real deserializer. **Do not "simplify" it back to `untagged`.** |
| `MPip` and `Edition` YAML shapes are pinned | `yaml__mpip_and_edition_format`: `!MultPlus 4`, `!Lucky [5, 15]` (as a block list), `Blank`, `Polychrome`. Once people write boards as YAML, renaming an `MPip` variant breaks their files (EPIC-05 open question 11). |

# Citations

* `src/types/board_spec.rs`, `Cargo.toml` (`[features]`), `deny.toml`.
* `docs/EPIC-05_Joker_Calculator.md`, Story 3.
