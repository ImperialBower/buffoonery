# EPIC-09: A whole board on one line (LINE)

Oct 23, 2026 · @Christoph · *Written against `main` at `679c03d`. Numbered in the shared cardpack / buffoonery space, after EPIC-08. The follow-on that EPIC-08 named.*

## Summary

- **Builds:** a one-line text form for a whole board: `KS KH 2C / KD KC / BARON+FOIL LUCKYCATx3.25 JOKER+OFF`. Played cards, then held cards, then jokers, then settings such as `FLUSH=2 money=20`.
- **Why:** a board is a YAML file today. EPIC-08 gave jokers a token, but a hand still needs a file. A chat message, a bug report, a shell one-liner or a test table wants the whole board in one line.
- **Shape:** `BoardSpec` learns `FromStr` and a canonical `line()`, in a new engine module with no dependencies. Cards get tokens of their own (`KS+STEEL`, `AH+GLASS+POLY`). The jokers section is EPIC-08's token line, unchanged. The component's sniff learns a third form, so the CLI, the page's paste box and `#board=` links accept a line with no WIT change.
- **Proves it:** every golden fixture round-trips through its line and scores to the same trace; `fixtures/cli/lines.txt` pins the 20 lines; `buffoonery score --board "…"` prints the same JSON as the YAML file.
- **Status:** Planned. No code yet.

---

## Context

The board already has a data form. The line is a third spelling of it, after YAML and JSON, not a new model.

- **A board is eight fields.** `BoardSpec` (`src/types/board_spec.rs:49`): `hand_levels`, `played`, `held`, `jokers`, `money`, `discards_left`, `hands_left`, `blind`. Every field is optional. The line maps onto exactly these eight.
- **A card is an index, or an index with overrides.** `CardSpec` (`src/types/board_spec.rs:79`) is `Index("AS")` or `Detailed { card, enhancement, edition }`. Its hand-written `Deserialize` (`:103`) reads a string or a map. `to_card` (`:270`) checks the index through `BuffoonCard::from_str` (`src/types/buffoon_card.rs:282`), which upper-cases and matches the 52 indexes.
- **Card enhancements are eight fixed values.** Each comes from a tarot: Bonus `Chips(30)` (`MPip::BONUS`, `src/types/mpip.rs:410`; Hierophant `src/decks/tarot.rs:88`), Mult `MultPlus(4)` (Empress, `tarot.rs:70`), Wild `Wild(PipType::Suit)` (Lovers, `:97`), Glass `Glass(2, 4)` (Justice, `:142`), Steel `MultTimes1Dot(15)` (`MPip::STEEL`, `mpip.rs:416`), Stone `Stone(50)` (`MPip::TOWER`, `:418`), Gold `Gold(3)` (`MPip::DEVIL`, `:411`), Lucky `Lucky(5, 15)` (Magician, `tarot.rs:52`). Every enhanced card in the golden set uses one of these eight values (`fixtures/golden/03`, `04`, `05`, `07`, `11`, `19`). There is no card token today.
- **Jokers have a token.** EPIC-08 shipped `parse_joker`, `parse_jokers` and `JokerSpec::shorthand` (`src/types/joker_shorthand.rs:136`, `:171`, `:250`). `ShorthandError` (`:55`) names the token. `LineError` (`:102`) adds the position, but its `Display` (`:111`) hard-codes the word `jokers`.
- **Settings are small types.** `HandType` (`src/types/hands.rs:8`) has 13 variants and no list constant. `Blind` (`src/types/blind.rs:75`) is `Small`, `Big` or `Boss(BossBlind)`; `BossBlind` (`:23`) has three bosses: The Needle, The Water, The Manacle. `money` is `isize` (it can be negative); the two counts are `usize`.
- **The sniff is in one place.** `read_spec` (`component/src/lib.rs:152`) reads JSON when the text starts with `{`, and YAML otherwise. The component's `score` and `parse` (`:38`, `:133`), the CLI's `with_jokers` (`cli/src/lib.rs:173`), the page's paste box and its `#board=` links (`web/src/main.js:53`, `:180`) all go through it. A YAML `BoardSpec` is a map, so it always holds a `:` unless it is empty.
- **The CLI reads boards from files or stdin.** `Command::Score`, `Link` and `Parse` (`cli/src/lib.rs:19`) take an `Input` (`:61`). `--jokers` (EPIC-08) replaces the jokers, but the hand still needs a file.
- **The golden harness reads YAML.** `tests/golden.rs:43` loads each fixture with `BoardSpec::from_yaml`. The set is 20 of a cap of 30 (`:63`).
- **Wild cards do not yet count toward a flush** (`BACKLOG.md`, Bugs). A `7D+WILD` token spells a Wild card; it does not fix that bug.

