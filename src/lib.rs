//! # buffoonery
//!
//! A Balatro-style card engine and the foundation of a solver, built on
//! [cardpack](https://crates.io/crates/cardpack).
//!
//! This is the scaffold. The engine arrives from cardpack's `funky` module
//! (see cardpack's `docs/EPIC-06_Funky_Extraction.md`).

/// Adds two numbers. Placeholder until the engine lands.
///
/// # Examples
///
/// ```
/// assert_eq!(buffoonery::add(2, 2), 4);
/// ```
#[must_use]
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_two_numbers() {
        assert_eq!(add(2, 2), 4);
    }
}
