use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign};

/// The running score of a hand: `chips` × `mult`.
///
/// `mult` is fractional, as in Balatro: ×1.5 on 3 mult is 4.5, and nothing
/// rounds between steps. `chips` is a whole number, as Balatro's chips always
/// are, and 64 bits wide so it does not overflow on wasm32, where `usize` is
/// 32 bits. Only [`Score::score`] rounds, once, down.
///
/// `Score` has no `Eq`, `Hash` or `Ord`: a float cannot derive them, and the
/// old derived `Ord` sorted by chips first, which never meant "higher score".
/// Compare [`Score::score`] values instead.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Score {
    pub chips: u64,
    pub mult: f64,
}

impl Score {
    #[must_use]
    pub fn new(chips: u64, mult: f64) -> Self {
        Self { chips, mult }
    }

    /// Builds a `Score` from the engine's whole-number counts. Card chips,
    /// hand levels and `+mult` values are `usize` throughout the engine; this
    /// is the one place they become a `Score`.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn from_usize(chips: usize, mult: usize) -> Self {
        Self {
            chips: chips as u64,
            mult: mult as f64,
        }
    }

    #[must_use]
    pub fn add_chips(&self, chips: u64) -> Self {
        Self {
            chips: self.chips.saturating_add(chips),
            mult: self.mult,
        }
    }

    #[must_use]
    pub fn add_mult(&mut self, mult: f64) -> Self {
        Self {
            chips: self.chips,
            mult: self.mult + mult,
        }
    }

    #[must_use]
    pub fn multi_mult(&self, mult: f64) -> Self {
        Self {
            chips: self.chips,
            mult: self.mult * mult,
        }
    }

    /// `chips × mult`, rounded down, as Balatro does. An `f64`, so late-game
    /// scores past `u64::MAX` still fit (Balatro uses floats too).
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn score(&self) -> f64 {
        (self.chips as f64 * self.mult).floor()
    }
}

impl Add for Score {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            chips: self.chips.saturating_add(other.chips),
            mult: self.mult + other.mult,
        }
    }
}

impl AddAssign for Score {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Score {{ {} chips times {} mult }} = {}",
            self.chips,
            self.mult,
            self.score()
        )
    }
}

#[cfg(test)]
#[allow(non_snake_case, unused_imports)]
mod funky__types__score_tests {
    use super::*;

    /// This is an example of a test that I would normally skip.
    #[test]
    fn new() {
        assert_eq!(Score::new(1, 2.0).score(), 2.0);
        assert_eq!(Score::new(2, 3.0).score(), 6.0);
    }

    #[test]
    fn add_chips() {
        assert_eq!(Score::new(1, 2.0).add_chips(1).score(), 4.0);
        assert_eq!(Score::new(2, 3.0).add_chips(1).score(), 9.0);
    }

    /// I love it when the AI suggests something that is basically wrong. The original
    /// second test had the result as 12.
    #[test]
    fn add_mult() {
        assert_eq!(Score::new(1, 2.0).add_mult(1.0).score(), 3.0);
        assert_eq!(Score::new(2, 3.0).add_mult(1.0).score(), 8.0);
    }

    #[test]
    fn multi_mult() {
        let original = Score::new(100, 10.0);
        let multi = original.multi_mult(1.5);

        let expected = Score::new(100, 15.0);

        assert_eq!(expected, multi);
        assert_eq!(1500.0, expected.score());
    }

    #[test]
    fn add() {
        let original = Score::new(2, 2.0);
        let expected = Score::new(4, 4.0);

        assert_eq!(original + original, expected);
    }

    #[test]
    fn default() {
        assert_eq!(Score::default().score(), 0.0);
    }

    #[test]
    fn display() {
        let score = Score::new(2, 2.0);
        assert_eq!(score.to_string(), "Score { 2 chips times 2 mult } = 4");
    }

    #[test]
    fn display__fractional_mult() {
        let score = Score::new(10, 4.5);
        assert_eq!(score.to_string(), "Score { 10 chips times 4.5 mult } = 45");
    }

    /// Balatro keeps fractional mult: ×1.5 on 3 mult is 4.5, not 5.
    #[test]
    fn multi_mult__keeps_the_fraction() {
        assert_eq!(Score::new(10, 3.0).multi_mult(1.5), Score::new(10, 4.5));
    }

    /// Two ×1.5 in a row is 3 × 2.25 = 6.75. Rounding up after each step
    /// would give 8.
    #[test]
    fn multi_mult__does_not_round_between_steps() {
        let twice = Score::new(10, 3.0).multi_mult(1.5).multi_mult(1.5);
        assert_eq!(twice.mult, 6.75);
    }

    /// The hand score is rounded down, as Balatro does.
    #[test]
    fn score__rounds_down() {
        assert_eq!(Score::new(10, 4.55).score(), 45.0);
        assert_eq!(Score::new(3, 6.75).score(), 20.0);
    }

    /// Late-game scores pass `u32::MAX` (4.29 × 10⁹), the size of `usize` on
    /// wasm32. They must not overflow.
    #[test]
    fn score__does_not_overflow() {
        let big = Score::new(5_000_000_000, 1_000.0);
        assert_eq!(big.score(), 5_000_000_000_000.0);
    }

    #[test]
    fn add_chips__does_not_overflow() {
        assert_eq!(Score::new(u64::MAX, 1.0).add_chips(1).chips, u64::MAX);
    }

    #[test]
    fn serde_format() {
        assert_eq!(
            serde_json::to_string(&Score::new(5, 3.0)).unwrap(),
            r#"{"chips":5,"mult":3.0}"#
        );
        assert_eq!(
            serde_json::to_string(&Score::new(5, 4.5)).unwrap(),
            r#"{"chips":5,"mult":4.5}"#
        );
    }

    #[test]
    fn serde_round_trip() {
        let score = Score::new(331, 34.25);
        let json = serde_json::to_string(&score).unwrap();
        assert_eq!(serde_json::from_str::<Score>(&json).unwrap(), score);
    }
}