**This EPIC does not:**

- add seals, card editions the engine does not model, or any field `BoardSpec` lacks. The line spells `BoardSpec`; it does not grow it.
- add scoring logic. No trace changes.
- change any fixture, trace, `parse` output or page link. `BoardSpec` still serializes as a map. Golden fixtures stay YAML.
- make the line the page's link format, or add a "copy line" button. Both are open questions below. The page accepts a pasted line for free through the component.
- change the WIT interface (`component/wit/world.wit`). Its doc comments say "JSON or YAML"; they gain "or a board line".
- express sets of boards or ranges. One line is one board.
- fix wild cards in flushes.

---

## Status

As of 2026-10-23, `main` at `679c03d`. Nothing is built.

| Component | Status |
| --- | --- |
| Phase 0 — `LineError` names its section; `ShorthandError` gains card and setting variants | Planned |
| Phase 1 — card tokens: `parse_card`, `CardSpec::shorthand` | Planned |
| Phase 2 — settings words and the board line: `BoardSpec: FromStr`, `BoardSpec::line` | Planned |
| Phase 3 — the component's sniff reads a line; every golden fixture round-trips | Planned |
| Phase 4 — CLI: `--board LINE` and the `line` command; `fixtures/cli/lines.txt` | Planned |
| Phase 5 — the page: a pasted line opens (Playwright) | Planned |
| Phase 6 — docs, OKF, CHANGELOG | Planned |

---

## Goals

- **One board, one line.** Any `BoardSpec` whose parts all have a token has a line, and the line reads back to the same spec.
- **Reads like the game.** Cards first, as they lie on the table; then held cards; then jokers, left to right; then the settings. `/` splits the sections, as a slash splits a score.
- **Strict.** An unknown card, an unknown word, two enhancements on a card, or a fifth section is an error that names the section, the position and the token. No guessing.
- **One grammar for jokers.** The jokers section is EPIC-08's token line. Nothing in it changes.
- **Pure.** The parser lives in the engine with no new dependencies. The component, the CLI and the page call it; none re-implements it.
- **Free everywhere.** The line works wherever a board is read today: a file, stdin, the page's paste box, a `#board=` link. One sniff, no new WIT function.
- **Shell-safe.** No `$`, `*`, `|`, `;`, `#` or `:` in the grammar. Quoting the line once is enough.

## Scope

The rules the line must obey. Grammar first.

```text
board    = section [ "/" section [ "/" section [ "/" section ] ] ]
                                           ; played / held / jokers / settings
section  = [ word { sep word } ]           ; may be empty
sep      = ( " " | "\t" | "," ) { " " | "\t" | "," }

played, held:  word = card
card     = index { "+" cardmod }
index    = rank suit                       ; "AS", "TD": as BuffoonCard::from_str
cardmod  = enhancement | edition
enhancement = "BONUS" | "MULT" | "WILD" | "GLASS" | "STEEL" | "STONE" | "GOLD" | "LUCKY"
edition  = (EPIC-08's edition words: FOIL, HOLO, HOLOGRAPHIC, POLY, POLYCHROME, NEG, NEGATIVE)

jokers:        word = token                ; EPIC-08's grammar, unchanged

settings:      word = hand "=" integer     ; hand level:     FLUSH=2
                    | "money=" [ "-" ] integer
                    | "hands=" integer     ; hands left
                    | "discards=" integer  ; discards left
                    | "blind=" blind
hand     = a HandType name                 ; HIGHCARD … FLUSHFIVE
blind    = "SMALL" | "BIG" | [ "THE" ] ( "NEEDLE" | "WATER" | "MANACLE" )
```

