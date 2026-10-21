# EPIC-08: Joker shorthand (TOKENS)

Oct 21, 2026 · @Christoph · *Written against branch `epic05-close` at `7383070`. Numbered in the shared cardpack / buffoonery space, after EPIC-07.*

## Summary

- **Builds:** a one-token text form for a joker, the way `66+` is a poker range: `BARON+FOIL` is a Foil Baron, `LUCKYCATx3.25` is a Lucky Cat whose effect is ×3.25 mult. A line of tokens is a joker row.
- **Why:** today a joker with an edition or an override is a four-line YAML map. Cards already have a shorthand (`AS`); jokers do not. A solver, a script, a chat message or a CLI flag wants one.
- **Shape:** `JokerSpec` learns `FromStr` and a canonical `shorthand()`, in a new engine module with no dependencies. `BoardSpec` accepts a joker as a bare string, as it accepts a card. The CLI gets a `jokers` command and a `--jokers` flag. One new `MPip` variant, `MultTimes2Dot`, so ×3.25 is exact.
- **Proves it:** every joker in the catalog round-trips through its token; a golden fixture written with tokens scores to a blessed trace; `buffoonery score --jokers "BARON+FOIL LUCKYCATx3.25"` prints the same JSON as the YAML map form.
- **Status:** Planned. Nothing landed yet.

---

## Context

The joker already has a data form. The shorthand is a second spelling of it, not a new model.

- **A joker in a board is four fields.** `JokerSpec` (`src/types/board_spec.rs:135`) is `ident`, an optional `enhancement` override, an optional `edition`, and `debuffed`. `to_card` (`src/types/board_spec.rs:239`) looks the ident up in the catalog and lays the overrides on the card. The shorthand maps onto exactly these four fields.
- **Cards have a shorthand already.** `BuffoonCard::from_str("AS")` (`src/types/buffoon_card.rs:282`) reads a card index. `CardSpec` (`src/types/board_spec.rs:78`) accepts a bare string or a map in YAML and JSON, through a hand-written `Deserialize` (`src/types/board_spec.rs:102`). The joker string form copies this pattern.
- **Idents are stable and unique.** `Joker::by_ident` (`src/decks/joker_catalog.rs:166`) is an exact-case lookup over 112 idents taken from the const names. Checked on 2026-10-21: no two idents collide once underscores are dropped (`LUCKY_CAT` → `LUCKYCAT`), and no ident contains an `X` followed by a digit or a dot, so `LUCKYCATx3.25` splits without ambiguity. A test pins both facts.
- **Editions are five words.** `Edition` (`src/types/edition.rs:17`): `None`, `Foil`, `Holographic`, `Polychrome`, `Negative`.
- **The page's overrides are three kinds, plus Custom.** `overrideEnhancement` (`web/src/model.js:190`) writes `Chips(n)`, `MultPlus(n)`, `MultTimes(n)` or `MultTimes1Dot(tenths)`. `overrideFromEnhancement` (`web/src/model.js:270`) reads the same four back. A custom effect writes `{Custom: id}` onto the joker (`web/src/model.js:243`). The shorthand's overrides are the three numeric kinds; `Custom` has no token.
- **The page passes through what it cannot edit.** `slotFromSpec` (`web/src/model.js:296`) keeps an unknown `enhancement` on the slot and `buildSpec` (`:245`) writes it back unchanged. `web/test/model.test.mjs:171` checks every fixture survives this round trip, and the page's own write side rounds a typed ×mult to tenths (`:203`; `model.test.mjs:93` pins ×1.25 → `MultTimes1Dot(13)`).
- **×3.25 does not fit today.** `MPip::MultTimes1Dot(usize)` (`src/types/mpip.rs:111`) holds tenths: `MultTimes1Dot(15)` is ×1.5 (`MPip::STEEL`, `:412`). The page rounds ×3.25 to ×3.3. The engine reads tenths in two places: a held card (`src/types/board/scoring.rs:344`) and a joker (`:656`). A third site, `full_deck_steel_count` (`:597`), matches the variant to count Steel cards for Steel Joker and never reads the value. `Joker::scores_hand` (`src/decks/joker.rs:2475`) classifies every variant and will not compile until a new one is listed.
- **Lucky Cat is `Blank`.** It gains ×0.25 per Lucky trigger, which needs mutating scoring (`Joker::BLANK_WITH_REASON`, `src/decks/joker.rs:327`). `LUCKYCATx3.25` is how to say "a Lucky Cat that has grown to ×3.25" without that work.
- **The CLI reads boards from files.** `Command::Score` and `Command::Link` (`cli/src/lib.rs:27`, `:39`) take a file or stdin. There is no way to name a joker on the command line.
- **The component reads the spec through `BoardSpec`.** `read_spec` (`component/src/lib.rs:138`, private) sniffs JSON or YAML and deserializes. The CLI never builds a `BoardSpec` today; `read_board` in `cli/src/lib.rs` returns the raw text. Anything `BoardSpec` accepts, the component and so the page's "Paste a board" accept, with no page change.
- **The YAML shapes of `MPip` and `Edition` are pinned.** `yaml__mpip_and_edition_format` (`src/types/board_spec.rs:689`). Adding a variant is allowed; renaming one is not (`.okf/decisions/board-spec-yaml.md`).
- **The golden set is capped at 20.** `tests/golden.rs:63` accepts 10 to 20 fixtures. There are 19.

