//! Open effect interpretation for the funky engine.
//!
//! Card effects are **data** (`MPip`), interpreted at scoring time. The built-in
//! effects are matched directly; a custom effect from a mod is identified by
//! `MPip::Custom(u32)` and resolved through an [`EffectRegistry`] the mod
//! populates at startup — so a mod can add scoring behaviour without editing
//! funky source.
//!
//! Because [`BuffoonCard`] is
//! `Copy`, `const`-constructible and `Serialize`, effects on cards cannot be
//! boxed trait objects. The indirection lives in the *registry* (keyed by a
//! plain, serializable `u32`) instead of on the card.

use crate::types::board::BuffoonBoard;
use crate::types::buffoon_card::BuffoonCard;
use crate::types::score::Score;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

/// A declarative description of an effect's contribution to a running [`Score`].
///
/// Effects return a `ScoreOp` rather than mutating a `Score` directly, so the
/// scoring pipeline stays in control of ordering (additive vs multiplicative)
/// and the operation is easy to test in isolation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ScoreOp {
    /// Contribute nothing.
    Nothing,
    /// Add flat chips.
    AddChips(usize),
    /// Add flat mult.
    AddMult(usize),
    /// Add both chips and mult (a whole [`Score`] contribution).
    Add(Score),
    /// Multiply the running mult by a factor (e.g. `2.0` for ×2).
    TimesMult(f64),
    /// Apply several operations in order.
    Seq(Vec<Self>),
}

impl ScoreOp {
    /// Folds this operation into `score`, returning the updated score.
    #[must_use]
    pub fn apply(&self, score: Score) -> Score {
        match self {
            Self::Nothing => score,
            Self::AddChips(chips) => score + Score::from_usize(*chips, 0),
            Self::AddMult(mult) => score + Score::from_usize(0, *mult),
            Self::Add(delta) => score + *delta,
            Self::TimesMult(factor) => score.multi_mult(*factor),
            Self::Seq(ops) => ops.iter().fold(score, |acc, op| op.apply(acc)),
        }
    }

    /// Whether this operation leaves every score unchanged: `Nothing`, `+0`,
    /// `×1`, or a `Seq` of those. A joker whose condition fails returns
    /// `Add(0 chips, 0 mult)` rather than `Nothing`, so a trace asks this, not
    /// `== Nothing`, to decide whether a step fired.
    #[must_use]
    // Exact compare on purpose: only a factor of exactly 1 is the identity.
    #[allow(clippy::float_cmp)]
    pub fn is_noop(&self) -> bool {
        match self {
            Self::Nothing => true,
            Self::AddChips(n) | Self::AddMult(n) => *n == 0,
            Self::Add(delta) => *delta == Score::default(),
            Self::TimesMult(factor) => *factor == 1.0,
            Self::Seq(ops) => ops.iter().all(Self::is_noop),
        }
    }
}

/// The breakdown-row text: `+4 Mult`, `+80 Chips`, `×3 Mult`. A no-op `Add`
/// and `Nothing` read `nothing`; a `Seq` joins its parts with `, `.
impl Display for ScoreOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nothing => write!(f, "nothing"),
            Self::AddChips(chips) => write!(f, "+{chips} Chips"),
            Self::AddMult(mult) => write!(f, "+{mult} Mult"),
            Self::Add(delta) => {
                let mut parts = Vec::new();
                if delta.chips > 0 {
                    parts.push(format!("+{} Chips", delta.chips));
                }
                if delta.mult.abs() > 0.0 {
                    parts.push(format!("+{} Mult", delta.mult));
                }
                if parts.is_empty() {
                    write!(f, "nothing")
                } else {
                    write!(f, "{}", parts.join(" "))
                }
            }
            Self::TimesMult(factor) => write!(f, "×{factor} Mult"),
            Self::Seq(ops) => {
                let parts: Vec<String> = ops.iter().map(ToString::to_string).collect();
                write!(f, "{}", parts.join(", "))
            }
        }
    }
}

/// Everything a custom [`Effect`] can read about the board while scoring: the
/// full board (played/held/joker piles, leveled hands) and the specific card or
/// joker that carries the effect.
pub struct ScoringContext<'a> {
    /// The board being scored.
    pub board: &'a BuffoonBoard,
    /// The card (or joker) whose effect is firing.
    pub source: BuffoonCard,
}

/// A custom, moddable scoring effect. Implement this in a mod crate and register
/// it under a `u32` id; attach the effect to a card via `MPip::Custom(id)`.
///
/// Kept object-safe so the registry can hold `Box<dyn Effect>`.
pub trait Effect {
    /// The contribution this effect makes given the current scoring context.
    fn score(&self, ctx: &ScoringContext<'_>) -> ScoreOp;
}

/// Maps `MPip::Custom` ids to their handlers. A mod builds one at startup and
/// passes it to [`BuffoonBoard::score_with_registry`].
///
/// [`BuffoonBoard::score_with_registry`]: crate::types::board::BuffoonBoard::score_with_registry
#[derive(Default)]
pub struct EffectRegistry {
    handlers: HashMap<u32, Box<dyn Effect>>,
}

impl EffectRegistry {
    /// A registry with no effects.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers `effect` under `id`, replacing any existing handler for it.
    pub fn register(&mut self, id: u32, effect: impl Effect + 'static) {
        self.handlers.insert(id, Box::new(effect));
    }