1. **Sections are by place.** The first is played cards, the second held cards, the third jokers, the fourth settings. A section may be empty: `AS AD / / JOKER` has no held cards. A trailing section may be left out: `AS AD` is two played cards and nothing else. A fifth section is an error.
2. **Case is ignored on input; underscores are ignored in names.** `ks+steel`, `two_pair=2` and `blind=the_water` read. Output is canonical: upper case for cards, enhancements, editions, hands and blinds; lower case for the four setting keys.
3. **A card** is an index, then modifiers in any order: at most one enhancement and at most one edition. A second of either is an error. The index is checked with `BuffoonCard::from_str`; an unknown index is an error, as an unknown joker is in EPIC-08.
4. **Each enhancement word is one fixed `MPip`**, the tarot's value listed in Context. `KS+STEEL` is `Detailed { card: "KS", enhancement: Some(MultTimes1Dot(15)), edition: None }`. A card with no modifiers is `CardSpec::Index`.
5. **A setting** is `key=value`. Each key appears at most once; a hand appears at most once. Numbers are whole; only `money` may be negative. The value is not range-checked here: `FLUSH=0` parses, and `BuffoonBoard::try_from` refuses it as it refuses a YAML level 0 (`try_from__bad_hand_level`, `src/types/board_spec.rs:613`). The line spells the spec; the board checks it.
6. **The canonical form** (`BoardSpec::line()`): sections joined by ` / `, an empty middle section written as nothing (`AS AD / / JOKER`), trailing empty sections dropped. Cards: index, then enhancement, then edition (`AH+GLASS+POLY`). Jokers: `shorthand_line`. Settings in this order: hand levels in `HandType` order, then `money`, `hands`, `discards`, `blind`. The empty spec is the empty line.
7. **Round trip.** `parse(line(s)) == s` for every spec that has a line, and `line(parse(l)) == l` for every canonical line. Nothing is dropped or normalized.
8. **A spec with no line.** `line()` is an error, never a lossy write, when: a card's enhancement is not one of the eight values; a card's edition is `Some(Edition::None)`; a card is `Detailed` with neither enhancement nor edition (it would read back as `Index`); a joker has no token (EPIC-08 rule 6). The error names the section and position: `held[0]: KH: Chips(50) has no shorthand`.
9. **Errors** are typed and say where. A token error carries its section and position: `played[1]: "KX": unknown card`, `settings[0]: "FLSH=2": unknown hand`. Too many sections is its own error: `a board line has at most 4 sections; found 5`.
10. **The sniff.** `read_spec` treats the text as a line when, trimmed, it is not empty, is one line, does not start with `{`, and holds no `:`. Every other text goes to JSON or YAML as today. A line never holds a `:`, and a non-empty YAML board always does, so no board that reads today changes meaning.

---

## Domain map

| Domain concept | Code construct | Status |
| --- | --- | --- |
| A board | `BoardSpec` (`src/types/board_spec.rs:49`) | ✅ |
| A board as text, in a file | YAML / JSON through `read_spec` (`component/src/lib.rs:152`) | ✅ |
| A board as one line | none | ❌ |
| A card as text | `BuffoonCard::from_str` (`src/types/buffoon_card.rs:282`) | 🟡 index only |
| A card with an enhancement as text | none | ❌ |
| A joker row as text | `parse_jokers` (`src/types/joker_shorthand.rs:171`) | ✅ EPIC-08 |
| A hand level, money, a blind as text | none | ❌ |
| A board on the command line | `--jokers` only (`cli/src/lib.rs:19`) | 🟡 jokers only |

---

## Design

### Errors: one vocabulary for every token

`src/types/joker_shorthand.rs:55`, `:102`. `ShorthandError` and `LineError` are not yet released (`CHANGELOG.md`, Unreleased), so they grow now rather than gain a twin.

```rust
pub enum ShorthandError {
    // EPIC-08, unchanged:
    Empty,
    UnknownJoker { token: String, ident: String },
    UnknownModifier { token: String, modifier: String },
    Duplicate { token: String, what: &'static str },   // + "enhancements", "money", "hands", …
    BadNumber { token: String, number: String },
    NoShorthand { ident: String, value: String },        // `ident` is a card index for a card
    // EPIC-09:
    UnknownCard { token: String, index: String },        // "KX": unknown card
    UnknownSetting { token: String, key: String },       // "speed=3": unknown setting
    UnknownHand { token: String, hand: String },         // "FLSH=2": unknown hand
    UnknownBlind { token: String, blind: String },       // "blind=WALL": unknown blind
}

pub struct LineError {
    /// The section: "played", "held", "jokers" or "settings".
    pub section: &'static str,
    pub position: usize,
    pub error: ShorthandError,
}
// Display: "{section}[{position}]: {error}". `parse_jokers` sets "jokers",
// so every EPIC-08 message is byte-for-byte the same.
```

