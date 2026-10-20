//! Golden fixtures shared across the boundary (EPIC-05 Story 5).
//!
//! Each `fixtures/golden/NN_name.yaml` is a [`BoardSpec`]; its
//! `NN_name.trace.json` is the [`ScoreTrace`] the engine must return for it.
//! The same pairs are read verbatim by the WIT component tests and the browser
//! parity test, so a change here is a change to the contract.
//!
//! After a deliberate scoring change, rewrite the expected traces with
//!
//! ```text
//! BLESS=1 cargo test --test golden
//! ```
//!
//! and review the JSON diff before committing it.
#![cfg(feature = "yaml")]
#![allow(non_snake_case, clippy::expect_used)]

use buffoonery::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/golden")
}

/// Every fixture spec, sorted by file name.
fn fixtures() -> Vec<PathBuf> {
    let mut specs: Vec<PathBuf> = fs::read_dir(golden_dir())
        .expect("fixtures/golden exists")
        .map(|entry| entry.expect("readable directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "yaml"))
        .collect();
    specs.sort();
    specs
}

fn trace_path(spec: &Path) -> PathBuf {
    spec.with_extension("trace.json")
}

fn traced(path: &Path) -> (BuffoonBoard, ScoreTrace) {
    let yaml = fs::read_to_string(path).expect("readable fixture");
    let spec = BoardSpec::from_yaml(&yaml).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let board = BuffoonBoard::try_from(&spec).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let trace = board.score_traced();
    (board, trace)
}

fn all_traces() -> Vec<(PathBuf, ScoreTrace)> {
    fixtures()
        .into_iter()
        .map(|spec| {
            let (_, trace) = traced(&spec);
            (spec, trace)
        })
        .collect()
}

#[test]
fn golden__set_size() {
    let count = fixtures().len();
    assert!(
        (10..=20).contains(&count),
        "Story 5 asked for 10-15 fixtures; the kicker and Ace-low fixes added two. Found {count}"
    );
}

#[test]
fn golden__traces_match() {
    let bless = std::env::var_os("BLESS").is_some();
    for spec in fixtures() {
        let (_, trace) = traced(&spec);
        let expected_path = trace_path(&spec);
        let actual = serde_json::to_value(&trace).expect("trace serializes");

        if bless {
            // From the trace, not `actual`: a `Value` sorts its keys, and the
            // struct's field order reads better in review.
            let mut json = serde_json::to_string_pretty(&trace).expect("trace serializes");
            json.push('\n');
            fs::write(&expected_path, json).expect("writable fixture");
            continue;
        }

        let text = fs::read_to_string(&expected_path).unwrap_or_else(|_| {
            panic!(
                "{} has no expected trace; run BLESS=1 cargo test --test golden",
                spec.display()
            )
        });
        let expected: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
        assert_eq!(actual, expected, "{}", spec.display());

        // The wire form reads back into the same trace.
        let read_back: ScoreTrace = serde_json::from_value(expected).expect("a ScoreTrace");
        assert_eq!(read_back, trace, "{}", spec.display());
    }
}

/// The trace and the untraced pipeline agree on every fixture.
#[test]
fn golden__trace_total_is_score() {
    for spec in fixtures() {
        let (board, trace) = traced(&spec);
        assert_eq!(trace.total, board.score(), "{}", spec.display());
    }
}

/// The set covers what Story 5 asks for. `ScoreOp::Seq` is the one variant no
/// fixture can reach: only a registered custom effect returns it, and a
/// `BoardSpec` cannot carry a registry.
#[test]
fn golden__coverage() {
    let traces = all_traces();
    let steps = || traces.iter().flat_map(|(_, trace)| trace.steps.iter());

    let has_op = |want: fn(&ScoreOp) -> bool| steps().any(|step| want(&step.op));
    assert!(has_op(|op| matches!(op, ScoreOp::Nothing)), "Nothing");
    assert!(
        has_op(|op| matches!(op, ScoreOp::AddChips(n) if *n > 0)),
        "AddChips"
    );
    assert!(
        has_op(|op| matches!(op, ScoreOp::AddMult(n) if *n > 0)),
        "AddMult"
    );
    assert!(
        has_op(|op| matches!(op, ScoreOp::Add(_)) && !op.is_noop()),
        "Add"
    );
    assert!(
        has_op(|op| matches!(op, ScoreOp::TimesMult(_)) && !op.is_noop()),
        "TimesMult"
    );

    assert!(
        steps()
            .any(|step| step.phase == Phase::Joker && !step.fired && step.part == StepPart::Effect),
        "a joker whose condition fails"
    );
    assert!(
        steps().any(|step| step.phase == Phase::Played && step.retrigger > 0),
        "a retriggered played card"
    );
    assert!(
        steps().any(|step| step.phase == Phase::Held && step.retrigger > 0),
        "a retriggered held card"
    );
    for phase in [Phase::Played, Phase::Joker] {
        assert!(
            steps().any(|step| step.phase == phase && step.part == StepPart::Edition && step.fired),
            "an edition that fires in {phase:?}"
        );
    }
    assert!(
        steps().any(|step| matches!(
            step.source,
            StepSource::Joker { card, .. } if Joker::blank_reason(&card).is_some()
        )),
        "a Blank joker"
    );
}
