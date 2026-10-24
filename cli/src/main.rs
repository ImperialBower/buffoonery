//! `buffoonery`: parses the arguments, calls [`buffoonery_cli::run`], and maps
//! the result to an exit code (0 success, 1 a bad board, 2 bad usage).

use buffoonery_cli::{Command, Input, run};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

/// Score Balatro-style boards with the buffoonery engine.
#[derive(Parser)]
#[command(name = "buffoonery", version = buffoonery::VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print the engine version.
    Version,
    /// Read a board (YAML, JSON or a board line) and print it as JSON.
    Parse {
        /// The board file; `-` or none reads stdin.
        file: Option<PathBuf>,
        /// The board itself instead of a file, e.g. "KS KH / KD / BARON+FOIL / FLUSH=2".
        #[arg(long, value_name = "LINE", conflicts_with = "file")]
        board: Option<String>,
        /// Replace the board's jokers with this line, e.g. "BARON+FOIL LUCKYCATx3.25".
        #[arg(long, value_name = "LINE")]
        jokers: Option<String>,
    },
    /// Explain a line of joker tokens, e.g. "BARON+FOIL LUCKYCATx3.25 JOKER+OFF".
    Jokers {
        /// The tokens, separated by spaces or commas.
        line: String,
        /// Print the jokers as JSON, in the board's map form.
        #[arg(long)]
        json: bool,
    },
    /// Read a board (YAML, JSON or a board line) and print its board line.
    Line {
        /// The board file; `-` or none reads stdin.
        file: Option<PathBuf>,
    },
    /// List the jokers.
    Catalog {
        /// Print the catalog as JSON, as the page reads it.
        #[arg(long)]
        json: bool,
        /// Keep only jokers whose ident or name contains this text (any case).
        #[arg(long)]
        find: Option<String>,
    },
    /// Score a board (YAML, JSON or a board line) and print its trace.
    Score {
        /// The board file; `-` or none reads stdin.
        file: Option<PathBuf>,
        /// The board itself instead of a file, e.g. "KS KH / KD / BARON+FOIL / FLUSH=2".
        #[arg(long, value_name = "LINE", conflicts_with = "file")]
        board: Option<String>,
        /// Print the trace as JSON, exactly as the component returns it.
        #[arg(long)]
        json: bool,
        /// Show every step, not only the ones the page shows.
        #[arg(long)]
        all: bool,
        /// Roll the chances (Lucky cards, Misprint) from this seed.
        #[arg(long)]
        seed: Option<u64>,
        /// Custom effects: a JSON file with the list the page puts in its link.
        #[arg(long, value_name = "FILE")]
        effects: Option<PathBuf>,
        /// Replace the board's jokers with this line, e.g. "BARON+FOIL LUCKYCATx3.25".
        #[arg(long, value_name = "LINE")]
        jokers: Option<String>,
    },
    /// Print a link that opens the board in the Joker Calculator page.
    Link {
        /// The board file; `-` or none reads stdin.
        file: Option<PathBuf>,
        /// The board itself instead of a file, e.g. "KS KH / KD / BARON+FOIL / FLUSH=2".
        #[arg(long, value_name = "LINE", conflicts_with = "file")]
        board: Option<String>,
        /// The seed the page scores with.
        #[arg(long)]
        seed: Option<u64>,
        /// Custom effects: a JSON file with the list the page puts in its link.
        #[arg(long, value_name = "FILE")]
        effects: Option<PathBuf>,
        /// Replace the board's jokers with this line, e.g. "BARON+FOIL LUCKYCATx3.25".
        #[arg(long, value_name = "LINE")]
        jokers: Option<String>,
        /// The page's address; `make dev` serves it at `http://localhost:5173/`.
        #[arg(long, default_value = buffoonery_cli::link::PAGE)]
        base: String,
    },
}

fn input(file: Option<PathBuf>) -> Input {
    match file {
        Some(path) if path.as_os_str() != "-" => Input::Path(path),
        _ => Input::Stdin,
    }
}

/// The board's source: `--board` when given (clap refuses it with a file),
/// else the file or stdin.
fn board_input(file: Option<PathBuf>, board: Option<String>) -> Input {
    board.map_or_else(|| input(file), Input::Line)
}

impl From<Commands> for Command {
    fn from(command: Commands) -> Self {
        match command {
            Commands::Version => Self::Version,
            Commands::Parse {
                file,
                board,
                jokers,
            } => Self::Parse {
                file: board_input(file, board),
                jokers,
            },
            Commands::Line { file } => Self::Line { file: input(file) },
            Commands::Jokers { line, json } => Self::Jokers { line, json },
            Commands::Catalog { json, find } => Self::Catalog { json, find },
            Commands::Score {
                file,
                board,
                json,
                all,
                seed,
                effects,
                jokers,
            } => Self::Score {
                file: board_input(file, board),
                json,
                all,
                seed,
                effects: effects.map(|path| input(Some(path))),
                jokers,
            },
            Commands::Link {
                file,
                board,
                seed,
                effects,
                jokers,
                base,
            } => Self::Link {
                file: board_input(file, board),
                seed,
                effects: effects.map(|path| input(Some(path))),
                jokers,
                base,
            },
        }
    }
}

fn main() -> ExitCode {
    let command = Command::from(Cli::parse().command);
    match run(
        &command,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("buffoonery: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}