**Why grow, not add.** Two error types for one grammar would make the CLI and the component map both. The types are unreleased, so the change costs nothing downstream. **Why `section` on `LineError`.** The hard-coded `jokers` in `Display` (`:111`) is the only thing stopping reuse.

### Card tokens

`src/types/board_line.rs` (new), beside `joker_shorthand.rs`:

```rust
//! A whole board on one line: `KS KH 2C / KD KC / BARON+FOIL / FLUSH=2`.

/// Reads one card token: `KS`, `KS+STEEL`, `ah+glass+poly`.
///
/// # Errors
/// `Empty`, `UnknownCard`, `UnknownModifier`, `Duplicate`.
pub fn parse_card(token: &str) -> Result<CardSpec, ShorthandError>;

impl FromStr for CardSpec { type Err = ShorthandError; /* parse_card */ }

impl CardSpec {
    /// The canonical token: `KS`, `KS+STEEL`, `AH+GLASS+POLY`.
    ///
    /// # Errors
    /// `NoShorthand` when the enhancement is not one of the eight, the
    /// edition is `Some(Edition::None)`, or the map sets nothing.
    /// `UnknownCard` when the index is not one `BuffoonCard::from_str` reads
    /// exactly (upper case).
    pub fn shorthand(&self) -> Result<String, ShorthandError>;
}
```

The eight words live in one table, read from the tarot constants (`src/decks/tarot.rs`) and `MPip`'s named constants, so the token and the tarot can never disagree:

```rust
const ENHANCEMENTS: [(&str, MPip); 8] = [
    ("BONUS", MPip::BONUS), ("MULT", EMPRESS.enhancement), ("WILD", LOVERS.enhancement),
    ("GLASS", JUSTICE.enhancement), ("STEEL", MPip::STEEL), ("STONE", MPip::TOWER),
    ("GOLD", MPip::DEVIL), ("LUCKY", MAGICIAN.enhancement),
];
```

Card modifiers split on `+` only. Edition words reuse `parse_edition` and `edition_word` (`src/types/joker_shorthand.rs:381`, `:396`), made `pub(crate)`.

**Why validate the index.** EPIC-08 checks the joker ident in the parser, so a bad token fails where it is written. A card is the same. **Why word enhancements, not numbers.** A card enhancement is a fixed tarot value in the game; `KS+STEEL` is what a player says. A hand-written `!Chips 50` card has no token and says so (rule 8). **Why `MULT` is safe.** Card words and joker words never meet: the section says which parser reads a word.

### Settings words

In `src/types/board_line.rs`. Private helpers; the public surface is the board line.

```rust
/// The word for a hand: "FLUSH", "TWOPAIR". An exhaustive match, so a new
/// HandType does not compile until it has a word.
fn hand_word(hand: HandType) -> &'static str;
const HANDS: [HandType; 13] = [/* every variant, in HandType order */];

/// The word for a blind: "SMALL", "BIG", "NEEDLE", "WATER", "MANACLE".
/// Exhaustive over Blind and BossBlind.
fn blind_word(blind: Blind) -> &'static str;
```

**Why `money=20`, not `$20`.** In a double-quoted shell string `$2` is a shell variable: `"… $20"` becomes `"… 0"`. **Why `key=value` for everything.** A setting can never look like a card or a joker, and an unknown key is a clear error. **Why `hands=` and `discards=`, not the field names.** The line is for people; `discards_left=2` is longer and says nothing more.

### The board line

```rust
/// Why a board line could not be read or written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoardLineError {
    /// More than four `/`-separated sections.
    TooManySections { found: usize },
    /// One token failed, with its section and position.
    Token(LineError),
}

impl FromStr for BoardSpec { type Err = BoardLineError; /* … */ }

impl BoardSpec {
    /// The canonical line: `KS KH 2C / KD KC / BARON+FOIL / FLUSH=2 money=20`.
    ///
    /// # Errors
    /// [`BoardLineError::Token`] for the first part with no token form.
    pub fn line(&self) -> Result<String, BoardLineError>;
}
```

Three spellings of fixture 20, all one board:

```yaml
played: [KS, KH, 2C]
held: [KD, KC]
jokers: [BARON+FOIL, LUCKYCATx3.25, JOKER+OFF]
```