**This EPIC does not:**

- write a whole board as one line (played cards, held cards, jokers). That is a follow-on EPIC that reuses this token grammar for the joker part. Decided 2026-10-21 with the author: jokers only.
- express **sets** of jokers (`BARON+FOIL|HOLO`, "any ×mult joker"). A poker range names many hands; this token names one joker. A solver that enumerates jokers can build its own sets from tokens.
- make the shorthand the page's input. The page keeps its pickers. Pasting a YAML board that uses tokens works through the component for free; the page does not write tokens.
- change what any existing fixture, JSON `parse` output, or page link contains. `JokerSpec` still serializes as a map.
- add scoring logic. The token is a spelling of `JokerSpec`; the engine scores the spec as it does today. The one engine change, `MultTimes2Dot`, is a precision fix, not a rule.
- change `MultTimes1Dot`. Its YAML shape is a contract.

---

## Status

As of 2026-10-21, branch `epic05-close` at `7383070`.

| Component | Status |
| --- | --- |
| Phase 0 — `MPip::MultTimes2Dot` | Planned |
| Phase 1 — `joker_shorthand`: parse one token, parse a line, `shorthand()` | Planned |
| Phase 2 — `BoardSpec` accepts a joker as a string; fixture 20 | Planned |
| Phase 3 — CLI `jokers` command and `--jokers` flag | Planned |
| Phase 4 — the page reads and writes `MultTimes2Dot` | Planned |
| Phase 5 — docs, OKF, CHANGELOG | Planned |

---

## Goals

- **One joker, one token.** Any `JokerSpec` with a numeric override, or none, has a token, and the token reads back to the same spec. A custom effect has no token; the error says so.
- **Reads like Balatro.** Ident, then `+FOIL`, then `x3.25`. A player who knows the game can read and write a token with no manual.
- **Strict.** An unknown ident, an unknown modifier, two editions, or ×3.255 is an error that names the token and the problem. No guessing, no rounding.
- **Pure.** The parser lives in the engine with no new dependencies, as the card parser does. The CLI and the component call it; neither re-implements it.
- **Same board, three spellings.** A joker written as a token in YAML, as a token on the command line, or as a map, scores to the same trace.
- **Exact fractions.** ×3.25 scores as ×3.25.

## Scope

The rules the token must obey. Grammar first, then the rules.

```text
line     = token { space token }                ; one joker row, left to right
token    = ident { modifier }
ident    = letter { letter | digit | "_" }      ; any case; "_" is ignored
modifier = "+" edition
         | "+" integer "c"                      ; chips override
         | "+" integer "m"                      ; +mult override
         | "x" number                           ; ×mult override
         | "+OFF"                               ; debuffed
edition  = "FOIL" | "HOLO" | "HOLOGRAPHIC" | "POLY" | "POLYCHROME" | "NEG" | "NEGATIVE"
number   = integer [ "." digit [ digit ] ]      ; at most two decimals
integer  = digit { digit }
```

