//! The buffoonery engine on the command line (EPIC-07).
//!
//! A second door over the same adapter functions the WIT component exports
//! (`buffoonery_component`): `score`, `score_with`, `parse`, `catalog` and
//! `version`. The CLI reads files and prints; it does no scoring of its own.
//!
//! [`run`] does the work without a process, so the tests call it directly;
//! `main.rs` only parses the arguments and maps the result to an exit code.

pub mod link;
pub mod table;

use buffoonery::prelude::*;
use std::io::{Read, Write};
use std::path::PathBuf;

/// One command line, parsed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    /// Print the engine version.
    Version,
    /// Read a board, print it as JSON.
    Parse {
        file: Input,
        /// A joker line (`"BARON+FOIL LUCKYCATx3.25"`) that replaces the
        /// board's jokers.
        jokers: Option<String>,
    },
    /// Explain a joker line: one row per token, or the jokers as JSON.
    Jokers { line: String, json: bool },
    /// Read a board in any form, print its canonical board line (EPIC-09).
    Line { file: Input },
    /// List the jokers: a table, or the component's JSON.
    Catalog { json: bool, find: Option<String> },
    /// Score a board: the trace as a table, or the component's JSON.
    Score {
        file: Input,
        /// Print the component's JSON, unchanged.
        json: bool,
        /// Show every step, not only the ones the page shows.
        all: bool,
        /// Roll the chances from this seed.
        seed: Option<u64>,
        /// Custom effects, as the JSON list the page puts in its link.
        effects: Option<Input>,
        /// A joker line that replaces the board's jokers.
        jokers: Option<String>,
    },
    /// Print a link that opens the board in the page.
    Link {
        file: Input,
        seed: Option<u64>,
        effects: Option<Input>,
        /// A joker line that replaces the board's jokers.
        jokers: Option<String>,
        /// The page's address, e.g. [`link::PAGE`] or `make dev`'s.
        base: String,
    },
}

/// Where a board comes from: a file, stdin (`-` or no file), or the board
/// itself from `--board`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    Stdin,
    Path(PathBuf),
    /// The text of `--board LINE` (EPIC-09). Any form the adapter reads
    /// works; a board line is the one that fits on a command line.
    Line(String),
}

/// Why a command failed. Each kind has its own exit code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CliError {
    /// A file named on the line cannot be read, or output cannot be written.
    /// Exit 2.
    Io(String),
    /// The board or effects are not valid: the adapter's message, unchanged.
    /// Exit 1.
    Board(String),
}

impl CliError {
    /// The process exit code: 1 for a bad board, 2 for a usage or file
    /// problem (the code `clap` uses for a bad argument).
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Board(_) => 1,
            Self::Io(_) => 2,
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(message) | Self::Board(message) => f.write_str(message),
        }
    }
}

/// Runs one command. Writes its output to `out`; the caller prints errors.
///
/// # Errors
///
/// [`CliError::Io`] when a file cannot be read or `out` cannot be written;
/// [`CliError::Board`] with the adapter's message when the board is not
/// valid.
pub fn run(command: &Command, stdin: &mut dyn Read, out: &mut dyn Write) -> Result<(), CliError> {
    let text = match command {
        Command::Version => buffoonery_component::version(),
        Command::Parse { file, jokers } => {
            let spec = with_jokers(read_input(file, stdin)?, jokers.as_deref())?;
            buffoonery_component::parse(&spec).map_err(CliError::Board)?
        }
        Command::Catalog { json, find } => catalog(*json, find.as_deref()),
        Command::Jokers { line, json } => jokers(line, *json)?,
        Command::Line { file } => board_line(&read_input(file, stdin)?)?,
        Command::Score {
            file,
            json,
            all,
            seed,
            effects,
            jokers,
        } => {
            let (spec, effects) = read_board(file, effects.as_ref(), jokers.as_deref(), stdin)?;
            let trace = score(&spec, *seed, effects)?;
            if *json {
                trace
            } else {
                let trace: ScoreTrace =
                    serde_json::from_str(&trace).map_err(|e| CliError::Board(e.to_string()))?;
                table::render(&table::rows(&trace, *all), &trace)
            }
        }
        Command::Link {
            file,
            seed,
            effects,
            jokers,
            base,
        } => {
            let (spec, effects) = read_board(file, effects.as_ref(), jokers.as_deref(), stdin)?;
            // The page opens only a board it can score; so does the link.
            score(&spec, *seed, effects.clone())?;
            let spec_json = buffoonery_component::parse(&spec).map_err(CliError::Board)?;
            let effects_json = (!effects.is_empty()).then(|| page_effects(&effects));
            link::board_link(base, &spec_json, *seed, effects_json.as_deref())
        }
    };
    writeln!(out, "{text}").map_err(|e| CliError::Io(e.to_string()))
}

