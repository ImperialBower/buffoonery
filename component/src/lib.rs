//! The buffoonery engine as a WebAssembly component (EPIC-05 Story 4).
//!
//! The component exports the `calculator` interface in `wit/world.wit`. This
//! crate is the adapter: it owns the formats (JSON and YAML in, JSON out) and
//! does no scoring of its own. The engine crate knows nothing about WIT.
//!
//! Build it with
//!
//! ```text
//! cargo build -p buffoonery-component --target wasm32-wasip2 --profile component
//! ```
//!
//! The five functions below are the exports. They are plain Rust, so the
//! native tests check them against `fixtures/golden/` without a wasm runtime;
//! the `js/` tests check the built component itself.

use buffoonery::prelude::*;

/// Scores a board: a [`BoardSpec`] as JSON or YAML in, its [`ScoreTrace`] as
/// JSON out. Input that starts with `{` is read as JSON, anything else as
/// YAML.
///
/// # Errors
///
/// The [`BoardSpecError`] message, which names the bad field: a parse error,
/// an unknown joker ident or card index, too many cards or jokers, or a bad
/// hand level.
///
/// # Examples
///
/// ```
/// let trace = buffoonery_component::score("played: [AS]").unwrap();
/// assert!(trace.contains(r#""hand_type":"HighCard""#));
///
/// let error = buffoonery_component::score("jokers: [{ident: NOPE}]").unwrap_err();
/// assert_eq!(error, r#"jokers[0]: unknown joker ident "NOPE""#);
/// ```
pub fn score(spec: &str) -> Result<String, String> {
    let spec = read_spec(spec).map_err(|e| e.to_string())?;
    let board = BuffoonBoard::try_from(&spec).map_err(|e| e.to_string())?;
    serde_json::to_string(&board.score_traced()).map_err(|e| e.to_string())
}

/// How [`score_with`] scores a board: an optional seed for the chances, and
/// the custom effects. The default scores like [`score`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScoreOptions {
    /// Rolls the chances (Lucky cards, Misprint) from this seed, so the same
    /// board and seed always give the same trace. `None` rolls nothing: each
    /// chance scores its floor.
    pub seed: Option<u64>,
    /// What each `MPip::Custom(id)` on the board does. An id with no entry
    /// does nothing.
    pub effects: Vec<CustomEffect>,
}

/// A custom effect: a card or joker with `MPip::Custom(id)` applies `op`, a
/// [`ScoreOp`] as JSON (e.g. `{"Seq": [{"AddChips": 20}, {"TimesMult": 2.0}]}`),
/// every time it scores. The op is fixed: it reads nothing from the board.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomEffect {
    /// The id in `MPip::Custom(id)`.
    pub id: u32,
    /// The [`ScoreOp`] as JSON.
    pub op: String,
}

/// An [`Effect`] that always returns the same [`ScoreOp`].
struct FixedOp(ScoreOp);

impl Effect for FixedOp {
    fn score(&self, _ctx: &ScoringContext<'_>) -> ScoreOp {
        self.0.clone()
    }
}