1. **The ident** is the catalog ident with underscores optional and case ignored: `LUCKYCAT`, `lucky_cat` and `LUCKY_CAT` are all Lucky Cat. The ident ends at the first `+`, or at an `x`/`X` that is followed by a digit or a dot.
2. **Modifiers** come in any order. Each kind appears at most once: one edition, one override, one `+OFF`. A second one is an error.
3. **The override** is one of `+Nc` (`MPip::Chips(N)`), `+Nm` (`MPip::MultPlus(N)`), `xN` (`MPip::MultTimes(N)` when N is whole, `MPip::MultTimes1Dot(N×10)` with one decimal, `MPip::MultTimes2Dot(N×100)` with two). More than two decimals, a negative number, or a decimal on `c` or `m` is an error.
4. **Editions** have a short and a long spelling on input. `+FOIL` has no long form.
5. **The canonical form** (`JokerSpec::shorthand()`) is: ident with no underscores, then edition (short spelling), then override, then `+OFF`. The decimal count follows the variant, so the token keeps it: `MultTimes(3)` is `x3`, `MultTimes1Dot(30)` is `x3.0`, `MultTimes2Dot(350)` is `x3.50`. Nothing is dropped or normalized. So `parse(shorthand(s)) == s` for every spec that has a token, and `shorthand(parse(t)) == t` for every canonical token.
6. **A `JokerSpec` whose override is any other `MPip`** (`{Custom: id}` from the page, or `!Lucky [5, 15]` written by hand in YAML) has no token. `shorthand()` returns an error naming the variant; it never rounds or drops it.
7. **A line** is tokens separated by any run of spaces, tabs or commas; `BARON,,JOKER` and `BARON , JOKER` are two tokens. Position counts from 0. An error from the third token says `jokers[2]`. An empty line is an empty row.
8. **In YAML and JSON**, a joker is a string (a token) or a map (the fields). The map form is unchanged. Serialization stays the map.
9. **Errors** are typed. One token fails with a `ShorthandError` that names the token and the problem: `"LUCKYCAR": unknown joker`, `"BARON+FOIL+HOLO": two editions`. A line fails with a `LineError` that adds the position: `jokers[1]: "LUCKYCAR": unknown joker`. In YAML or JSON the token error reaches the user through serde, which appends the line and column; no index is invented.

---

## Domain map

| Domain concept | Code construct | Status |
| --- | --- | --- |
| A joker with its state (edition, grown effect, debuffed) | `JokerSpec` (`src/types/board_spec.rs:135`) | ✅ |
| The joker's name | `Joker::by_ident` (`src/decks/joker_catalog.rs:166`) | ✅ exact case only |
| An edition | `Edition` (`src/types/edition.rs:17`) | ✅ |
| A grown ×mult | `MPip::MultTimes`, `MultTimes1Dot` (`src/types/mpip.rs:108`, `:111`) | 🟡 tenths only |
| A card as text | `BuffoonCard::from_str` (`src/types/buffoon_card.rs:282`) | ✅ |
| A joker as text | none | ❌ |
| A joker row as text | none | ❌ |
| A joker on the command line | none (`cli/src/lib.rs:19`) | ❌ |

---

## Design

### `MPip::MultTimes2Dot`

`src/types/mpip.rs:111` (next to `MultTimes1Dot`):

```rust
    /// ×mult in hundredths: `MultTimes2Dot(325)` is ×3.25. The shorthand
    /// writes this when a factor has two decimals; `MultTimes1Dot` keeps
    /// tenths, and its YAML shape is pinned.
    MultTimes2Dot(usize),
```

Three engine sites read tenths; each gains a hundredths arm:

```rust
// src/types/board/scoring.rs:344 (builtin_held_op) and :656 (joker factor)
MPip::MultTimes1Dot(n) => n as f64 / 10.0,
MPip::MultTimes2Dot(n) => n as f64 / 100.0,
```

`full_deck_steel_count` (`src/types/board/scoring.rs:597`) stays as it is: a `MultTimes2Dot` card is not a Steel card. `Joker::scores_hand` (`src/decks/joker.rs:2517`) lists it beside `MultTimes1Dot`. `Display` (`src/types/mpip.rs:430`) prints `MultTimes2Dot(325)`. The YAML form is `!MultTimes2Dot 325`, pinned in `yaml__mpip_and_edition_format`.

**Why a new variant, not a change.** `MultTimes1Dot`'s YAML shape is a contract (`.okf/decisions/board-spec-yaml.md`); boards on disk say `!MultTimes1Dot 15`. Changing it to hundredths would silently make them ×0.15. **Why not `f64`.** `MPip` is `Eq`/`Hash` data; a float breaks that and the `const` tables. Hundredths is as far as Balatro's own display goes (Lucky Cat grows by 0.25).

### `joker_shorthand`

`src/types/joker_shorthand.rs` (new), listed in `src/types/mod.rs` and re-exported from the prelude:

