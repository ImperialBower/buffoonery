//! The adapter against the golden fixtures (EPIC-05 Stories 4 and 5): every
//! `fixtures/golden/` board, sent as YAML and as JSON, returns its expected
//! trace. The `js/` tests repeat this against the built component.
#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use buffoonery::prelude::*;
use buffoonery_component::{catalog, score, version};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn fixtures() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/golden");
    let mut specs: Vec<PathBuf> = fs::read_dir(dir)
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