/// The board text, its jokers replaced by `jokers` when given, and its
/// custom effects. Only one of board and effects can come from stdin.
fn read_board(
    file: &Input,
    effects: Option<&Input>,
    jokers: Option<&str>,
    stdin: &mut dyn Read,
) -> Result<(String, Vec<buffoonery_component::CustomEffect>), CliError> {
    if *file == Input::Stdin && effects == Some(&Input::Stdin) {
        return Err(CliError::Io(
            "stdin can hold the board or the effects, not both".to_string(),
        ));
    }
    let spec = with_jokers(read_input(file, stdin)?, jokers)?;
    let effects = match effects {
        Some(input) => read_effects(&read_input(input, stdin)?)?,
        None => Vec::new(),
    };
    Ok((spec, effects))
}

/// `spec` with its jokers replaced by the joker line, as JSON; `spec`
/// unchanged when there is no line. Replace, not append: the flag is the one
/// source of the jokers.
fn with_jokers(spec: String, jokers: Option<&str>) -> Result<String, CliError> {
    let Some(line) = jokers else {
        return Ok(spec);
    };
    let mut spec =
        buffoonery_component::read_spec(&spec).map_err(|e| CliError::Board(e.to_string()))?;
    spec.jokers = parse_jokers(line).map_err(|e| CliError::Board(e.to_string()))?;
    serde_json::to_string(&spec).map_err(|e| CliError::Board(e.to_string()))
}

/// A joker line explained: a table of token, name, edition, override and
/// debuffed, or the `JokerSpec` list as JSON.
fn jokers(line: &str, json: bool) -> Result<String, CliError> {
    let jokers = parse_jokers(line).map_err(|e| CliError::Board(e.to_string()))?;
    if json {
        return serde_json::to_string(&jokers).map_err(|e| CliError::Board(e.to_string()));
    }
    let mut rows = vec![
        ["Token", "Name", "Edition", "Override", "Debuffed"]
            .map(String::from)
            .to_vec(),
    ];
    for joker in &jokers {
        // A parsed joker always has a token and a catalog name.
        let token = joker
            .shorthand()
            .map_err(|e| CliError::Board(e.to_string()))?;
        let name = Joker::by_ident(&joker.ident)
            .and_then(|card| Joker::name(&card))
            .unwrap_or_default();
        rows.push(vec![
            token,
            name.to_string(),
            joker
                .edition
                .map(|edition| edition.to_string())
                .unwrap_or_default(),
            joker.enhancement.map(override_text).unwrap_or_default(),
            if joker.debuffed { "yes" } else { "" }.to_string(),
        ]);
    }
    Ok(table::columns(&rows, &[false; 5]))
}

/// An override as the breakdown writes its op: `+50 Chips`, `×3.25 Mult`.
#[allow(clippy::cast_precision_loss)]
fn override_text(enhancement: MPip) -> String {
    let op = match enhancement {
        MPip::Chips(chips) => ScoreOp::AddChips(chips),
        MPip::MultPlus(mult) => ScoreOp::AddMult(mult),
        MPip::MultTimes(times) => ScoreOp::TimesMult(times as f64),
        MPip::MultTimes1Dot(tenths) => ScoreOp::TimesMult(tenths as f64 / 10.0),
        MPip::MultTimes2Dot(hundredths) => ScoreOp::TimesMult(hundredths as f64 / 100.0),
        other => return other.to_string(),
    };
    op.to_string()
}

/// A board in any form the adapter reads, written as its canonical line.
fn board_line(text: &str) -> Result<String, CliError> {
    buffoonery_component::read_spec(text)
        .map_err(|e| CliError::Board(e.to_string()))?
        .line()
        .map_err(|e| CliError::Board(e.to_string()))
}

