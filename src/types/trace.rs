//! A step-by-step record of how a board's score was built (EPIC-05 Story 1).
//!
//! The scoring folds call [`ScoreOp::apply`] at every point that changes the
//! score. When asked for a trace they also push one [`ScoreStep`] per call, so
//! the trace explains the score without a second copy of the scoring logic:
//! replaying the steps' ops from an empty [`Score`] gives every step's `after`,
//! and the last `after` is the total.
//!
//! One step per op, not per card: a played card applies up to 3 ops per pass
//! (its effect, its special, its edition) and a joker 2 (effect, edition).
//! Steps whose op is a no-op are kept, with `fired: false`, because "this joker
//! did not fire" is a row the calculator shows.

use crate::types::buffoon_card::BuffoonCard;
use crate::types::effect::ScoreOp;
use crate::types::hands::HandType;
use crate::types::score::Score;
use serde::{Deserialize, Serialize};

/// The scoring phase a step belongs to, in the order they run.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Phase {
    /// The hand type's base chips and mult at its level.
    Base,
    /// The played cards, left to right.
    Played,
    /// The cards held in hand.
    Held,
    /// The jokers, left to right.
    Joker,
}

/// What produced a step.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum StepSource {
    /// The played hand's type and level (the base step).
    Hand { hand_type: HandType, level: usize },
    /// A played or held card, with its position in its pile.
    Card { card: BuffoonCard, index: usize },
    /// A joker, with its slot.
    Joker { card: BuffoonCard, index: usize },
}

/// Which part of the source produced a step.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum StepPart {
    /// The source's main effect: the base hand, a card's chips and built-in
    /// plus-effects, a held card's ability, or a joker's effect.
    Effect,
    /// A played card's special enhancement: Lucky, Glass, or a custom effect.
    Special,
    /// The card's or joker's edition: Foil, Holographic, Polychrome.
    Edition,
}

/// One application of a [`ScoreOp`] to the running score.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScoreStep {
    pub phase: Phase,
    pub source: StepSource,
    pub part: StepPart,
    /// 0 for the first pass, 1 and up for each retrigger.
    pub retrigger: usize,
    /// What was applied. A failed condition shows as a no-op.
    pub op: ScoreOp,
    /// Whether the op changed anything ([`ScoreOp::is_noop`] is false).
    pub fired: bool,
    /// Whether the op came from a random roll (Lucky, Misprint). Only a seeded
    /// or RNG-driven score rolls; the plain score never does.
    pub rolled: bool,
    /// The running score once this op was applied.
    pub after: Score,
}

/// Every step of one board's score, and the total.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScoreTrace {
    pub hand_type: HandType,
    pub steps: Vec<ScoreStep>,
    pub total: Score,
}

/// Where a step happens; everything about a [`ScoreStep`] except the op and
/// its result.
#[derive(Clone, Copy, Debug)]
pub(crate) struct StepAt {
    pub phase: Phase,
    pub source: StepSource,
    pub part: StepPart,
    pub retrigger: usize,
    pub rolled: bool,
}

/// Applies `op` to `score` and, when `trace` is present, records the step.
/// The folds call this at every point that changes the score, so tracing
/// cannot change a number.
pub(crate) fn apply_step(
    trace: &mut Option<&mut Vec<ScoreStep>>,
    score: Score,
    op: &ScoreOp,
    at: StepAt,
) -> Score {
    let after = op.apply(score);
    if let Some(steps) = trace {
        steps.push(ScoreStep {
            phase: at.phase,
            source: at.source,
            part: at.part,
            retrigger: at.retrigger,
            op: op.clone(),
            fired: !op.is_noop(),
            rolled: at.rolled,
            after,
        });
    }
    after
}

#[cfg(test)]
#[allow(non_snake_case)]
mod funky__types__trace_tests {
    use super::*;
    use crate::decks::joker::card;

    fn sample() -> ScoreTrace {
        let base = Score::new(35, 4.0);
        ScoreTrace {
            hand_type: HandType::Flush,
            steps: vec![
                ScoreStep {
                    phase: Phase::Base,
                    source: StepSource::Hand {
                        hand_type: HandType::Flush,
                        level: 1,
                    },
                    part: StepPart::Effect,
                    retrigger: 0,
                    op: ScoreOp::Add(base),
                    fired: true,
                    rolled: false,
                    after: base,
                },
                ScoreStep {
                    phase: Phase::Joker,
                    source: StepSource::Joker {
                        card: card::JOKER,
                        index: 0,
                    },
                    part: StepPart::Effect,
                    retrigger: 0,
                    op: ScoreOp::AddMult(4),
                    fired: true,
                    rolled: false,
                    after: Score::new(35, 8.0),
                },
            ],
            total: Score::new(35, 8.0),
        }
    }

    #[test]
    fn score_trace__serde_round_trip() {
        let trace = sample();
        let json = serde_json::to_string(&trace).unwrap();
        assert_eq!(serde_json::from_str::<ScoreTrace>(&json).unwrap(), trace);
    }

    /// Pins the shape of the small enums: the WIT boundary and the calculator
    /// read these strings.
    #[test]
    fn trace_enums__serde_format() {
        assert_eq!(serde_json::to_string(&Phase::Joker).unwrap(), r#""Joker""#);
        assert_eq!(
            serde_json::to_string(&StepPart::Edition).unwrap(),
            r#""Edition""#
        );
        assert_eq!(
            serde_json::to_string(&StepSource::Hand {
                hand_type: HandType::Pair,
                level: 2
            })
            .unwrap(),
            r#"{"Hand":{"hand_type":"Pair","level":2}}"#
        );
    }

    #[test]
    fn apply_step__records_only_with_a_trace() {
        let at = StepAt {
            phase: Phase::Joker,
            source: StepSource::Joker {
                card: card::JOKER,
                index: 0,
            },
            part: StepPart::Effect,
            retrigger: 0,
            rolled: false,
        };
        let op = ScoreOp::AddMult(4);

        let mut none: Option<&mut Vec<ScoreStep>> = None;
        assert_eq!(
            apply_step(&mut none, Score::new(10, 1.0), &op, at),
            Score::new(10, 5.0)
        );

        let mut steps = Vec::new();
        let mut some = Some(&mut steps);
        let after = apply_step(&mut some, Score::new(10, 1.0), &op, at);
        assert_eq!(after, Score::new(10, 5.0));
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].after, after);
        assert!(steps[0].fired);
    }
}