```rust
//! One token per joker: `BARON+FOIL`, `LUCKYCATx3.25`, `JOKER+50c+OFF`.

use crate::types::board_spec::JokerSpec;
use std::str::FromStr;

/// Why one token could not become a `JokerSpec`. No position: a token
/// does not know where it sits. Every variant but `NoShorthand` carries
/// the token, and `Display` quotes it first: `"LUCKYCAR": unknown joker`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShorthandError {
    /// The token was empty.
    Empty,
    /// No catalog ident matches, with underscores dropped and case ignored.
    UnknownJoker { token: String, ident: String },
    /// A modifier that is none of the grammar's forms.
    UnknownModifier { token: String, modifier: String },
    /// A second edition, override or `+OFF` on one token.
    Duplicate { token: String, what: &'static str },
    /// A number that is not an integer where one is needed, is negative, or
    /// has more than two decimals.
    BadNumber { token: String, number: String },
    /// `shorthand()` on a spec whose override has no token form.
    NoShorthand { ident: String, enhancement: String },
}

impl std::fmt::Display for ShorthandError { /* `"LUCKYCAR": unknown joker` */ }
impl std::error::Error for ShorthandError {}

/// A token error with its place in a line: `jokers[2]: "LUCKYCAR": unknown joker`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineError {
    pub position: usize,
    pub error: ShorthandError,
}

impl std::fmt::Display for LineError { /* "jokers[{position}]: {error}" */ }
impl std::error::Error for LineError {}

/// Reads one token.
///
/// # Errors
/// Any [`ShorthandError`] but `NoShorthand`.
pub fn parse_joker(token: &str) -> Result<JokerSpec, ShorthandError>;

/// Reads a line of tokens, left to right. Any run of spaces, tabs and
/// commas separates; an empty line is an empty row.
///
/// # Errors
/// The first token that fails, with its position.
pub fn parse_jokers(line: &str) -> Result<Vec<JokerSpec>, LineError>;

impl FromStr for JokerSpec {
    type Err = ShorthandError;
    fn from_str(token: &str) -> Result<Self, Self::Err> { parse_joker(token) }
}

impl JokerSpec {
    /// The canonical token: `LUCKYCATx3.25`, `BARON+FOIL`, `JOKER`.
    ///
    /// # Errors
    /// [`ShorthandError::NoShorthand`] if the override is not `Chips`,
    /// `MultPlus`, `MultTimes`, `MultTimes1Dot` or `MultTimes2Dot`.
    pub fn shorthand(&self) -> Result<String, ShorthandError>;
}

/// Writes a line of tokens, one space between.
///
/// # Errors
/// The first spec with no token form, with its position.
pub fn shorthand_line(jokers: &[JokerSpec]) -> Result<String, LineError>;
```

Ident lookup is one pass over `Joker::catalog()`: compare `ident.replace('_', "")` to the token's ident, both upper-cased. No new table. The catalog test `idents_are_unique_without_underscores` pins the fact the lookup needs.

**Why in the engine.** The card parser is in the engine (`src/types/buffoon_card.rs:282`) and the shorthand is the same kind of thing: a spelling of a domain value, with no I/O and no format crate. The CLI and the component call it, so there is one parser. **Why `Result` for `shorthand()`, not `Display`.** A `Display` that silently drops a `Lucky` override would lie. The token form covers what the page writes; anything else must say so. **Why `+OFF`.** It is short, it reads as "switched off", and it cannot be confused with an edition or a number. **Why `x`, not `*`.** `*` is a shell glob; `x` is what Balatro prints.

### `BoardSpec` string jokers

`src/types/board_spec.rs:135`. `JokerSpec` loses `#[derive(Deserialize)]` and gains a hand-written one on the `CardSpec` pattern (`:102`):

```rust
impl<'de> Deserialize<'de> for JokerSpec {
    // visit_str  → parse_joker(text).map_err(de::Error::custom); serde adds line and column
    // visit_map  → JokerDetail (the four fields, deny_unknown_fields) → JokerSpec
}
```

Serialization is untouched: a `JokerSpec` still writes the map, so every `.trace.json`, every `parse` output and every page link is byte-for-byte the same. A YAML board may now say:

```yaml
played: [KS, KH, 2C]
held: [KD, KC]
jokers: [BARON+FOIL, LUCKYCATx3.25, JOKER+OFF]
```

**Why not `#[serde(untagged)]`.** The same reason as `CardSpec`: untagged buffers its input and cannot carry a YAML tag like `!MultTimes1Dot 15` in the map form. Do not simplify it back.

### CLI: `jokers` and `--jokers`

`cli/src/lib.rs:19`, `cli/src/main.rs:17`:

```rust
pub enum Command {
    // ...
    /// Explain tokens: one row per joker, or the `JokerSpec` list as JSON.
    Jokers { line: String, json: bool },
    Score { file: Input, jokers: Option<String>, /* ... */ },
    Link  { file: Input, jokers: Option<String>, /* ... */ },
    Parse { file: Input, jokers: Option<String> },
}
```

