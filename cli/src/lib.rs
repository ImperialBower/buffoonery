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
    Parse { file: Input },
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
    },
    /// Print a link that opens the board in the page.
    Link {
        file: Input,
        seed: Option<u64>,
        effects: Option<Input>,
        /// The page's address, e.g. [`link::PAGE`] or `make dev`'s.
        base: String,
    },
}

/// Where a board comes from: a file, or stdin (`-` or no file).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    Stdin,
    Path(PathBuf),
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
        Command::Parse { file } => {
            buffoonery_component::parse(&read_input(file, stdin)?).map_err(CliError::Board)?
        }
        Command::Catalog { json, find } => catalog(*json, find.as_deref()),
        Command::Score {
            file,
            json,
            all,
            seed,
            effects,
        } => {
            let (spec, effects) = read_board(file, effects.as_ref(), stdin)?;
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
            base,
        } => {
            let (spec, effects) = read_board(file, effects.as_ref(), stdin)?;
            // The page opens only a board it can score; so does the link.
            score(&spec, *seed, effects.clone())?;
            let spec_json = buffoonery_component::parse(&spec).map_err(CliError::Board)?;
            let effects_json = (!effects.is_empty()).then(|| page_effects(&effects));
            link::board_link(base, &spec_json, *seed, effects_json.as_deref())
        }
    };
    writeln!(out, "{text}").map_err(|e| CliError::Io(e.to_string()))
}

/// The board text and its custom effects. Only one of the two can come
/// from stdin.
fn read_board(
    file: &Input,
    effects: Option<&Input>,
    stdin: &mut dyn Read,
) -> Result<(String, Vec<buffoonery_component::CustomEffect>), CliError> {
    if *file == Input::Stdin && effects == Some(&Input::Stdin) {
        return Err(CliError::Io(
            "stdin can hold the board or the effects, not both".to_string(),
        ));
    }
    let spec = read_input(file, stdin)?;
    let effects = match effects {
        Some(input) => read_effects(&read_input(input, stdin)?)?,
        None => Vec::new(),
    };
    Ok((spec, effects))
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
        };
        let yaml = std::fs::read_to_string(match &command {
            Command::Parse {
                file: Input::Path(path),
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
        let command = Command::Parse { file: Input::Stdin };
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
        let command = Command::Parse { file: Input::Stdin };
        let error = output(&command, "nonsense: 1").unwrap_err();
        assert_eq!(
            error,
            CliError::Board(buffoonery_component::parse("nonsense: 1").unwrap_err())
        );
        assert_eq!(error.exit_code(), 1);
    }
}
