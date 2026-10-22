//! The adapter against the golden fixtures (EPIC-05 Stories 4 and 5): every
//! `fixtures/golden/` board, sent as YAML and as JSON, returns its expected
//! trace. The `js/` tests repeat this against the built component.
#![allow(non_snake_case, clippy::expect_used, clippy::unwrap_used)]

use buffoonery::prelude::*;
use buffoonery_component::{
    CustomEffect, ScoreOptions, catalog, parse, score, score_with, version,
};
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

fn seeded(seed: u64) -> ScoreOptions {
    ScoreOptions {
        seed: Some(seed),
        effects: Vec::new(),
    }
}

fn custom(id: u32, op: &str) -> CustomEffect {
    CustomEffect {
        id,
        op: op.to_string(),
    }
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

/// EPIC-08: a joker token reads in; `parse` writes the map form back.
#[test]
fn parse__token_jokers_become_the_map_form() {
    let json = parsed(&parse("jokers: [BARON+FOIL, LUCKYCATx3.25, JOKER+OFF]").unwrap());
    assert_eq!(
        json["jokers"],
        serde_json::json!([
            {"ident": "BARON", "edition": "Foil"},
            {"ident": "LUCKY_CAT", "enhancement": {"MultTimes2Dot": 325}},
            {"ident": "JOKER", "debuffed": true},
        ])
    );
}

#[test]
fn score__a_bad_token_names_the_token() {
    let error = score("jokers: [BARON, LUCKYCAR]").unwrap_err();
    assert!(error.starts_with("parse: "), "{error}");
    assert!(error.contains(r#""LUCKYCAR": unknown joker"#), "{error}");
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
fn score_with__seed_totals_match_the_engine() {
    for spec in fixtures() {
        let yaml = fs::read_to_string(&spec).expect("readable fixture");
        let board = BuffoonBoard::try_from(&BoardSpec::from_yaml(&yaml).unwrap()).unwrap();
        for seed in [0, 7, u64::MAX] {
            let trace = parsed(&score_with(&yaml, &seeded(seed)).unwrap());
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
fn score_with__a_seed_marks_rolled_steps() {
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
    assert_eq!(rolled(&parsed(&score_with(&yaml, &seeded(7)).unwrap())), 2);
    assert_eq!(
        score_with(&yaml, &seeded(7)),
        score_with(&yaml, &seeded(7)),
        "a seed repeats"
    );
}

#[test]
fn score_with__no_options_is_score() {
    for spec in fixtures() {
        let yaml = fs::read_to_string(&spec).expect("readable fixture");
        assert_eq!(
            score_with(&yaml, &ScoreOptions::default()),
            score(&yaml),
            "{}",
            spec.display()
        );
    }
}

#[test]
fn score_with__a_custom_effect_scores_its_op() {
    let board = "played: [AS]\njokers: [{ident: JOKER, enhancement: !Custom 3}]";
    let options = ScoreOptions {
        seed: None,
        effects: vec![custom(
            3,
            r#"{"Seq": [{"AddChips": 20}, {"TimesMult": 2.0}]}"#,
        )],
    };
    let trace = parsed(&score_with(board, &options).unwrap());
    let joker = trace["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|step| step["phase"] == "Joker" && step["part"] == "Effect")
        .unwrap();
    assert_eq!(
        joker["op"],
        serde_json::json!({"Seq": [{"AddChips": 20}, {"TimesMult": 2.0}]})
    );
    assert_eq!(joker["fired"], true);
    // High Card: 5 + 11 chips, 1 mult; then +20 chips, x2 mult.
    assert_eq!(
        trace["total"],
        serde_json::json!({"chips": 36, "mult": 2.0})
    );

    let unregistered = parsed(&score_with(board, &ScoreOptions::default()).unwrap());
    assert_eq!(
        unregistered["total"],
        serde_json::json!({"chips": 16, "mult": 1.0})
    );
}

#[test]
fn score_with__errors_name_the_field() {
    assert_eq!(
        score_with("played: [AS, ZZ]", &seeded(1)).unwrap_err(),
        r#"played[1]: unknown card "ZZ""#
    );
    let bad_op = ScoreOptions {
        seed: None,
        effects: vec![
            custom(1, r#"{"AddMult": 2}"#),
            custom(2, r#"{"AddChips": -5}"#),
        ],
    };
    let error = score_with("played: [AS]", &bad_op).unwrap_err();
    assert!(error.starts_with("effects[1].op: "), "{error}");
    let twice = ScoreOptions {
        seed: None,
        effects: vec![
            custom(1, r#"{"AddMult": 2}"#),
            custom(1, r#"{"AddMult": 3}"#),
        ],
    };
    assert_eq!(
        score_with("played: [AS]", &twice).unwrap_err(),
        "effects[1]: id 1 is already defined"
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