```text
KS KH 2C / KD KC / BARON+FOIL LUCKYCATx3.25 JOKER+OFF
```

Fixture 03, with an empty held section and a setting:

```text
AH+BONUS JH+MULT 9H 6H 3H / / DROLLJOKER CRAFTYJOKER / FLUSH=2
```

**Why `/`.** It is not in any token, it needs no shell quoting, and it reads as "then". `|` is a pipe, `;` ends a command. **Why by place, not `played: …`.** A label would bring back the `:` the sniff relies on, and the order is the order of the table. **Why `line()`, not `shorthand()`.** A `BoardSpec` is not a token; `line` says what comes out.

### The sniff

`component/src/lib.rs:152`:

```rust
pub fn read_spec(spec: &str) -> Result<BoardSpec, BoardSpecError> {
    let trimmed = spec.trim();
    if trimmed.starts_with('{') {
        serde_json::from_str(spec).map_err(|e| BoardSpecError::Parse(e.to_string()))
    } else if is_board_line(trimmed) {
        trimmed.parse().map_err(|e: BoardLineError| BoardSpecError::Parse(e.to_string()))
    } else {
        BoardSpec::from_yaml(spec)
    }
}

/// Not empty, one line, no `:`. A YAML board is a map and always has a `:`.
fn is_board_line(text: &str) -> bool;
```

Everything downstream gets the line for free: the WIT `score` and `parse`, the CLI's file and stdin input, the page's paste box (`web/src/main.js:180`) and `#board=` links (`:53`). **Why here and not a new WIT function.** One sniff is EPIC-07's rule; a fourth reader would be a second place to keep in step. **Why `BoardLineError` lives in the engine.** The component maps it to its `Parse` text like the other two forms.

### CLI: `--board` and `line`

`cli/src/lib.rs:19`, `cli/src/main.rs`:

```rust
pub enum Input {
    Stdin,
    Path(PathBuf),
    /// The board itself, from `--board LINE`.
    Line(String),
}

pub enum Command {
    // ...
    /// Print a board's canonical line.
    Line { file: Input },
}
```

