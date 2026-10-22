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
    /// Read a board (YAML or JSON) and print it as JSON.
    Parse {
        /// The board file; `-` or none reads stdin.
        file: Option<PathBuf>,
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
    /// List the jokers.
    Catalog {
        /// Print the catalog as JSON, as the page reads it.
        #[arg(long)]
        json: bool,
        /// Keep only jokers whose ident or name contains this text (any case).
        #[arg(long)]
        find: Option<String>,
    },
    /// Score a board (YAML or JSON) and print its trace.
    Score {
        /// The board file; `-` or none reads stdin.
        file: Option<PathBuf>,
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

impl From<Commands> for Command {
    fn from(command: Commands) -> Self {
        match command {
            Commands::Version => Self::Version,
            Commands::Parse { file, jokers } => Self::Parse {
                file: input(file),
                jokers,
            },
            Commands::Jokers { line, json } => Self::Jokers { line, json },
            Commands::Catalog { json, find } => Self::Catalog { json, find },
            Commands::Score {
                file,
                json,
                all,
                seed,
                effects,
                jokers,
            } => Self::Score {
                file: input(file),
                json,
                all,
                seed,
                effects: effects.map(|path| input(Some(path))),
                jokers,
            },
            Commands::Link {
                file,
                seed,
                effects,
                jokers,
                base,
            } => Self::Link {
                file: input(file),
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