/// Like [`score`], with a seed for the chances and custom effects for
/// `MPip::Custom(id)` (see [`ScoreOptions`]). Rolled steps carry
/// `"rolled": true`.
///
/// # Errors
///
/// The same [`BoardSpecError`] messages as [`score`]; `effects[i].op: …` when
/// an op is not a [`ScoreOp`]; `effects[i]: id N is already defined` when two
/// effects share an id.
///
/// # Examples
///
/// ```
/// use buffoonery_component::{CustomEffect, ScoreOptions, score_with};
///
/// let board = "played: [AS]\njokers: [{ident: JOKER, enhancement: !Custom 1}]";
/// let options = ScoreOptions {
///     seed: Some(7),
///     effects: vec![CustomEffect { id: 1, op: r#"{"TimesMult": 3.0}"#.into() }],
/// };
/// let trace = score_with(board, &options).unwrap();
/// assert!(trace.contains(r#""op":{"TimesMult":3.0}"#));
/// ```
pub fn score_with(spec: &str, options: &ScoreOptions) -> Result<String, String> {
    let spec = read_spec(spec).map_err(|e| e.to_string())?;
    let board = BuffoonBoard::try_from(&spec).map_err(|e| e.to_string())?;
    let mut registry = EffectRegistry::new();
    for (i, effect) in options.effects.iter().enumerate() {
        let op: ScoreOp =
            serde_json::from_str(&effect.op).map_err(|e| format!("effects[{i}].op: {e}"))?;
        if registry.contains(effect.id) {
            return Err(format!("effects[{i}]: id {} is already defined", effect.id));
        }
        registry.register(effect.id, FixedOp(op));
    }
    let trace = match options.seed {
        Some(seed) => board.score_traced_with_seed_and_registry(seed, &registry),
        None => board.score_traced_with_registry(&registry),
    };
    serde_json::to_string(&trace).map_err(|e| e.to_string())
}

/// Reads a [`BoardSpec`] as JSON or YAML and writes it back as JSON, so a host
/// without a YAML parser can load a YAML board. It checks the shape only; an
/// unknown joker ident or card index is reported by [`score`].
///
/// # Errors
///
/// The [`BoardSpecError::Parse`] message when the text is not a spec.
///
/// # Examples
///
/// ```
/// let json = buffoonery_component::parse("played: [AS]").unwrap();
/// assert_eq!(json, r#"{"played":["AS"]}"#);
/// ```
pub fn parse(spec: &str) -> Result<String, String> {
    let spec = read_spec(spec).map_err(|e| e.to_string())?;
    serde_json::to_string(&spec).map_err(|e| e.to_string())
}

fn read_spec(spec: &str) -> Result<BoardSpec, BoardSpecError> {
    if spec.trim_start().starts_with('{') {
        serde_json::from_str(spec).map_err(|e| BoardSpecError::Parse(e.to_string()))
    } else {
        BoardSpec::from_yaml(spec)
    }
}

/// Every joker as a JSON array of [`JokerInfo`], in [`Joker::ALL_JOKERS`]
/// order.
///
/// # Examples
///
/// ```
/// let catalog = buffoonery_component::catalog();
/// assert!(catalog.contains(r#""ident":"JOLLY_JOKER""#));
/// ```
#[must_use]
pub fn catalog() -> String {
    // `JokerInfo` holds only strings, numbers and an `MPip`, so serializing it
    // cannot fail; the empty array is never reached.
    serde_json::to_string(&Joker::catalog()).unwrap_or_else(|_| String::from("[]"))
}

/// The version of the buffoonery engine that scores the boards.
///
/// # Examples
///
/// ```
/// assert_eq!(buffoonery_component::version(), buffoonery::VERSION);
/// ```
#[must_use]
pub fn version() -> String {
    buffoonery::VERSION.to_string()
}

#[cfg(target_arch = "wasm32")]
// The generated lift code rebuilds a `Vec` with length == capacity, on purpose.
#[allow(clippy::same_length_and_capacity)]
mod bindings {
    wit_bindgen::generate!({ world: "joker-calculator", path: "wit" });

    use exports::imperialbower::solver::calculator::ScoreOptions;

    struct Component;

    impl exports::imperialbower::solver::calculator::Guest for Component {
        fn score(spec: String) -> Result<String, String> {
            super::score(&spec)
        }

        fn score_with(spec: String, options: ScoreOptions) -> Result<String, String> {
            let options = super::ScoreOptions {
                seed: options.seed,
                effects: options
                    .effects
                    .into_iter()
                    .map(|effect| super::CustomEffect {
                        id: effect.id,
                        op: effect.op,
                    })
                    .collect(),
            };
            super::score_with(&spec, &options)
        }

        fn parse(spec: String) -> Result<String, String> {
            super::parse(&spec)
        }

        fn catalog() -> String {
            super::catalog()
        }

        fn version() -> String {
            super::version()
        }
    }

    export!(Component);
}