/// The trace as the component's JSON: `score`, or `score_with` when there is
/// a seed or a custom effect.
fn score(
    spec: &str,
    seed: Option<u64>,
    effects: Vec<buffoonery_component::CustomEffect>,
) -> Result<String, CliError> {
    if seed.is_none() && effects.is_empty() {
        buffoonery_component::score(spec)
    } else {
        let options = buffoonery_component::ScoreOptions { seed, effects };
        buffoonery_component::score_with(spec, &options)
    }
    .map_err(CliError::Board)
}

/// Custom effects as the page keeps them in its link: each op an object.
/// [`read_effects`] has checked that each op is JSON.
fn page_effects(effects: &[buffoonery_component::CustomEffect]) -> String {
    let list: Vec<serde_json::Value> = effects
        .iter()
        .map(|effect| {
            serde_json::json!({
                "id": effect.id,
                "op": serde_json::from_str::<serde_json::Value>(&effect.op)
                    .unwrap_or(serde_json::Value::Null),
            })
        })
        .collect();
    serde_json::Value::Array(list).to_string()
}

/// The text of `input`. A file that cannot be read is an [`CliError::Io`]
/// that names it.
fn read_input(input: &Input, stdin: &mut dyn Read) -> Result<String, CliError> {
    match input {
        Input::Stdin => {
            let mut text = String::new();
            stdin
                .read_to_string(&mut text)
                .map_err(|e| CliError::Io(format!("stdin: {e}")))?;
            Ok(text)
        }
        Input::Path(path) => std::fs::read_to_string(path)
            .map_err(|e| CliError::Io(format!("{}: {e}", path.display()))),
        Input::Line(text) => Ok(text.clone()),
    }
}

/// Custom effects from the JSON list the page puts in its link:
/// `[{"id": 1, "op": {"TimesMult": 2.0}}]`. The component takes each op as
/// JSON text, so an op object is written back as text; an op that is already
/// text passes through.
fn read_effects(text: &str) -> Result<Vec<buffoonery_component::CustomEffect>, CliError> {
    let not_a_list = || CliError::Board("effects: not a list of {id, op}".to_string());
    let serde_json::Value::Array(list) =
        serde_json::from_str(text).map_err(|e| CliError::Board(format!("effects: {e}")))?
    else {
        return Err(not_a_list());
    };
    list.iter()
        .map(|effect| {
            let id = effect["id"]
                .as_u64()
                .and_then(|id| u32::try_from(id).ok())
                .ok_or_else(not_a_list)?;
            let op = match &effect["op"] {
                serde_json::Value::Null => return Err(not_a_list()),
                serde_json::Value::String(op) => op.clone(),
                op => op.to_string(),
            };
            Ok(buffoonery_component::CustomEffect { id, op })
        })
        .collect()
}

/// The joker list. `--json` is the component's `catalog()`, as the page reads
/// it; the table shows ident, name, rarity and cost. `find` keeps the jokers
/// whose ident or name contains it, ignoring case.
fn catalog(json: bool, find: Option<&str>) -> String {
    let find = find.map(str::to_lowercase);
    let wanted = |ident: &str, name: &str| {
        find.as_ref().is_none_or(|find| {
            ident.to_lowercase().contains(find) || name.to_lowercase().contains(find)
        })
    };
    if json {
        let all = buffoonery_component::catalog();
        if find.is_none() {
            return all;
        }
        let Ok(serde_json::Value::Array(jokers)) = serde_json::from_str(&all) else {
            return all;
        };
        let kept: Vec<_> = jokers
            .into_iter()
            .filter(|joker| {
                wanted(
                    joker["ident"].as_str().unwrap_or_default(),
                    joker["name"].as_str().unwrap_or_default(),
                )
            })
            .collect();
        return serde_json::Value::Array(kept).to_string();
    }
    let mut rows = vec![vec![
        "Ident".to_string(),
        "Name".to_string(),
        "Rarity".to_string(),
        "Cost".to_string(),
    ]];
    for info in Joker::catalog() {
        if wanted(info.ident, info.name) {
            rows.push(vec![
                info.ident.to_string(),
                info.name.to_string(),
                rarity(info.rarity).to_string(),
                format!("${}", info.cost),
            ]);
        }
    }
    table::columns(&rows, &[false; 4])
}

