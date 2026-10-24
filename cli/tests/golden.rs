//! The CLI against the golden fixtures (EPIC-07): for every
//! `fixtures/golden/` board, `score --json` gives the fixture's trace, and the
//! table equals its snapshot in `fixtures/cli/`.
//!
//! After a deliberate change to the table or to scoring, rewrite the
//! snapshots with
//!
//! ```text
//! BLESS=1 cargo test -p buffoonery-cli --test golden
//! ```
//!
//! and review the diff before committing it.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use buffoonery_cli::{Command, Input, run};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures")
}

fn fixtures() -> Vec<PathBuf> {
    let mut specs: Vec<PathBuf> = fs::read_dir(root().join("golden"))
        .expect("fixtures/golden exists")
        .map(|entry| entry.expect("readable directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "yaml"))
        .collect();
    specs.sort();
    specs
}

fn score(spec: &Path, json: bool) -> String {
    let command = Command::Score {
        file: Input::Path(spec.to_path_buf()),
        json,
        all: false,
        seed: None,
        effects: None,
        jokers: None,
    };
    let mut out = Vec::new();
    run(&command, &mut std::io::empty(), &mut out)
        .unwrap_or_else(|e| panic!("{}: {e}", spec.display()));
    String::from_utf8(out).expect("UTF-8 output")
}

#[test]
fn score_json_equals_golden() {
    for spec in fixtures() {
        let text = score(&spec, true);
        assert!(
            text.ends_with('\n') && !text.ends_with("\n\n"),
            "{}",
            spec.display()
        );
        let actual: Value = serde_json::from_str(&text).expect("valid JSON");
        let expected: Value = serde_json::from_str(
            &fs::read_to_string(spec.with_extension("trace.json")).expect("expected trace"),
        )
        .expect("valid JSON");
        assert_eq!(actual, expected, "{}", spec.display());
    }
}

#[test]
fn table_matches_snapshot() {
    let bless = std::env::var_os("BLESS").is_some();
    for spec in fixtures() {
        let table = score(&spec, false);
        let name = spec.file_stem().expect("a file name");
        let snapshot = root().join("cli").join(name).with_extension("txt");
        if bless {
            fs::write(&snapshot, &table).expect("writable snapshot");
            continue;
        }
        let expected = fs::read_to_string(&snapshot).unwrap_or_else(|_| {
            panic!(
                "{} has no snapshot; run BLESS=1 cargo test -p buffoonery-cli --test golden",
                spec.display()
            )
        });
        assert_eq!(table, expected, "{}", snapshot.display());
    }
}

#[test]
fn every_snapshot_has_a_fixture() {
    let stems: Vec<_> = fixtures()
        .iter()
        .map(|spec| spec.file_stem().expect("a file name").to_owned())
        .collect();
    for entry in fs::read_dir(root().join("cli")).expect("fixtures/cli exists") {
        let path = entry.expect("readable directory entry").path();
        // `lines.txt` holds every fixture's board line (EPIC-09), not a table.
        if path.extension().is_some_and(|ext| ext == "txt") && !path.ends_with("lines.txt") {
            let stem = path.file_stem().expect("a file name").to_owned();
            assert!(stems.contains(&stem), "{} has no fixture", path.display());
        }
    }
}

fn line(spec: &Path) -> String {
    let command = Command::Line {
        file: Input::Path(spec.to_path_buf()),
    };
    let mut out = Vec::new();
    run(&command, &mut std::io::empty(), &mut out)
        .unwrap_or_else(|e| panic!("{}: {e}", spec.display()));
    String::from_utf8(out).expect("UTF-8 output")
}

/// EPIC-09: every fixture's board line, one row each, pinned in
/// `fixtures/cli/lines.txt`.
#[test]
fn lines_match_snapshot() {
    let lines = fixtures()
        .iter()
        .map(|spec| {
            let name = spec.file_stem().expect("a file name").to_string_lossy();
            format!("{name}: {}", line(spec))
        })
        .collect::<Vec<_>>()
        .concat();
    let snapshot = root().join("cli/lines.txt");
    if std::env::var_os("BLESS").is_some() {
        fs::write(&snapshot, &lines).expect("writable snapshot");
        return;
    }
    let expected = fs::read_to_string(&snapshot).unwrap_or_else(|_| {
        panic!("no lines.txt; run BLESS=1 cargo test -p buffoonery-cli --test golden")
    });
    assert_eq!(lines, expected, "{}", snapshot.display());
}

/// EPIC-09: every fixture, scored from `--board` with its line, gives its
/// trace.
#[test]
fn score_board_flag_equals_golden() {
    for spec in fixtures() {
        let command = Command::Score {
            file: Input::Line(line(spec.as_path()).trim_end().to_string()),
            json: true,
            all: false,
            seed: None,
            effects: None,
            jokers: None,
        };
        let mut out = Vec::new();
        run(&command, &mut std::io::empty(), &mut out)
            .unwrap_or_else(|e| panic!("{}: {e}", spec.display()));
        let actual: Value = serde_json::from_slice(&out).expect("valid JSON");
        let expected: Value = serde_json::from_str(
            &fs::read_to_string(spec.with_extension("trace.json")).expect("expected trace"),
        )
        .expect("valid JSON");
        assert_eq!(actual, expected, "{}", spec.display());
    }
}