- `buffoonery jokers "BARON+FOIL LUCKYCATx3.25"` prints a table: token, name, edition, override, debuffed. `--json` prints the `Vec<JokerSpec>` as JSON. A bad token is exit 1 with the `ShorthandError` text.
- `--jokers LINE` on `score`, `link` and `parse` replaces the board's `jokers` with the parsed line before the board goes to the adapter. The file is still required: the hand comes from it. `buffoonery score board.yaml --jokers "BARON+FOIL"` scores `board.yaml`'s hand with one Foil Baron.

The CLI needs a `BoardSpec` to replace into. `read_spec` (`component/src/lib.rs:138`) becomes `pub fn read_spec(text: &str) -> Result<BoardSpec, String>`; the CLI calls it, sets `jokers`, serializes with `serde_json` (already a CLI dependency) and hands the JSON to `score` / `score_with` / `parse`. The five adapter functions are unchanged and the CLI still has no sniff of its own (EPIC-07's rule). **Why replace, not append.** A flag that merges with the file's jokers has two sources of truth and an order question. Replace is one rule.

### The page reads and writes `MultTimes2Dot`

Reading alone is not enough. `slotFromSpec` (`web/src/model.js:296`) would turn the Lucky Cat's `MultTimes2Dot(325)` into an `xmult` override of 3.25, and `buildSpec` (`:245`) would write it back through `overrideEnhancement` (`:190`), which rounds to tenths: `MultTimes1Dot(33)`, ×3.3. Fixture 20 would fail the page's round-trip test (`web/test/model.test.mjs:171`). So the page follows the grammar's rule: the decimal count picks the variant.

`web/src/model.js:275`, one line after the `MultTimes1Dot` one:

```js
  if ('MultTimes2Dot' in mpip) return { kind: 'xmult', value: mpip.MultTimes2Dot / 100 };
```

`overrideEnhancement` (`:203`): a whole number is `MultTimes(n)`; one decimal is `MultTimes1Dot(n × 10)`; two decimals is `MultTimes2Dot(n × 100)`; more than two is rounded to two. A typed ×1.25 now writes `MultTimes2Dot(125)`, where today it writes `MultTimes1Dot(13)` (pinned at `model.test.mjs:93`; the pin moves). This is a deliberate change to what a new page link contains for a two-decimal ×mult; links already written are read as before. The CHANGELOG says so.

**Why not read-only.** Read-only breaks parity on the one fixture this EPIC adds. **Why not leave the page out.** Without the read arm the page shows no override on the slot, and a user cannot see the ×3.25 they pasted.

---

## Work Items

### Phase 0 — `MultTimes2Dot`

- [ ] **0a.** Add `MPip::MultTimes2Dot(usize)` after `MultTimes1Dot` (`src/types/mpip.rs:111`) with its doc comment, and its `Display` arm (`:430`).
- [ ] **0b.** Hundredths arms in `builtin_held_op` (`src/types/board/scoring.rs:344`) and the joker factor (`:656`). List it in `Joker::scores_hand` (`src/decks/joker.rs:2517`). Build until the exhaustive matches compile.
- [ ] **0c.** Tests: `mult_times_2dot__joker_scores_hundredths` (a `JOKER` with `MultTimes2Dot(325)` on a pair: mult × 3.25 exactly), `mult_times_2dot__held_card_scores_hundredths`, and `MultTimes2Dot(325)` added to `yaml__mpip_and_edition_format` (`src/types/board_spec.rs:689`) as `!MultTimes2Dot 325\n`.
- [ ] **0d.** `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` green. `BLESS=1` is not needed: no fixture uses the variant yet.

### Phase 1 — `joker_shorthand`

- [ ] **1.** `src/types/joker_shorthand.rs`: `ShorthandError` with `Display` and `Error`; add `pub mod joker_shorthand;` to `src/types/mod.rs`; `LineError`; re-export `parse_joker`, `parse_jokers`, `shorthand_line`, `ShorthandError`, `LineError` from `src/prelude.rs`.
- [ ] **2.** Ident lookup. Test `parse__ident_ignores_case_and_underscores` (`LUCKYCAT`, `lucky_cat`, `Lucky_Cat` → `LUCKY_CAT`), `parse__unknown_ident_is_an_error`. In `joker_catalog.rs` tests: `idents__unique_without_underscores` and `idents__no_x_before_a_digit`.
- [ ] **3.** Modifiers. Tests `parse__each_edition_short_and_long` (all 7 spellings → 4 `Edition` values), `parse__chips_override`, `parse__mult_override`, `parse__xmult_whole_tenths_hundredths` (`x3` → `MultTimes(3)`, `x3.5` → `MultTimes1Dot(35)`, `x3.25` → `MultTimes2Dot(325)`), `parse__off_is_debuffed`, `parse__modifiers_in_any_order`.
- [ ] **4.** Errors. Tests `parse__two_editions_is_duplicate`, `parse__two_overrides_is_duplicate`, `parse__three_decimals_is_bad_number`, `parse__decimal_chips_is_bad_number`, `parse__unknown_modifier_names_it`, `error__display_quotes_the_token`, `line_error__display_adds_the_position`.
- [ ] **5.** `shorthand()` and `shorthand_line`. Tests `shorthand__canonical_form` (`BARON+FOIL`, `LUCKYCATx3.25`, `JOKER+50c+OFF`), `shorthand__keeps_the_variant` (`MultTimes(3)` → `x3`, `MultTimes1Dot(30)` → `x3.0`, `MultTimes2Dot(350)` → `x3.50`, each reading back to the same spec), `shorthand__round_trips_every_catalog_joker` (every ident, bare), `shorthand__round_trips_every_numeric_override_kind` (`Chips`, `MultPlus`, `MultTimes`, `MultTimes1Dot`, `MultTimes2Dot`), `shorthand__other_override_is_no_shorthand` (`Lucky(5, 15)` and `Custom(1)` → `NoShorthand`).
- [ ] **6.** `parse_jokers`. Tests `parse_jokers__any_run_of_separators` (`BARON,,JOKER`, `BARON , JOKER`, tabs), `parse_jokers__error_carries_position` (third token bad → `jokers[2]`), `parse_jokers__empty_line_is_empty_vec`, `parse_joker__empty_is_an_error`.
- [ ] **7.** Doc tests on every public item, per `CLAUDE.md`. `cargo test --doc` green.

### Phase 2 — `BoardSpec` string jokers and fixture 20

- [ ] **8.** Hand-written `Deserialize` for `JokerSpec` (`src/types/board_spec.rs:135`), `JokerDetail` map struct with `deny_unknown_fields`. Serialize unchanged. Tests `yaml__joker_as_token`, `json__joker_as_token`, `yaml__joker_map_form_still_reads`, `yaml__bad_token_is_a_parse_error_naming_the_token` (the serde message quotes the token and gives a line and column), `json__joker_spec_still_serializes_as_a_map` (pin: a token never appears in output).
- [ ] **9.** `fixtures/golden/20_shorthand_jokers.yaml`: the board in the Design section (held Kings for Baron, a Lucky Cat at ×3.25, a debuffed Joker), written with tokens. `BLESS=1 cargo test --test golden` writes `20_shorthand_jokers.trace.json`; check the total by hand (Baron: ×1.5 per held King, compounding; Lucky Cat: ×3.25; Joker: nothing). The set is now 20, at the cap (`tests/golden.rs:63`); raise the cap to 30 in the same change and say so in the fixture README.
- [ ] **10.** `BLESS=1 cargo test -p buffoonery-cli --test golden` writes `fixtures/cli/20_shorthand_jokers.txt`. Component: `component/tests/golden.rs` walks every fixture, so it covers 20 with no change; add `parse__token_jokers_become_the_map_form` (parse the YAML above, assert the JSON has `"ident": "LUCKY_CAT"` and `"MultTimes2Dot": 325`). `make component` green, size under budget (`component/README.md`).

### Phase 3 — CLI

- [ ] **11.** `Command::Jokers { line, json }` (`cli/src/lib.rs:19`) and the `clap` subcommand (`cli/src/main.rs:17`). Table: token, name, edition, override, debuffed. Tests `run_jokers_prints_one_row_per_token`, `run_jokers_json_is_joker_spec_list`, `run_jokers_bad_token_is_board_error` (exit 1, the `ShorthandError` text on stderr).
- [ ] **12.** Make `read_spec` public (`component/src/lib.rs:138`), returning `BoardSpec`; its error text is unchanged (`parse__errors_name_the_field` in `component/tests/golden.rs:85` still passes). `--jokers LINE` on `score`, `link` and `parse`: `read_spec` the file, replace `jokers`, serialize to JSON, hand to the adapter. Tests `score_jokers_flag_equals_map_form` (fixture 20 scored from its hand with `--jokers "BARON+FOIL LUCKYCATx3.25 JOKER+OFF"` equals its `.trace.json`), `link_jokers_flag_is_in_the_hash`, `parse_jokers_flag_replaces_file_jokers`.
- [ ] **13.** `cli/tests/process.rs`: one run of the binary with `jokers`, one with `score --jokers`, exit codes 0 and 1.

### Phase 4 — the page

- [ ] **14.** `overrideFromEnhancement` reads `MultTimes2Dot` (`web/src/model.js:275`); `overrideEnhancement` (`:203`) writes by decimal count. Node tests in `web/test/model.test.mjs`: fixture 20 opens with the Lucky Cat slot's override `{ kind: 'xmult', value: 3.25 }`; a typed 1.25 writes `MultTimes2Dot(125)` (move the pin at `:93`); a typed 1.5 still writes `MultTimes1Dot(15)`; a typed 3 still writes `MultTimes(3)`. The fixture round trip at `:171` covers 20 with no change. The Playwright walk (`web/e2e/parity.spec.mjs`) opens every fixture; check that 20 passes cell by cell. `make web` and `make e2e` green.

### Phase 5 — Docs and the gate

- [ ] **15.** `README.md`: a "Joker shorthand" section with the grammar table and three examples; the `jokers` command and `--jokers` in the "Command line" section. `fixtures/golden/README.md`: the string form and the new cap.
- [ ] **16.** `.okf/decisions/joker-shorthand.md` (`type: decision`): the grammar, why it is in the engine, why `MultTimes2Dot` and not a change to `1Dot`, why `shorthand()` is a `Result`, why `--jokers` replaces. Append to `.okf/log.md`; add to `.okf/index.md`; update `.okf/architecture/engine.md` (the `MPip` count and the shorthand).
- [ ] **17.** `CHANGELOG.md` Unreleased: engine (`MultTimes2Dot`, `joker_shorthand`, `JokerSpec` string form), CLI (`jokers`, `--jokers`), component (`read_spec` public), page (reads `MultTimes2Dot`; a typed two-decimal ×mult now writes hundredths). `BACKLOG.md` row. `make ayce` green.

---

## Test Plan

- `parse__xmult_whole_tenths_hundredths` — pins which `MPip` each decimal count becomes; the ×3.25 case is the reason this EPIC exists.
- `shorthand__round_trips_every_catalog_joker` — every one of the 112 idents survives `shorthand` → `parse`; a joker added to the catalog is covered with no test change.
- `idents__unique_without_underscores`, `idents__no_x_before_a_digit` — the two catalog facts the grammar leans on. If a future joker breaks one, the test says so before the parser misreads it.
- `shorthand__keeps_the_variant` and `shorthand__other_override_is_no_shorthand` — the token form never rounds, normalizes or drops information.
- `json__joker_spec_still_serializes_as_a_map` — output contracts (`parse`, page links, traces) are unchanged.
- `golden` fixture 20 — a token board scores to a blessed trace; the CLI table and the page agree with it.
- `score_jokers_flag_equals_map_form` — the same board by flag and by map gives the same JSON.
- `mult_times_2dot__joker_scores_hundredths` — ×3.25 scores as ×3.25, not ×3.3.

## Key Files

| File | Role |
| --- | --- |
| `src/types/joker_shorthand.rs` | New: grammar, `ShorthandError`, `parse_joker`, `parse_jokers`, `shorthand()`, `shorthand_line` |
| `src/types/mod.rs`, `src/prelude.rs` | Register and re-export the module |
| `src/types/mpip.rs` | `MultTimes2Dot` variant and `Display` arm |
| `src/types/board/scoring.rs` | Hundredths arms at `:344` and `:656` |
| `src/decks/joker.rs` | `scores_hand` lists the variant (`:2517`) |
| `src/decks/joker_catalog.rs` | Two ident-shape tests |
| `src/types/board_spec.rs` | Hand-written `Deserialize` for `JokerSpec`; YAML pin gains `MultTimes2Dot` |
| `fixtures/golden/20_shorthand_jokers.{yaml,trace.json}` | The token board and its trace |
| `fixtures/cli/20_shorthand_jokers.txt` | Its table |
| `tests/golden.rs` | Set cap 20 → 30 |
| `component/src/lib.rs` | `read_spec` becomes public |
| `component/tests/golden.rs` | `parse__token_jokers_become_the_map_form` |
| `cli/src/lib.rs`, `cli/src/main.rs`, `cli/src/table.rs` | `jokers` command, `--jokers` flag, the jokers table |
| `cli/tests/golden.rs`, `cli/tests/process.rs` | Flag parity and process tests |
| `web/src/model.js`, `web/test/model.test.mjs` | Read `MultTimes2Dot`; write by decimal count |
| `README.md`, `CHANGELOG.md`, `BACKLOG.md`, `.okf/` | Docs |

## Reuse (do NOT recreate)

- `Joker::catalog()` / `Joker::by_ident` (`src/decks/joker_catalog.rs:159`, `:166`) — the ident table. The shorthand normalizes and compares against it; it does not keep a second list.
- `CardSpec`'s hand-written `Deserialize` (`src/types/board_spec.rs:102`) — the string-or-map pattern, copied for `JokerSpec`.
- `BuffoonCard::from_str` (`src/types/buffoon_card.rs:282`) — the precedent and the naming style for a text form in the engine.
- `Edition` (`src/types/edition.rs:17`) and its `Display` — the long spellings come from it, not a new table.
- The page's override kinds (`web/src/model.js:190`, `:270`) — the shorthand's three override kinds are these; do not invent a fourth.
- `read_spec` (`component/src/lib.rs:138`) — the one JSON-or-YAML sniff; made public, not copied.
- `component/tests/golden.rs:18` fixture walk and `cli/tests/golden.rs` — fixture 20 is covered by walking, not by a new harness.
- `cli/src/table.rs` — the `jokers` table uses the same rendering helpers as the score table.

## Compatibility

- **Preserves** every public type and function; `JokerSpec`'s fields and its serialized map form; every golden `.trace.json` (the 19 existing ones do not change: no existing board uses `MultTimes2Dot`); the WIT interface (`component/wit/world.wit`); page links; the CLI's existing commands and flags.
- **Adds** `MPip::MultTimes2Dot`, the `joker_shorthand` module, `FromStr for JokerSpec`, `JokerSpec::shorthand`, string jokers in YAML/JSON, a public `read_spec` on the component, `buffoonery jokers`, `--jokers`, fixture 20.
- **Changes** one page output: a ×mult typed with two decimals is written as `MultTimes2Dot` instead of rounded to tenths. Old links still read.
- **Breaks** nothing. A `match` on `MPip` outside this crate that is exhaustive without a wildcard would stop compiling; `MPip` has 142 variants and the crate has no such downstream match that we know of (cardpack does not depend on buffoonery).

## Dependencies

- **Blocks:** a whole-board one-liner (follow-on; the joker part of that grammar is this token). A solver that enumerates joker rows (it would speak tokens).
- **Built on:** EPIC-05 (`JokerSpec`, the catalog, `BoardSpec` YAML, golden fixtures), EPIC-07 (the CLI and its fixture tests).
- **Related:** the Lucky Cat `Blank` gap (`BACKLOG.md`, "8 Blank jokers"): `LUCKYCATx3.25` is the workaround until scoring mutates.

## Open questions

1. **Should `catalog` print tokens?** `buffoonery catalog` shows ident and name. A `token` column would be the bare ident with no underscores. Recommend: no; the bare token is derivable, and the column adds width.
2. **Comma or space as the canonical separator in `shorthand_line`?** Space reads like a hand (`AS KS`); comma is safer in a shell flag with no quotes. Recommend: space on output, both on input (as specified).

## Verification

```bash
cargo build --workspace
cargo test --workspace
cargo test --doc
cargo clippy --workspace --all-targets -- -D warnings
cargo build --no-default-features
cargo deny check
cargo run -p buffoonery-cli -- jokers "BARON+FOIL LUCKYCATx3.25 JOKER+OFF"
cargo run -p buffoonery-cli -- score fixtures/golden/20_shorthand_jokers.yaml
cargo run -p buffoonery-cli -- score --json fixtures/golden/20_shorthand_jokers.yaml \
  | diff - fixtures/golden/20_shorthand_jokers.trace.json
cargo run -p buffoonery-cli -- score --json fixtures/golden/01_high_card.yaml \
  --jokers "LUCKYCATx3.25"
make ayce
```

Exit criteria:

1. `LUCKYCATx3.25` parses to `MultTimes2Dot(325)` and scores as ×3.25; `shorthand()` writes it back as `LUCKYCATx3.25`.
2. Every catalog joker round-trips through its token; the two catalog-shape tests pass.
3. Fixture 20, written with tokens, scores to its blessed trace in the engine, the component, the CLI and the page.
4. The 19 existing golden traces, every `parse` output and every CLI snapshot are byte-for-byte unchanged.
5. The engine's dependency list is unchanged; `--no-default-features` builds and tests.
6. `make ayce` is green.
