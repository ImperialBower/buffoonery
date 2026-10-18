//! # buffoonery
//!
//! A Balatro-style card engine and the foundation of a solver, built on
//! [cardpack](https://crates.io/crates/cardpack): jokers, planets, tarots and
//! spectral cards as data (`MPip` effects), a `BuffoonBoard` that scores a
//! played hand in four phases, plus the shop, vouchers, editions and blinds.
//!
//! Most code starts from the prelude:
//!
//! ```
//! use buffoonery::prelude::*;
//!
//! let hand = BuffoonPile::forgiving_from_str("AS KS QS JS TS");
//! assert_eq!(hand.determine_hand_type(), HandType::RoyalFlush);
//! ```
//!
//! The engine was copied from cardpack's `funky` module; see cardpack's
//! `docs/EPIC-06_Funky_Extraction.md`.
#![allow(clippy::struct_field_names)]
// Carried over from cardpack, which allows dead code crate-wide. Known sites:
// `BoardEvent::CardAdded`'s field (src/types/board.rs:34) and the private
// `BuffoonCard::get_enhanced_chips` (src/types/buffoon_card.rs:174).
#![allow(dead_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

pub mod decks;
pub mod macros;
pub mod prelude;
pub mod types;