fn rarity(card_type: BCardType) -> &'static str {
    match card_type {
        BCardType::CommonJoker => "Common",
        BCardType::UncommonJoker => "Uncommon",
        BCardType::RareJoker => "Rare",
        BCardType::LegendaryJoker => "Legendary",
        _ => "",
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::path::Path;

    fn fixture(name: &str) -> Input {
        Input::Path(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../fixtures/golden")
                .join(name),
        )
    }

    /// Runs `command` with `stdin`, returning stdout as text.
    fn output(command: &Command, stdin: &str) -> Result<String, CliError> {
        let mut out = Vec::new();
        run(command, &mut stdin.as_bytes(), &mut out)?;
        Ok(String::from_utf8(out).unwrap())
    }

    #[test]
    fn run_version_prints_engine_version() {
        assert_eq!(
            output(&Command::Version, "").unwrap(),
            format!("{}\n", buffoonery_component::version())
        );
    }

    #[test]
    fn run_parse_yaml_prints_json() {
        let command = Command::Parse {
            file: fixture("01_high_card.yaml"),
            jokers: None,
        };
        let yaml = std::fs::read_to_string(match &command {
            Command::Parse {
                file: Input::Path(path),
                ..
            } => path,
            _ => unreachable!(),
        })
        .unwrap();
        assert_eq!(
            output(&command, "").unwrap(),
            format!("{}\n", buffoonery_component::parse(&yaml).unwrap())
        );
    }

    #[test]
    fn run_parse_reads_stdin() {
        let command = Command::Parse {
            file: Input::Stdin,
            jokers: None,
        };
        assert_eq!(
            output(&command, "played: [AS]").unwrap(),
            "{\"played\":[\"AS\"]}\n"
        );
    }

    #[test]
    fn run_catalog_json_equals_component() {
        let command = Command::Catalog {
            json: true,
            find: None,
        };
        assert_eq!(
            output(&command, "").unwrap(),
            format!("{}\n", buffoonery_component::catalog())
        );
    }

    #[test]
    fn run_catalog_find_filters() {
        let command = Command::Catalog {
            json: false,
            find: Some("JOLLY".into()),
        };
        let text = output(&command, "").unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2, "a header and one joker:\n{text}");
        assert!(lines[1].starts_with("JOLLY_JOKER"), "{text}");
        assert!(lines[1].contains("Jolly Joker"), "{text}");
        assert!(lines[1].contains("Common"), "{text}");
    }

    #[test]
    fn run_catalog_find_filters_json() {
        let command = Command::Catalog {
            json: true,
            find: Some("jolly".into()),
        };
        let json: serde_json::Value = serde_json::from_str(&output(&command, "").unwrap()).unwrap();
        let jokers = json.as_array().unwrap();
        assert_eq!(jokers.len(), 1);
        assert_eq!(jokers[0]["ident"], "JOLLY_JOKER");
    }

    #[test]
    fn run_catalog_table_lists_every_joker() {
        let command = Command::Catalog {
            json: false,
            find: None,
        };
        let text = output(&command, "").unwrap();
        assert_eq!(text.lines().count(), 1 + Joker::catalog().len());
    }

    #[test]
    fn run_missing_file_is_io_error() {
        let command = Command::Parse {
            file: fixture("no_such_board.yaml"),
            jokers: None,
        };
        let error = output(&command, "").unwrap_err();
        assert!(matches!(error, CliError::Io(_)), "{error:?}");
        assert!(error.to_string().contains("no_such_board.yaml"), "{error}");
        assert_eq!(error.exit_code(), 2);
    }

    fn score(file: Input, json: bool) -> Command {
        Command::Score {
            file,
            json,
            all: false,
            seed: None,
            effects: None,
            jokers: None,
        }
    }

    #[test]
    fn run_score_json_equals_component() {
        let yaml = "played: [AS, AH]\njokers: [{ident: JOLLY_JOKER}]";
        assert_eq!(
            output(&score(Input::Stdin, true), yaml).unwrap(),
            format!("{}\n", buffoonery_component::score(yaml).unwrap())
        );
    }

    #[test]
    fn run_score_table_ends_with_total() {
        let text = output(&score(Input::Stdin, false), "played: [AS, AH]").unwrap();
        assert!(text.ends_with("Pair  32 × 2 = 64\n"), "{text}");
    }

    #[test]
    fn score_seed_matches_score_with() {
        let command = Command::Score {
            file: fixture("11_inert_jokers.yaml"),
            json: true,
            all: false,
            seed: Some(7),
            effects: None,
            jokers: None,
        };
        let yaml = std::fs::read_to_string(match &command {
            Command::Score {
                file: Input::Path(path),
                ..
            } => path,
            _ => unreachable!(),
        })
        .unwrap();
        let options = buffoonery_component::ScoreOptions {
            seed: Some(7),
            effects: Vec::new(),
        };
        let expected = buffoonery_component::score_with(&yaml, &options).unwrap();
        assert!(expected.contains(r#""rolled":true"#));
        assert_eq!(output(&command, "").unwrap(), format!("{expected}\n"));
    }

    #[test]
    fn score_effects_reads_page_format() {
        let dir = std::env::temp_dir().join(format!("buffoonery-cli-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let effects = dir.join("effects.json");
        // The list the page writes in its link: each op is a `ScoreOp` object.
        std::fs::write(
            &effects,
            r#"[{"id":1,"op":{"Seq":[{"AddChips":20},{"TimesMult":2.0}]}}]"#,
        )
        .unwrap();
        let command = Command::Score {
            file: Input::Stdin,
            json: false,
            all: false,
            seed: None,
            effects: Some(Input::Path(effects)),
            jokers: None,
        };
        let text = output(
            &command,
            "played: [AS]\njokers: [{ident: JOKER, enhancement: !Custom 1}]",
        )
        .unwrap();
        assert!(text.contains("+20 Chips, ×2 Mult"), "{text}");
        assert!(text.ends_with("High Card  36 × 2 = 72\n"), "{text}");
    }

    #[test]
    fn score_effects_that_are_not_a_list_are_a_board_error() {
        let dir = std::env::temp_dir().join(format!("buffoonery-cli-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let effects = dir.join("effects.json");
        std::fs::write(&effects, r#"{"id":1}"#).unwrap();
        let command = Command::Score {
            file: Input::Stdin,
            json: true,
            all: false,
            seed: None,
            effects: Some(Input::Path(effects)),
            jokers: None,
        };
        let error = output(&command, "played: [AS]").unwrap_err();
        assert!(matches!(error, CliError::Board(_)), "{error:?}");
        assert!(error.to_string().starts_with("effects:"), "{error}");
    }

    #[test]
    fn score_board_and_effects_cannot_both_be_stdin() {
        let command = Command::Score {
            file: Input::Stdin,
            json: true,
            all: false,
            seed: None,
            effects: Some(Input::Stdin),
            jokers: None,
        };
        let error = output(&command, "played: [AS]").unwrap_err();
        assert_eq!(error.exit_code(), 2, "{error}");
    }

    #[test]
    fn run_bad_joker_is_board_error() {
        let error = output(&score(Input::Stdin, true), "jokers: [{ident: NOPE}]").unwrap_err();
        assert_eq!(
            error,
            CliError::Board(r#"jokers[0]: unknown joker ident "NOPE""#.to_string())
        );
        assert_eq!(error.exit_code(), 1);
    }

    fn link(file: Input, seed: Option<u64>) -> Command {
        Command::Link {
            file,
            seed,
            effects: None,
            jokers: None,
            base: link::PAGE.to_string(),
        }
    }

    #[test]
    fn run_link_prints_the_page_link() {
        let text = output(&link(Input::Stdin, Some(7)), "played: [AS]").unwrap();
        assert_eq!(
            text,
            format!(
                "{}\n",
                link::board_link(link::PAGE, r#"{"played":["AS"]}"#, Some(7), None)
            )
        );
    }

    #[test]
    fn run_link_writes_effects_as_the_page_does() {
        let dir = std::env::temp_dir().join(format!("buffoonery-cli-link-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let effects = dir.join("effects.json");
        // An op written as text comes out as an object, as the page keeps it.
        std::fs::write(&effects, r#"[{"id":1,"op":"{\"TimesMult\":2.0}"}]"#).unwrap();
        let command = Command::Link {
            file: Input::Stdin,
            seed: None,
            effects: Some(Input::Path(effects)),
            jokers: None,
            base: "http://x/".to_string(),
        };
        let text = output(
            &command,
            "played: [AS]\njokers: [{ident: JOKER, enhancement: !Custom 1}]",
        )
        .unwrap();
        let effects = text
            .split("effects=")
            .nth(1)
            .and_then(|rest| rest.split('&').next())
            .unwrap();
        assert_eq!(
            effects,
            link::encode_uri_component(r#"[{"id":1,"op":{"TimesMult":2.0}}]"#)
        );
    }

    #[test]
    fn run_link_rejects_a_board_the_page_cannot_score() {
        let error = output(&link(Input::Stdin, None), "jokers: [{ident: NOPE}]").unwrap_err();
        assert_eq!(error.exit_code(), 1, "{error}");
    }

    #[test]
    fn run_bad_spec_is_board_error() {
        let command = Command::Parse {
            file: Input::Stdin,
            jokers: None,
        };
        let error = output(&command, "nonsense: 1").unwrap_err();
        assert_eq!(
            error,
            CliError::Board(buffoonery_component::parse("nonsense: 1").unwrap_err())
        );
        assert_eq!(error.exit_code(), 1);
    }

    // ---- Joker shorthand (EPIC-08) -------------------------------------------

    /// Fixture 20's hand, without its jokers.
    const FIXTURE_20_HAND: &str = "played: [KS, KH, 2C]\nheld: [KD, KC]";
    const FIXTURE_20_JOKERS: &str = "BARON+FOIL LUCKYCATx3.25 JOKER+OFF";

    fn jokers(line: &str, json: bool) -> Command {
        Command::Jokers {
            line: line.to_string(),
            json,
        }
    }

    #[test]
    fn run_jokers_prints_one_row_per_token() {
        let text = output(&jokers(FIXTURE_20_JOKERS, false), "").unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 4, "a header and three jokers:\n{text}");
        for heading in ["Token", "Name", "Edition", "Override", "Debuffed"] {
            assert!(lines[0].contains(heading), "{text}");
        }
        assert!(lines[1].starts_with("BARON+FOIL"), "{text}");
        assert!(
            lines[1].contains("Baron") && lines[1].contains("Foil"),
            "{text}"
        );
        assert!(lines[2].starts_with("LUCKYCATx3.25"), "{text}");
        assert!(
            lines[2].contains("Lucky Cat") && lines[2].contains("×3.25 Mult"),
            "{text}"
        );
        assert!(lines[3].starts_with("JOKER+OFF"), "{text}");
        assert!(lines[3].trim_end().ends_with("yes"), "{text}");
    }

    #[test]
    fn run_jokers_writes_the_canonical_token() {
        let text = output(&jokers("lucky_cat+holographic", false), "").unwrap();
        assert!(
            text.lines().nth(1).unwrap().starts_with("LUCKYCAT+HOLO "),
            "{text}"
        );
    }

    #[test]
    fn run_jokers_json_is_joker_spec_list() {
        let text = output(&jokers(FIXTURE_20_JOKERS, true), "").unwrap();
        let expected = serde_json::to_string(&parse_jokers(FIXTURE_20_JOKERS).unwrap()).unwrap();
        assert_eq!(text, format!("{expected}\n"));
        assert!(text.contains(r#""MultTimes2Dot":325"#), "{text}");
    }

    #[test]
    fn run_jokers_bad_token_is_board_error() {
        let error = output(&jokers("BARON LUCKYCAR", false), "").unwrap_err();
        assert_eq!(
            error,
            CliError::Board(r#"jokers[1]: "LUCKYCAR": unknown joker"#.to_string())
        );
        assert_eq!(error.exit_code(), 1);
    }

    fn score_with_jokers(json: bool, line: &str) -> Command {
        Command::Score {
            file: Input::Stdin,
            json,
            all: false,
            seed: None,
            effects: None,
            jokers: Some(line.to_string()),
        }
    }

    #[test]
    fn score_jokers_flag_equals_map_form() {
        let command = score_with_jokers(true, FIXTURE_20_JOKERS);
        let trace: serde_json::Value =
            serde_json::from_str(&output(&command, FIXTURE_20_HAND).unwrap()).unwrap();
        let expected: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../fixtures/golden/20_shorthand_jokers.trace.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(trace, expected);
    }

    #[test]
    fn score_jokers_flag_replaces_file_jokers() {
        let command = score_with_jokers(false, "");
        let text = output(&command, "played: [AS, AH]\njokers: [JOLLY_JOKER]").unwrap();
        assert!(text.ends_with("Pair  32 × 2 = 64\n"), "{text}");
    }

    #[test]
    fn score_bad_jokers_flag_is_board_error() {
        let error = output(&score_with_jokers(true, "LUCKYCAR"), "played: [AS]").unwrap_err();
        assert_eq!(
            error,
            CliError::Board(r#"jokers[0]: "LUCKYCAR": unknown joker"#.to_string())
        );
    }

    #[test]
    fn link_jokers_flag_is_in_the_hash() {
        let command = Command::Link {
            file: Input::Stdin,
            seed: None,
            effects: None,
            jokers: Some("BARON+FOIL".to_string()),
            base: "http://x/".to_string(),
        };
        assert_eq!(
            output(&command, "played: [AS]").unwrap(),
            format!(
                "{}\n",
                link::board_link(
                    "http://x/",
                    r#"{"played":["AS"],"jokers":[{"ident":"BARON","edition":"Foil"}]}"#,
                    None,
                    None
                )
            )
        );
    }

    #[test]
    fn parse_jokers_flag_replaces_file_jokers() {
        let command = Command::Parse {
            file: Input::Stdin,
            jokers: Some("JOKER+OFF".to_string()),
        };
        assert_eq!(
            output(&command, "played: [AS]\njokers: [{ident: JOLLY_JOKER}]").unwrap(),
            "{\"played\":[\"AS\"],\"jokers\":[{\"ident\":\"JOKER\",\"debuffed\":true}]}\n"
        );
    }

    // ---- EPIC-09: --board and line ------------------------------------------

    const LINE_20: &str = "KS KH 2C / KD KC / BARON+FOIL LUCKYCATx3.25 JOKER+OFF";

    fn board(line: &str) -> Input {
        Input::Line(line.to_string())
    }

    #[test]
    fn score_board_flag_equals_file() {
        let by_flag = output(&score(board(LINE_20), true), "").unwrap();
        let by_file = output(&score(fixture("20_shorthand_jokers.yaml"), true), "").unwrap();
        assert_eq!(by_flag, by_file);
    }

    #[test]
    fn score_board_flag_with_jokers_flag_replaces() {
        let command = Command::Score {
            file: board("AS AH / / JOLLYJOKER"),
            json: false,
            all: false,
            seed: None,
            effects: None,
            jokers: Some(String::new()),
        };
        let text = output(&command, "").unwrap();
        assert!(text.ends_with("Pair  32 × 2 = 64\n"), "{text}");
    }

    #[test]
    fn link_board_flag_is_in_the_hash() {
        let command = Command::Link {
            file: board("AS / / BARON+FOIL"),
            seed: None,
            effects: None,
            jokers: None,
            base: "http://x/".to_string(),
        };
        assert_eq!(
            output(&command, "").unwrap(),
            format!(
                "{}\n",
                link::board_link(
                    "http://x/",
                    r#"{"played":["AS"],"jokers":[{"ident":"BARON","edition":"Foil"}]}"#,
                    None,
                    None
                )
            )
        );
    }

    #[test]
    fn score_bad_board_flag_is_board_error() {
        let error = output(&score(board("AS KX"), true), "").unwrap_err();
        assert_eq!(
            error,
            CliError::Board(r#"parse: played[1]: "KX": unknown card"#.to_string())
        );
        assert_eq!(error.exit_code(), 1);
    }

    #[test]
    fn run_line_prints_the_canonical_line() {
        let command = Command::Line {
            file: fixture("20_shorthand_jokers.yaml"),
        };
        assert_eq!(output(&command, "").unwrap(), format!("{LINE_20}\n"));
    }

    #[test]
    fn run_line_reads_any_form() {
        let command = Command::Line { file: Input::Stdin };
        assert_eq!(
            output(&command, "ks kh / / baron+foil").unwrap(),
            "KS KH / / BARON+FOIL\n"
        );
        assert_eq!(
            output(&command, r#"{"played": ["AS"], "money": 4}"#).unwrap(),
            "AS / / / money=4\n"
        );
    }

    #[test]
    fn run_line_no_line_is_board_error() {
        let command = Command::Line { file: Input::Stdin };
        let error = output(
            &command,
            "held:\n  - card: KH\n    enhancement: !Chips 50\n",
        )
        .unwrap_err();
        assert_eq!(
            error,
            CliError::Board("held[0]: KH: Chips(50) has no shorthand".to_string())
        );
    }
}