    /// Looks up the handler for `id`, if any.
    #[must_use]
    pub fn get(&self, id: u32) -> Option<&dyn Effect> {
        self.handlers.get(&id).map(|effect| &**effect)
    }

    /// Whether an effect is registered for `id`.
    #[must_use]
    pub fn contains(&self, id: u32) -> bool {
        self.handlers.contains_key(&id)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod funky__types__effect_tests {
    use super::*;

    fn every_score_op() -> ScoreOp {
        ScoreOp::Seq(vec![
            ScoreOp::Nothing,
            ScoreOp::AddChips(5),
            ScoreOp::AddMult(3),
            ScoreOp::Add(Score::new(5, 3.0)),
            ScoreOp::TimesMult(1.5),
        ])
    }

    #[test]
    fn score_op__serde_round_trip() {
        let op = every_score_op();
        let json = serde_json::to_string(&op).unwrap();

        assert_eq!(serde_json::from_str::<ScoreOp>(&json).unwrap(), op);
    }

    /// Pins the wire format: a consumer parsing this JSON breaks if it changes.
    #[test]
    fn score_op__serde_format() {
        assert_eq!(
            serde_json::to_string(&every_score_op()).unwrap(),
            r#"{"Seq":["Nothing",{"AddChips":5},{"AddMult":3},{"Add":{"chips":5,"mult":3.0}},{"TimesMult":1.5}]}"#
        );
    }

    #[test]
    fn score_op__apply() {
        let base = Score::new(10, 4.0);
        assert_eq!(ScoreOp::Nothing.apply(base), Score::new(10, 4.0));
        assert_eq!(ScoreOp::AddChips(5).apply(base), Score::new(15, 4.0));
        assert_eq!(ScoreOp::AddMult(3).apply(base), Score::new(10, 7.0));
        assert_eq!(
            ScoreOp::Add(Score::new(5, 3.0)).apply(base),
            Score::new(15, 7.0)
        );
        assert_eq!(ScoreOp::TimesMult(2.0).apply(base), Score::new(10, 8.0));
    }

    #[test]
    fn score_op__seq_applies_in_order() {
        // +2 mult then ×3 -> (4+2) * 3 = 18; the reverse would be 4*3 + 2 = 14.
        let op = ScoreOp::Seq(vec![ScoreOp::AddMult(2), ScoreOp::TimesMult(3.0)]);
        assert_eq!(op.apply(Score::new(0, 4.0)), Score::new(0, 18.0));
    }

    /// The calculator shows these strings as breakdown rows.
    #[test]
    fn score_op__display() {
        assert_eq!(ScoreOp::AddMult(4).to_string(), "+4 Mult");
        assert_eq!(ScoreOp::AddChips(80).to_string(), "+80 Chips");
        assert_eq!(ScoreOp::TimesMult(3.0).to_string(), "×3 Mult");
        assert_eq!(ScoreOp::TimesMult(1.5).to_string(), "×1.5 Mult");
        assert_eq!(
            ScoreOp::Add(Score::new(5, 3.0)).to_string(),
            "+5 Chips +3 Mult"
        );
        assert_eq!(ScoreOp::Add(Score::new(11, 0.0)).to_string(), "+11 Chips");
        assert_eq!(ScoreOp::Add(Score::new(0, 4.5)).to_string(), "+4.5 Mult");
        assert_eq!(ScoreOp::Add(Score::default()).to_string(), "nothing");
        assert_eq!(ScoreOp::Nothing.to_string(), "nothing");
        assert_eq!(
            ScoreOp::Seq(vec![ScoreOp::AddMult(2), ScoreOp::TimesMult(2.0)]).to_string(),
            "+2 Mult, ×2 Mult"
        );
    }

    /// A no-op leaves every score unchanged. A joker whose condition fails
    /// returns `Add(0 chips, 0 mult)`, not `Nothing`, so both must count.
    #[test]
    fn score_op__is_noop() {
        assert!(ScoreOp::Nothing.is_noop());
        assert!(ScoreOp::Add(Score::default()).is_noop());
        assert!(ScoreOp::AddChips(0).is_noop());
        assert!(ScoreOp::AddMult(0).is_noop());
        assert!(ScoreOp::TimesMult(1.0).is_noop());
        assert!(ScoreOp::Seq(vec![]).is_noop());
        assert!(ScoreOp::Seq(vec![ScoreOp::Nothing, ScoreOp::AddMult(0)]).is_noop());

        assert!(!ScoreOp::AddChips(1).is_noop());
        assert!(!ScoreOp::AddMult(1).is_noop());
        assert!(!ScoreOp::Add(Score::new(0, 0.5)).is_noop());
        assert!(!ScoreOp::TimesMult(1.5).is_noop());
        assert!(!ScoreOp::Seq(vec![ScoreOp::Nothing, ScoreOp::AddMult(2)]).is_noop());
    }

    struct FlatMult(usize);
    impl Effect for FlatMult {
        fn score(&self, _ctx: &ScoringContext<'_>) -> ScoreOp {
            ScoreOp::AddMult(self.0)
        }
    }

    #[test]
    fn registry__register_and_get() {
        let mut registry = EffectRegistry::new();
        assert!(!registry.contains(1));
        registry.register(1, FlatMult(7));
        assert!(registry.contains(1));
        assert!(registry.get(2).is_none());

        // The handler resolves; exercising it needs a ScoringContext, which the
        // board-level tests cover end to end.
        assert!(registry.get(1).is_some());
    }
}
