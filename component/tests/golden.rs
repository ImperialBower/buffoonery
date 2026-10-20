//! The adapter against the golden fixtures (EPIC-05 Stories 4 and 5): every
//! `fixtures/golden/` board, sent as YAML and as JSON, returns its expected
//! trace. The `js/` tests repeat this against the built component.
#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use buffoonery::prelude::*;
use buffoonery_component::{catalog, parse, score, score_seeded, version};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/golden")
}

fn fixtures() -> Vec<PathBuf> {
    let mut specs: Vec<PathBuf> = fs::read_dir(fixtures_dir())
        .expect("fixtures/golden exists")
        .map(|entry| entry.expect("readable directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "yaml"))
        .collect();
    specs.sort();
    specs
}

fn expected(spec: &Path) -> Value {
    let text = fs::read_to_string(spec.with_extension("trace.json")).expect("expected trace");
    serde_json::from_str(&text).expect("valid JSON")
}

fn parsed(trace: &str) -> Value {
    serde_json::from_str(trace).expect("score returns JSON")
}

#[test]
fn score__yaml_fixtures_match() {
    let specs = fixtures();
    assert!(specs.len() >= 10, "found {} fixtures", specs.len());
    for spec in specs {
        let yaml = fs::read_to_string(&spec).expect("readable fixture");
        let trace = score(&yaml).unwrap_or_else(|e| panic!("{}: {e}", spec.display()));
        assert_eq!(parsed(&trace), expected(&spec), "{}", spec.display());
    }
}

#[test]
fn score__json_fixtures_match() {
    for spec in fixtures() {
        let yaml = fs::read_to_string(&spec).expect("readable fixture");
        let json = serde_json::to_string(&BoardSpec::from_yaml(&yaml).unwrap()).unwrap();
        let trace = score(&json).unwrap_or_else(|e| panic!("{}: {e}", spec.display()));
        assert_eq!(parsed(&trace), expected(&spec), "{}", spec.display());
    }
}

#[test]
fn parse__yaml_fixtures_become_json_that_scores_the_same() {
    for spec in fixtures() {
        let yaml = fs::read_to_string(&spec).expect("readable fixture");
        let json = parse(&yaml).unwrap_or_else(|e| panic!("{}: {e}", spec.display()));
        assert!(json.starts_with('{'), "{}: {json}", spec.display());
        assert_eq!(parse(&json).unwrap(), json, "parse is idempotent");
        let trace = score(&json).unwrap_or_else(|e| panic!("{}: {e}", spec.display()));
        assert_eq!(parsed(&trace), expected(&spec), "{}", spec.display());
    }
}

#[test]
fn parse__errors_name_the_field() {
    assert!(parse("colour: red").unwrap_err().starts_with("parse: "));
}

#[test]
fn score__errors_name_the_field() {
    assert_eq!(
        score("played: [AS, ZZ]").unwrap_err(),
        r#"played[1]: unknown card "ZZ""#
    );
    assert_eq!(
        score(r#"{"jokers": [{"ident": "NOPE"}]}"#).unwrap_err(),
        r#"jokers[0]: unknown joker ident "NOPE""#
    );
    assert!(score("{not json").unwrap_err().starts_with("parse: "));
    assert!(score("colour: red").unwrap_err().starts_with("parse: "));
}

#[test]
fn score_seeded__totals_match_the_engine() {
    for spec in fixtures() {
        let yaml = fs::read_to_string(&spec).expect("readable fixture");
        let board = BuffoonBoard::try_from(&BoardSpec::from_yaml(&yaml).unwrap()).unwrap();
        for seed in [0, 7, u64::MAX] {
            let trace = parsed(&score_seeded(&yaml, seed).unwrap());
            let total: Score = serde_json::from_value(trace["total"].clone()).unwrap();
            assert_eq!(
                total,
                board.score_with_seed(seed),
                "{} seed {seed}",
                spec.display()
            );
        }
    }
}

#[test]
fn score_seeded__marks_rolled_steps() {
    let yaml = fs::read_to_string(fixtures_dir().join("11_inert_jokers.yaml")).unwrap();
    let rolled = |trace: &Value| {
        trace["steps"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|step| step["rolled"] == true)
            .count()
    };
    assert_eq!(rolled(&parsed(&score(&yaml).unwrap())), 0);
    // The Lucky card's special and Misprint's effect.
    assert_eq!(rolled(&parsed(&score_seeded(&yaml, 7).unwrap())), 2);
    assert_eq!(
        score_seeded(&yaml, 7),
        score_seeded(&yaml, 7),
        "a seed repeats"
    );
}

#[test]
fn score_seeded__errors_name_the_field() {
    assert_eq!(
        score_seeded("played: [AS, ZZ]", 1).unwrap_err(),
        r#"played[1]: unknown card "ZZ""#
    );
}

#[test]
fn catalog__lists_every_joker() {
    let catalog: Value = serde_json::from_str(&catalog()).unwrap();
    let jokers = catalog.as_array().unwrap();
    assert_eq!(jokers.len(), Joker::ALL_JOKERS.len());
    assert_eq!(jokers[0]["ident"], "JOKER");
}

#[test]
fn version__is_the_engine_version() {
    assert_eq!(version(), buffoonery::VERSION);
}