- `--board LINE` on `score`, `link` and `parse` gives the board in place of a file. `clap` refuses `--board` with a file (exit 2). `--jokers` still replaces the jokers, whatever the source (EPIC-08's one rule).
- `buffoonery score --board "KS KH 2C / KD KC / BARON+FOIL LUCKYCATx3.25 JOKER+OFF"` prints the score of fixture 20.
- `buffoonery line fixtures/golden/20_shorthand_jokers.yaml` prints `KS KH 2C / KD KC / BARON+FOIL LUCKYCATx3.25 JOKER+OFF`. A board with no line is exit 1 with the `BoardLineError` text.
- `buffoonery link --board "…"` prints a page link. The link still carries the JSON (open question 1).

**Why a flag and not a positional.** `buffoonery score "AS AD"` would make the file name ambiguous. **Why also stdin works.** The sniff is in `read_spec`, so `echo "AS AD" | buffoonery score` needs nothing new; the flag is for the one-liner.

---

## Work Items

### Phase 0 — Errors

- [ ] **0a.** `LineError` gains `section: &'static str` (`src/types/joker_shorthand.rs:102`); `Display` writes `{section}[{position}]` (`:111`); `parse_jokers` and `shorthand_line` set `"jokers"`. Every existing joker test passes unchanged; that is the proof the text did not move.
- [ ] **0b.** `ShorthandError` gains `UnknownCard`, `UnknownSetting`, `UnknownHand`, `UnknownBlind`, each with a `Display` arm. Test `error__display_new_variants` (one assert per variant).
- [ ] **0c.** `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` green.

### Phase 1 — Card tokens

- [ ] **1.** `src/types/board_line.rs`: module doc, `pub mod board_line;` in `src/types/mod.rs`, re-exports from `src/prelude.rs`. `ENHANCEMENTS` table from the tarot constants. Make `parse_edition`, `edition_word` and `parse_whole` `pub(crate)` (`joker_shorthand.rs:381`, `:396`, `:420`).
- [ ] **2.** `parse_card`. Tests `parse_card__index_alone_is_index` (`KS` → `CardSpec::Index("KS")`), `parse_card__case_ignored` (`ks+steel`), `parse_card__each_enhancement_word` (all eight → their tarot `MPip`), `parse_card__edition_words` (the seven EPIC-08 spellings), `parse_card__modifiers_in_any_order`, `parse_card__unknown_index_is_an_error` (`KX`), `parse_card__two_enhancements_is_duplicate`, `parse_card__two_editions_is_duplicate`, `parse_card__unknown_modifier_names_it`, `parse_card__empty_is_an_error`.
- [ ] **3.** `CardSpec::shorthand`. Tests `card_shorthand__canonical_form` (`AH+GLASS+POLY`, index upper case), `card_shorthand__round_trips_every_enhancement_and_edition` (8 × 5 combinations, plus none), `card_shorthand__other_enhancement_is_no_shorthand` (`Chips(50)`, `Custom(1)`), `card_shorthand__edition_none_is_no_shorthand`, `card_shorthand__empty_map_is_no_shorthand`.
- [ ] **4.** Doc tests on `parse_card`, `FromStr for CardSpec`, `CardSpec::shorthand`.

### Phase 2 — Settings and the board line

- [ ] **5.** Settings words: `hand_word`, `HANDS`, `blind_word`. Tests `hands__list_has_every_variant` (13, each word reads back), `blinds__every_blind_has_a_word` (5, each reads back), `settings__the_is_optional` (`blind=THEWATER` and `blind=WATER`), `settings__underscores_ignored` (`TWO_PAIR=2`), `settings__money_may_be_negative` (`money=-5`), `settings__counts_may_not` (`hands=-1` → `BadNumber`), `settings__each_key_once` (`money=1 money=2` → `Duplicate`), `settings__unknown_key_hand_blind_name_it`.
- [ ] **6.** `BoardLineError` with `Display` and `Error`; `FromStr for BoardSpec`. Tests `line__sections_by_place` (1, 2, 3 and 4 sections), `line__empty_middle_section` (`AS / / JOKER`), `line__empty_is_the_default_spec`, `line__five_sections_is_an_error`, `line__error_names_section_and_position` (`played[1]`, `held[0]`, `jokers[2]`, `settings[0]`), `line__separators_are_any_run` (spaces, tabs, commas, as EPIC-08).
- [ ] **7.** `BoardSpec::line`. Tests `line__canonical_form` (fixture 20's and fixture 03's lines from the Design section), `line__settings_in_canonical_order`, `line__trailing_empty_sections_dropped`, `line__no_line_names_section_and_position` (a `Chips(50)` held card → `held[0]`).
- [ ] **8.** Doc tests on every public item. `cargo test --doc` green.

### Phase 3 — The sniff and the golden set

- [ ] **9.** `read_spec` reads a line (`component/src/lib.rs:152`); `is_board_line`. Tests `read_spec__reads_a_board_line`, `read_spec__yaml_with_a_colon_is_still_yaml`, `read_spec__empty_text_is_still_yaml` (today's behavior, pinned), `parse__board_line_becomes_the_map_form` (the component's `parse` on fixture 20's line returns the same JSON as on its YAML), `score__a_bad_line_names_section_and_position`. Update the doc comments on `score` and `parse` in `component/wit/world.wit:22`, `:28`: "JSON, YAML or a board line". `make component` green, size under budget.
- [ ] **10.** `tests/golden.rs`: `every_fixture_round_trips_through_its_line` — for each fixture, `line()` succeeds, `parse(line) == spec`, and scoring the parsed spec gives the blessed trace. All 20 fixtures have a line (checked 2026-10-23: every enhanced card uses a tarot value). A fixture added later with no line fails here and must say why in the test's allow-list.

### Phase 4 — CLI

- [ ] **11.** `Input::Line` and `--board LINE` on `score`, `link`, `parse` (`cli/src/lib.rs:19`, `:61`, `cli/src/main.rs`); `clap` `conflicts_with` the file. Tests `score_board_flag_equals_file` (fixture 20 by flag and by file give the same JSON), `score_board_flag_with_jokers_flag_replaces`, `link_board_flag_is_in_the_hash`, `score_bad_board_flag_is_board_error` (exit 1, `played[1]: "KX": unknown card`).
- [ ] **12.** `Command::Line { file }`. Tests `run_line_prints_the_canonical_line`, `run_line_no_line_is_board_error`.
- [ ] **13.** `fixtures/cli/lines.txt`: one row per golden fixture, `name: line`, written by `BLESS=1 cargo test -p buffoonery-cli --test golden`. This is both a pin and a page of 20 worked examples.
- [ ] **14.** `cli/tests/process.rs`: one run with `score --board`, one with `line`, one with `--board` and a file (exit 2).

### Phase 5 — The page

- [ ] **15.** No page code. `web/e2e/parity.spec.mjs`, beside `a pasted board opens like a link does` (`:85`): `a pasted line opens like its YAML` (fixture 20's line in the paste box gives fixture 20's cells), and `a line in the link opens` (`#board=` with fixture 03's line). A bad line shows `Could not open it: played[1]: "KX": unknown card`. `make e2e` green.

### Phase 6 — Docs and the gate

- [ ] **16.** `README.md`: a "Board line" section after "Joker shorthand" (`README.md:99`): the grammar, the eight card words, the settings, three examples. `--board` and `line` in "Command line" (`:60`).
- [ ] **17.** `.okf/decisions/board-line.md` (`type: decision`): `/` by place, `key=value` settings, why not `$`, why the sniff and not a WIT function, why `line()` refuses lossy writes. Update `.okf/decisions/joker-shorthand.md` (`LineError.section`). Append to `.okf/log.md`; add to `.okf/index.md`.
- [ ] **18.** `CHANGELOG.md` Unreleased: engine (`board_line`, `parse_card`, `CardSpec::shorthand`, `BoardSpec: FromStr`, `BoardSpec::line`, `LineError.section`, new `ShorthandError` variants), component (`read_spec` reads a line), CLI (`--board`, `line`). `BACKLOG.md` row. `make ayce` green.

---

## Test Plan

- `every_fixture_round_trips_through_its_line` — the central claim: 20 real boards, line and back, same spec, same trace.
- `card_shorthand__round_trips_every_enhancement_and_edition` — every card word survives the round trip, and the eight words are the tarot values.
- `hands__list_has_every_variant`, `blinds__every_blind_has_a_word` — with the exhaustive matches, a new `HandType` or `BossBlind` fails to compile or fails here, never silently has no word.
- `card_shorthand__other_enhancement_is_no_shorthand`, `line__no_line_names_section_and_position` — `line()` never rounds, normalizes or drops.
- `line__error_names_section_and_position` — every error says where.
- `read_spec__yaml_with_a_colon_is_still_yaml`, `read_spec__empty_text_is_still_yaml` — the sniff changes no board that reads today.
- `score_board_flag_equals_file` — the same board by flag and by file gives the same JSON.
- EPIC-08's joker tests, unchanged — `LineError`'s new field moves no message.
- `fixtures/cli/lines.txt` — a pinned, human-readable list of 20 lines.
- Playwright `a pasted line opens like its YAML` — the page gets the line with no page code.

## Key Files

| File | Role |
| --- | --- |
| `src/types/board_line.rs` | New: `parse_card`, `CardSpec::shorthand`, settings words, `BoardLineError`, `FromStr for BoardSpec`, `BoardSpec::line` |
| `src/types/joker_shorthand.rs` | `LineError.section`; four new `ShorthandError` variants; three helpers `pub(crate)` |
| `src/types/mod.rs`, `src/prelude.rs` | Register and re-export |
| `component/src/lib.rs`, `component/wit/world.wit` | The sniff; doc comments |
| `component/tests/golden.rs` | Line parse and score tests |
| `tests/golden.rs` | Every fixture round-trips through its line |
| `cli/src/lib.rs`, `cli/src/main.rs` | `Input::Line`, `--board`, `line` command |
| `cli/tests/golden.rs`, `cli/tests/process.rs` | `lines.txt` snapshot; process runs |
| `fixtures/cli/lines.txt` | New: the 20 lines |
| `web/e2e/parity.spec.mjs` | Paste and link a line |
| `README.md`, `CHANGELOG.md`, `BACKLOG.md`, `.okf/` | Docs |

## Reuse (do NOT recreate)

- `parse_jokers`, `shorthand_line` (`src/types/joker_shorthand.rs:171`, `:194`) — the jokers section, whole.
- `parse_edition`, `edition_word`, `parse_whole` (`:381`, `:396`, `:420`) — edition words and whole numbers for cards and settings.
- `BuffoonCard::from_str` (`src/types/buffoon_card.rs:282`) — the index check; no second card table.
- The tarot constants (`src/decks/tarot.rs:48`–`:187`) and `MPip::BONUS`, `STEEL`, `TOWER`, `DEVIL` (`src/types/mpip.rs:410`–`:418`) — the eight enhancement values.
- `read_spec` (`component/src/lib.rs:152`) — the one sniff; it grows a branch, it is not copied.
- `with_jokers` (`cli/src/lib.rs:173`) — `--jokers` keeps working on a `--board` input through it.
- `board_link` (`cli/src/link.rs:12`) — `link --board` reuses it unchanged.
- The golden fixture walk (`tests/golden.rs:43`) and the CLI bless pattern (`cli/tests/golden.rs`) — the round trip and `lines.txt` use them.
- The paste test (`web/e2e/parity.spec.mjs:85`) — the line tests copy its shape.

## Compatibility

- **Preserves** every golden trace, every `parse` output, every page link, the WIT interface, the YAML and JSON forms, every EPIC-08 error message, and every existing CLI command and flag.
- **Adds** the `board_line` module, `parse_card`, `FromStr` and `shorthand()` for `CardSpec`, `FromStr` and `line()` for `BoardSpec`, `BoardLineError`, a line branch in `read_spec`, `--board`, `buffoonery line`, `fixtures/cli/lines.txt`.
- **Changes** two unreleased types: `LineError` gains a public field and `ShorthandError` gains four variants. Both shipped in EPIC-08 and are in `CHANGELOG.md` Unreleased, so no released API changes. A struct literal `LineError { position, error }` outside the crate would stop compiling; there is none.
- **Breaks** nothing released. A text that is one line with no `:` used to fail as YAML; it now reads as a line or fails with a line error.

## Dependencies

- **Blocks:** a page link that carries the line instead of JSON (open question 1). A test table of boards as lines.
- **Built on:** EPIC-08 (the joker token grammar, `ShorthandError`, `LineError`), EPIC-07 (the CLI, the one-sniff rule, the page's paste box), EPIC-05 (`BoardSpec`, the golden set).
- **Related:** "Wild cards do not count toward a flush" (`BACKLOG.md`): `+WILD` spells the card; the bug is separate. Release 0.1.0: `ShorthandError` and `LineError` should settle before it.

## Open questions

1. **Should the page link carry the line?** `#board=KS%20KH…` is far shorter than the JSON. But a board with a custom effect or an odd `MPip` has no line, so the page would need both forms and a rule. Recommend: not in this EPIC; the page already reads a line link, so a later change only touches the write side.
2. **Should the page show the board's line with a copy button?** It needs `line()` in the page, which means a new WIT function. Recommend: not in this EPIC, for the same reason as EPIC-08 kept tokens out of the page.
3. **Should `line()` normalize instead of refuse?** A `Detailed` card with nothing set, or with `Edition::None`, is the same board as the bare index. Refusing keeps the exact round trip, as EPIC-08 did for jokers. Recommend: refuse; no golden fixture hits it.

## Verification

```bash
cargo build --workspace
cargo test --workspace
cargo test --doc
cargo clippy --workspace --all-targets -- -D warnings
cargo build --no-default-features
cargo deny check
cargo run -p buffoonery-cli -- line fixtures/golden/20_shorthand_jokers.yaml
cargo run -p buffoonery-cli -- score --board "KS KH 2C / KD KC / BARON+FOIL LUCKYCATx3.25 JOKER+OFF"
cargo run -p buffoonery-cli -- score --json --board "KS KH 2C / KD KC / BARON+FOIL LUCKYCATx3.25 JOKER+OFF" \
  | diff - fixtures/golden/20_shorthand_jokers.trace.json
echo "AH+BONUS JH+MULT 9H 6H 3H / / DROLLJOKER CRAFTYJOKER / FLUSH=2" | cargo run -p buffoonery-cli -- score
make ayce
```

Exit criteria:

1. Every golden fixture has a line; the line reads back to the same spec and scores to the blessed trace.
2. `buffoonery score --board` on fixture 20's line prints the same JSON as the YAML file.
3. A pasted line and a `#board=` line open in the page and show the same cells as the YAML.
4. Every existing trace, `parse` output, CLI snapshot and EPIC-08 error message is byte-for-byte unchanged.
5. The engine's dependency list is unchanged; `--no-default-features` builds and tests. The component stays under its size budget.
6. `make ayce` is green.
