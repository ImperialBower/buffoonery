//! `BoardSpec`: a board written as data (EPIC-05 Story 3).
//!
//! A `BoardSpec` describes a board in plain serde types, so the same board can
//! be written in Rust, JSON or YAML, and built with
//! `BuffoonBoard::try_from(spec)`. Jokers are named by their ident (the const
//! name, e.g. `"JOLLY_JOKER"`); cards by their index (`"AS"`). Every field is
//! optional. YAML loading is behind the `yaml` feature (on by default).
//!
//! ```yaml
//! hand_levels: { Flush: 2 }
//! played: [AS, KS, QS, JS, TS]
//! held:
//!   - card: KH
//!     enhancement: !MultTimes1Dot 15   # Steel
//! jokers:
//!   - ident: DROLL_JOKER
//!   - ident: JOKER
//!     edition: Polychrome
//! ```
//!
//! YAML sets data: any `MPip` and its numbers. A new kind of effect needs Rust
//! code in an [`EffectRegistry`](crate::types::effect::EffectRegistry).

use crate::decks::basic::Deck;
use crate::decks::joker::Joker;
use crate::types::blind::Blind;
use crate::types::board::BuffoonBoard;
use crate::types::buffoon_card::BuffoonCard;
use crate::types::buffoon_pile::BuffoonPile;
use crate::types::draws::Draws;
use crate::types::edition::Edition;
use crate::types::hands::HandType;
use crate::types::joker_shorthand::parse_joker;
use crate::types::mpip::MPip;
use serde::de::value::MapAccessDeserializer;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// The most cards a hand can play.
pub const MAX_PLAYED: usize = 5;

/// A board as data. Every field is optional; an empty spec is a fresh board
/// with 4 hands, 3 discards and no cards.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BoardSpec {
    /// Hand levels; a hand not listed is level 1.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub hand_levels: BTreeMap<HandType, usize>,
    /// The played cards, at most [`MAX_PLAYED`].
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub played: Vec<CardSpec>,
    /// The cards held in hand.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub held: Vec<CardSpec>,
    /// The jokers, left to right. Negative jokers take no slot.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub jokers: Vec<JokerSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub money: Option<isize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discards_left: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hands_left: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blind: Option<Blind>,
}

/// A played or held card: its index (`"AS"`), or its index with overrides.
///
/// Serializes as a plain string or a map. Deserializes by hand, not with
/// `#[serde(untagged)]`: untagged enums buffer their input, and a buffer
/// cannot carry a YAML tag such as `!MultTimes1Dot 15`.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum CardSpec {
    /// A plain card, e.g. `"AS"`.
    Index(String),
    /// A card with an enhancement and/or edition, e.g. a Steel King.
    Detailed {
        card: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        enhancement: Option<MPip>,
        #[serde(skip_serializing_if = "Option::is_none")]
        edition: Option<Edition>,
    },
}

/// The map form of a [`CardSpec`], read straight from the input.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CardDetail {
    card: String,
    #[serde(default)]
    enhancement: Option<MPip>,
    #[serde(default)]
    edition: Option<Edition>,
}

impl<'de> Deserialize<'de> for CardSpec {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct CardSpecVisitor;

        impl<'de> Visitor<'de> for CardSpecVisitor {
            type Value = CardSpec;

            fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(r#"a card index such as "AS", or a map with a `card` key"#)
            }

            fn visit_str<E: de::Error>(self, index: &str) -> Result<CardSpec, E> {
                Ok(CardSpec::Index(index.to_string()))
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<CardSpec, A::Error> {
                let detail = CardDetail::deserialize(MapAccessDeserializer::new(map))?;
                Ok(CardSpec::Detailed {
                    card: detail.card,
                    enhancement: detail.enhancement,
                    edition: detail.edition,
                })
            }
        }

        deserializer.deserialize_any(CardSpecVisitor)
    }
}

/// A joker by ident, with optional overrides. The override is the
/// calculator's "updatable effect": the joker keeps its identity and name.
///
/// Deserializes from a map of these fields or from a shorthand token such as
/// `"BARON+FOIL"` (see [`joker_shorthand`]); serializes as the map, always.
/// Hand-written for the same reason as [`CardSpec`]'s: an untagged enum
/// would lose the map form's YAML tags.
///
/// [`joker_shorthand`]: crate::types::joker_shorthand
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct JokerSpec {
    /// The joker const's name, e.g. `"JOLLY_JOKER"`.
    pub ident: String,
    /// Replaces the joker's effect; `None` keeps the catalog's.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhancement: Option<MPip>,
    /// The joker's edition; `None` keeps the catalog's (unedited).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<Edition>,
    /// A debuffed joker scores nothing.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub debuffed: bool,
}

/// The map form of a [`JokerSpec`], read straight from the input.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct JokerDetail {
    ident: String,
    #[serde(default)]
    enhancement: Option<MPip>,
    #[serde(default)]
    edition: Option<Edition>,
    #[serde(default)]
    debuffed: bool,
}

impl<'de> Deserialize<'de> for JokerSpec {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct JokerSpecVisitor;

        impl<'de> Visitor<'de> for JokerSpecVisitor {
            type Value = JokerSpec;

            fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(r#"a joker token such as "BARON+FOIL", or a map with an `ident` key"#)
            }

            fn visit_str<E: de::Error>(self, token: &str) -> Result<JokerSpec, E> {
                parse_joker(token).map_err(E::custom)
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<JokerSpec, A::Error> {
                let detail = JokerDetail::deserialize(MapAccessDeserializer::new(map))?;
                Ok(JokerSpec {
                    ident: detail.ident,
                    enhancement: detail.enhancement,
                    edition: detail.edition,
                    debuffed: detail.debuffed,
                })
            }
        }

        deserializer.deserialize_any(JokerSpecVisitor)
    }
}

/// Why a [`BoardSpec`] could not become a board. Each variant names the field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoardSpecError {
    /// `jokers[position]` names no joker.
    UnknownJoker { position: usize, ident: String },
    /// `played[position]` or `held[position]` is not a card index.
    UnknownCard {
        field: &'static str,
        position: usize,
        card: String,
    },
    /// More than [`MAX_PLAYED`] played cards.
    TooManyPlayed { count: usize, max: usize },
    /// More slot-taking jokers than the board has slots.
    TooManyJokers { count: usize, slots: usize },
    /// A level of 0, or a hand with no level of its own (Royal Flush).
    BadHandLevel { hand_type: HandType, level: usize },
    /// The text was not a valid spec (YAML syntax, or an unknown field).
    Parse(String),
}

impl Display for BoardSpecError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownJoker { position, ident } => {
                write!(f, "jokers[{position}]: unknown joker ident \"{ident}\"")
            }
            Self::UnknownCard {
                field,
                position,
                card,
            } => write!(f, "{field}[{position}]: unknown card \"{card}\""),
            Self::TooManyPlayed { count, max } => {
                write!(f, "played: {count} cards, at most {max} can be played")
            }
            Self::TooManyJokers { count, slots } => {
                write!(f, "jokers: {count} take a slot, the board has {slots}")
            }
            Self::BadHandLevel { hand_type, level } => {
                write!(f, "hand_levels: {hand_type:?} cannot be level {level}")
            }
            Self::Parse(message) => write!(f, "parse: {message}"),
        }
    }
}

impl std::error::Error for BoardSpecError {}

impl BoardSpec {
    /// Reads a spec from YAML.
    ///
    /// # Errors
    ///
    /// [`BoardSpecError::Parse`] if the text is not YAML or not a spec.
    #[cfg(feature = "yaml")]
    pub fn from_yaml(yaml: &str) -> Result<Self, BoardSpecError> {
        serde_norway::from_str(yaml).map_err(|e| BoardSpecError::Parse(e.to_string()))
    }

    /// Writes the spec as YAML.
    ///
    /// # Errors
    ///
    /// [`BoardSpecError::Parse`] if serialization fails.
    #[cfg(feature = "yaml")]
    pub fn to_yaml(&self) -> Result<String, BoardSpecError> {
        serde_norway::to_string(self).map_err(|e| BoardSpecError::Parse(e.to_string()))
    }
}

impl CardSpec {
    fn to_card(&self, field: &'static str, position: usize) -> Result<BuffoonCard, BoardSpecError> {
        let (index, enhancement, edition) = match self {
            Self::Index(index) => (index, None, None),
            Self::Detailed {
                card,
                enhancement,
                edition,
            } => (card, *enhancement, *edition),
        };
        let card = BuffoonCard::from_str(index).map_err(|_| BoardSpecError::UnknownCard {
            field,
            position,
            card: index.clone(),
        })?;
        Ok(BuffoonCard {
            enhancement: enhancement.unwrap_or(card.enhancement),
            edition: edition.unwrap_or(card.edition),
            ..card
        })
    }
}

impl JokerSpec {
    fn to_card(&self, position: usize) -> Result<BuffoonCard, BoardSpecError> {
        let joker = Joker::by_ident(&self.ident).ok_or_else(|| BoardSpecError::UnknownJoker {
            position,
            ident: self.ident.clone(),
        })?;
        Ok(BuffoonCard {
            enhancement: self.enhancement.unwrap_or(joker.enhancement),
            edition: self.edition.unwrap_or(joker.edition),
            debuffed: self.debuffed,
            ..joker
        })
    }
}

fn to_pile(cards: &[CardSpec], field: &'static str) -> Result<BuffoonPile, BoardSpecError> {
    cards
        .iter()
        .enumerate()
        .map(|(position, card)| card.to_card(field, position))
        .collect::<Result<Vec<_>, _>>()
        .map(BuffoonPile::from)
}

impl TryFrom<&BoardSpec> for BuffoonBoard {
    type Error = BoardSpecError;

    /// Builds the board: a fresh full deck, the spec's draws, levels, cards
    /// and jokers. Checks every field before returning.
    fn try_from(spec: &BoardSpec) -> Result<Self, Self::Error> {
        if spec.played.len() > MAX_PLAYED {
            return Err(BoardSpecError::TooManyPlayed {
                count: spec.played.len(),
                max: MAX_PLAYED,
            });
        }

        let draws = Draws::new(
            spec.hands_left.unwrap_or(4),
            spec.discards_left.unwrap_or(3),
        );
        let mut board = Self::new(draws, Deck::basic_buffoon_pile());

        for (&hand_type, &level) in &spec.hand_levels {
            if !board.poker_hands.set_level(hand_type, level) {
                return Err(BoardSpecError::BadHandLevel { hand_type, level });
            }
        }

        board.played = to_pile(&spec.played, "played")?;
        board.in_hand = to_pile(&spec.held, "held")?;

        let jokers = to_pile_of_jokers(&spec.jokers)?;
        let taken = Self::slots_taken(&jokers);
        if taken > board.joker_slots {
            return Err(BoardSpecError::TooManyJokers {
                count: taken,
                slots: board.joker_slots,
            });
        }
        for joker in &jokers {
            board.push_joker(*joker);
        }

        if let Some(money) = spec.money {
            board.money = money;
        }
        if let Some(blind) = spec.blind {
            board.blind = blind;
        }
        Ok(board)
    }
}

impl TryFrom<BoardSpec> for BuffoonBoard {
    type Error = BoardSpecError;

    fn try_from(spec: BoardSpec) -> Result<Self, Self::Error> {
        Self::try_from(&spec)
    }
}

fn to_pile_of_jokers(jokers: &[JokerSpec]) -> Result<BuffoonPile, BoardSpecError> {
    jokers
        .iter()
        .enumerate()
        .map(|(position, joker)| joker.to_card(position))
        .collect::<Result<Vec<_>, _>>()
        .map(BuffoonPile::from)
}

#[cfg(test)]
#[allow(non_snake_case)]
mod funky__types__board_spec_tests {
    use super::*;
    use crate::decks::basic::Deck;
    use crate::decks::basic::card as basic;
    use crate::decks::joker::{Joker, card};
    use crate::types::blind::BossBlind;
    use crate::types::buffoon_pile::BuffoonPile;
    use crate::types::draws::Draws;
    use crate::types::score::Score;
    use crate::types::trace::ScoreTrace;

    fn played(cards: &[&str]) -> Vec<CardSpec> {
        cards
            .iter()
            .map(|c| CardSpec::Index((*c).to_string()))
            .collect()
    }

    fn joker(ident: &str) -> JokerSpec {
        JokerSpec {
            ident: ident.to_string(),
            ..JokerSpec::default()
        }
    }

    /// The `buffoon` example's board, as a spec.
    fn buffoon_spec() -> BoardSpec {
        BoardSpec {
            played: played(&["AS", "KS", "QS", "JS", "TS"]),
            held: vec![CardSpec::Detailed {
                card: "KH".to_string(),
                enhancement: Some(MPip::STEEL),
                edition: None,
            }],
            jokers: vec![
                joker("CRAZY_JOKER"),
                joker("DROLL_JOKER"),
                joker("DEVIOUS_JOKER"),
                joker("CRAFTY_JOKER"),
            ],
            ..BoardSpec::default()
        }
    }

    /// The same board, built in Rust the way `examples/buffoon.rs` does.
    fn buffoon_board() -> BuffoonBoard {
        let mut board = BuffoonBoard::new(Draws::new(4, 3), Deck::basic_buffoon_pile());
        board.played = crate::bcards!("AS KS QS JS TS");
        board.in_hand = BuffoonPile::from(vec![BuffoonCard {
            enhancement: MPip::STEEL,
            ..basic::KING_HEARTS
        }]);
        for joker in [
            card::CRAZY_JOKER,
            card::DROLL_JOKER,
            card::DEVIOUS_JOKER,
            card::CRAFTY_JOKER,
        ] {
            board.push_joker(joker);
        }
        board
    }

    // ---- Building a board ----------------------------------------------------

    #[test]
    fn try_from__the_buffoon_board_scores_11254() {
        let board = BuffoonBoard::try_from(buffoon_spec()).unwrap();
        assert_eq!(board.score(), Score::new(331, 34.0));
        assert_eq!(board.score().score(), 11254.0);
    }

    #[test]
    fn try_from__matches_the_board_built_in_rust() {
        let from_spec = BuffoonBoard::try_from(buffoon_spec()).unwrap();
        assert_eq!(from_spec.score_traced(), buffoon_board().score_traced());
    }

    #[test]
    fn try_from__an_empty_spec_is_an_empty_board() {
        let board = BuffoonBoard::try_from(BoardSpec::default()).unwrap();
        assert!(board.played.is_empty());
        assert!(board.in_hand.is_empty());
        assert!(board.jokers.is_empty());
        assert_eq!(board.hands_remaining(), 4);
        assert_eq!(board.discards_remaining(), 3);
    }

    #[test]
    fn try_from__sets_levels_money_hands_discards_and_blind() {
        let spec = BoardSpec {
            hand_levels: [(HandType::Flush, 3)].into_iter().collect(),
            money: Some(10),
            hands_left: Some(2),
            discards_left: Some(0),
            blind: Some(Blind::Boss(BossBlind::TheNeedle)),
            ..BoardSpec::default()
        };
        let board = BuffoonBoard::try_from(spec).unwrap();
        assert_eq!(board.poker_hands.get(&HandType::Flush).unwrap().level, 3);
        assert_eq!(board.money, 10);
        assert_eq!(board.hands_remaining(), 2);
        assert_eq!(board.discards_remaining(), 0);
        assert_eq!(board.blind, Blind::Boss(BossBlind::TheNeedle));
    }

    /// The override is the calculator's "updatable effect": the joker keeps
    /// its identity, with a changed effect, edition or debuff.
    #[test]
    fn try_from__applies_joker_overrides() {
        let spec = BoardSpec {
            jokers: vec![JokerSpec {
                ident: "JOKER".to_string(),
                enhancement: Some(MPip::MultPlus(10)),
                edition: Some(Edition::Foil),
                debuffed: true,
            }],
            ..BoardSpec::default()
        };
        let board = BuffoonBoard::try_from(spec).unwrap();
        let joker = board.jokers.get(0).unwrap();
        assert_eq!(joker.enhancement, MPip::MultPlus(10));
        assert_eq!(joker.edition, Edition::Foil);
        assert!(joker.debuffed);
        assert_eq!(Joker::name(joker), Some("Joker"));
        assert_eq!(
            board.joker_state.len(),
            1,
            "push_joker keeps joker_state aligned"
        );
    }

    #[test]
    fn try_from__applies_card_overrides() {
        let spec = BoardSpec {
            played: vec![CardSpec::Detailed {
                card: "AS".to_string(),
                enhancement: Some(MPip::Glass(2, 4)),
                edition: Some(Edition::Polychrome),
            }],
            ..BoardSpec::default()
        };
        let board = BuffoonBoard::try_from(spec).unwrap();
        let ace = board.played.get(0).unwrap();
        assert_eq!(ace.enhancement, MPip::Glass(2, 4));
        assert_eq!(ace.edition, Edition::Polychrome);
    }

    // ---- Errors name the bad field -------------------------------------------

    #[test]
    fn try_from__unknown_joker_ident() {
        let spec = BoardSpec {
            jokers: vec![joker("JOKER"), joker("NOT_A_JOKER")],
            ..BoardSpec::default()
        };
        let err = BuffoonBoard::try_from(spec).unwrap_err();
        assert_eq!(
            err,
            BoardSpecError::UnknownJoker {
                position: 1,
                ident: "NOT_A_JOKER".to_string()
            }
        );
        assert_eq!(
            err.to_string(),
            r#"jokers[1]: unknown joker ident "NOT_A_JOKER""#
        );
    }

    #[test]
    fn try_from__unknown_card_index() {
        let spec = BoardSpec {
            held: played(&["KH", "XX"]),
            ..BoardSpec::default()
        };
        let err = BuffoonBoard::try_from(spec).unwrap_err();
        assert_eq!(
            err,
            BoardSpecError::UnknownCard {
                field: "held",
                position: 1,
                card: "XX".to_string()
            }
        );
        assert_eq!(err.to_string(), r#"held[1]: unknown card "XX""#);
    }

    #[test]
    fn try_from__more_than_five_played_cards() {
        let spec = BoardSpec {
            played: played(&["AS", "KS", "QS", "JS", "TS", "9S"]),
            ..BoardSpec::default()
        };
        assert_eq!(
            BuffoonBoard::try_from(spec).unwrap_err(),
            BoardSpecError::TooManyPlayed { count: 6, max: 5 }
        );
    }

    #[test]
    fn try_from__more_jokers_than_slots() {
        let spec = BoardSpec {
            jokers: vec![joker("JOKER"); 6],
            ..BoardSpec::default()
        };
        assert_eq!(
            BuffoonBoard::try_from(spec).unwrap_err(),
            BoardSpecError::TooManyJokers { count: 6, slots: 5 }
        );
    }

    /// A Negative joker takes no slot, so it is the sixth joker that fits.
    #[test]
    fn try_from__a_negative_joker_takes_no_slot() {
        let mut jokers = vec![joker("JOKER"); 5];
        jokers.push(JokerSpec {
            edition: Some(Edition::Negative),
            ..joker("JOKER")
        });
        let spec = BoardSpec {
            jokers,
            ..BoardSpec::default()
        };
        assert_eq!(BuffoonBoard::try_from(spec).unwrap().jokers.len(), 6);
    }

    #[test]
    fn try_from__bad_hand_level() {
        for (hand_type, level) in [(HandType::Pair, 0), (HandType::RoyalFlush, 2)] {
            let spec = BoardSpec {
                hand_levels: [(hand_type, level)].into_iter().collect(),
                ..BoardSpec::default()
            };
            assert_eq!(
                BuffoonBoard::try_from(spec).unwrap_err(),
                BoardSpecError::BadHandLevel { hand_type, level }
            );
        }
    }

    // ---- Serde ---------------------------------------------------------------

    #[test]
    fn json__every_field_is_optional() {
        assert_eq!(
            serde_json::from_str::<BoardSpec>("{}").unwrap(),
            BoardSpec::default()
        );
    }

    #[test]
    fn json__an_unknown_field_is_an_error() {
        assert!(serde_json::from_str::<BoardSpec>(r#"{"playd": ["AS"]}"#).is_err());
    }

    #[test]
    fn json__an_unknown_card_key_is_an_error() {
        assert!(
            serde_json::from_str::<BoardSpec>(r#"{"held": [{"card": "KH", "edtion": "Foil"}]}"#)
                .is_err()
        );
    }

    #[test]
    fn json__round_trip() {
        let spec = buffoon_spec();
        let json = serde_json::to_string(&spec).unwrap();
        assert_eq!(serde_json::from_str::<BoardSpec>(&json).unwrap(), spec);
    }

    /// A card is a plain index string unless it carries an override.
    #[test]
    fn json__card_shapes() {
        let spec = BoardSpec {
            played: played(&["AS"]),
            held: vec![CardSpec::Detailed {
                card: "KH".to_string(),
                enhancement: Some(MPip::STEEL),
                edition: None,
            }],
            ..BoardSpec::default()
        };
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains(r#""played":["AS"]"#), "{json}");
        assert!(
            json.contains(r#""held":[{"card":"KH","enhancement":{"MultTimes1Dot":15}}]"#),
            "{json}"
        );
    }

    /// EPIC-08: the three jokers of fixture 20, as specs.
    fn token_jokers() -> Vec<JokerSpec> {
        vec![
            JokerSpec {
                edition: Some(Edition::Foil),
                ..joker("BARON")
            },
            JokerSpec {
                enhancement: Some(MPip::MultTimes2Dot(325)),
                ..joker("LUCKY_CAT")
            },
            JokerSpec {
                debuffed: true,
                ..joker("JOKER")
            },
        ]
    }

    #[test]
    fn json__joker_as_token() {
        let spec: BoardSpec =
            serde_json::from_str(r#"{"jokers": ["BARON+FOIL", "LUCKYCATx3.25", "JOKER+OFF"]}"#)
                .unwrap();
        assert_eq!(spec.jokers, token_jokers());
    }

    #[test]
    fn json__joker_tokens_and_maps_mix() {
        let spec: BoardSpec =
            serde_json::from_str(r#"{"jokers": ["BARON", {"ident": "JOKER", "debuffed": true}]}"#)
                .unwrap();
        assert_eq!(
            spec.jokers,
            vec![
                joker("BARON"),
                JokerSpec {
                    debuffed: true,
                    ..joker("JOKER")
                }
            ]
        );
    }

    #[test]
    fn json__an_unknown_joker_key_is_an_error() {
        assert!(
            serde_json::from_str::<BoardSpec>(
                r#"{"jokers": [{"ident": "JOKER", "edtion": "Foil"}]}"#
            )
            .is_err()
        );
    }

    #[test]
    fn json__bad_token_names_the_token() {
        let error = serde_json::from_str::<BoardSpec>(r#"{"jokers": ["BARON", "LUCKYCAR"]}"#)
            .unwrap_err()
            .to_string();
        assert!(error.contains(r#""LUCKYCAR": unknown joker"#), "{error}");
    }

    /// A token is input only: output, and so every trace, `parse` result and
    /// page link, keeps the map form.
    #[test]
    fn json__joker_spec_still_serializes_as_a_map() {
        let spec = BoardSpec {
            jokers: token_jokers(),
            ..BoardSpec::default()
        };
        assert_eq!(
            serde_json::to_string(&spec).unwrap(),
            r#"{"jokers":[{"ident":"BARON","edition":"Foil"},{"ident":"LUCKY_CAT","enhancement":{"MultTimes2Dot":325}},{"ident":"JOKER","debuffed":true}]}"#
        );
    }

    /// EPIC-05 Story 3: spec → board → trace → JSON → parse gives the same trace.
    #[test]
    fn round_trip__spec_board_trace_json() {
        let trace = BuffoonBoard::try_from(buffoon_spec())
            .unwrap()
            .score_traced();
        let json = serde_json::to_string(&trace).unwrap();
        assert_eq!(serde_json::from_str::<ScoreTrace>(&json).unwrap(), trace);
    }

    // ---- YAML ----------------------------------------------------------------

    #[cfg(feature = "yaml")]
    const BUFFOON_YAML: &str = r"
played: [AS, KS, QS, JS, TS]
held:
  - card: KH
    enhancement: !MultTimes1Dot 15
jokers:
  - ident: CRAZY_JOKER
  - ident: DROLL_JOKER
  - ident: DEVIOUS_JOKER
  - ident: CRAFTY_JOKER
";

    /// The EPIC's definition of done: a board written as YAML and the same
    /// board built in Rust produce the same trace.
    #[cfg(feature = "yaml")]
    #[test]
    fn yaml__the_buffoon_board_matches_rust() {
        let spec = BoardSpec::from_yaml(BUFFOON_YAML).unwrap();
        assert_eq!(spec, buffoon_spec());
        let board = BuffoonBoard::try_from(spec).unwrap();
        assert_eq!(board.score_traced(), buffoon_board().score_traced());
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn yaml__joker_as_token() {
        let spec = BoardSpec::from_yaml("jokers: [BARON+FOIL, LUCKYCATx3.25, JOKER+OFF]").unwrap();
        assert_eq!(spec.jokers, token_jokers());
    }

    /// The map form keeps reading YAML tags, which an untagged enum would lose.
    #[cfg(feature = "yaml")]
    #[test]
    fn yaml__joker_map_form_still_reads() {
        let yaml = "
jokers:
  - ident: JOKER
    enhancement: !MultTimes1Dot 15
    edition: Polychrome
    debuffed: true
";
        assert_eq!(
            BoardSpec::from_yaml(yaml).unwrap().jokers,
            vec![JokerSpec {
                ident: "JOKER".to_string(),
                enhancement: Some(MPip::MultTimes1Dot(15)),
                edition: Some(Edition::Polychrome),
                debuffed: true,
            }]
        );
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn yaml__bad_token_is_a_parse_error_naming_the_token() {
        let Err(BoardSpecError::Parse(error)) =
            BoardSpec::from_yaml("played: [AS]\njokers: [BARON, LUCKYCAR]")
        else {
            panic!("expected a parse error");
        };
        assert!(error.contains(r#""LUCKYCAR": unknown joker"#), "{error}");
        assert!(error.contains("line 2"), "{error}");
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn yaml__round_trip() {
        let spec = BoardSpec {
            hand_levels: [(HandType::Flush, 2)].into_iter().collect(),
            money: Some(-5),
            blind: Some(Blind::Boss(BossBlind::TheNeedle)),
            ..buffoon_spec()
        };
        let yaml = spec.to_yaml().unwrap();
        assert_eq!(BoardSpec::from_yaml(&yaml).unwrap(), spec);
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn yaml__a_parse_error_is_a_board_spec_error() {
        assert!(matches!(
            BoardSpec::from_yaml("played: [AS"),
            Err(BoardSpecError::Parse(_))
        ));
        assert!(matches!(
            BoardSpec::from_yaml("playd: [AS]"),
            Err(BoardSpecError::Parse(_))
        ));
    }

    /// Pins how `MPip` and `Edition` look in YAML. Once boards are written as
    /// YAML this shape is a public contract (EPIC-05 open question 11).
    #[cfg(feature = "yaml")]
    #[test]
    fn yaml__mpip_and_edition_format() {
        let yaml = |value: &dyn erased::Yaml| value.to_yaml();
        assert_eq!(yaml(&MPip::MultPlus(4)), "!MultPlus 4\n");
        assert_eq!(yaml(&MPip::Lucky(5, 15)), "!Lucky\n- 5\n- 15\n");
        assert_eq!(yaml(&MPip::MultTimes2Dot(325)), "!MultTimes2Dot 325\n");
        assert_eq!(yaml(&MPip::Blank), "Blank\n");
        assert_eq!(yaml(&Edition::Polychrome), "Polychrome\n");
    }

    #[cfg(feature = "yaml")]
    mod erased {
        pub trait Yaml {
            fn to_yaml(&self) -> String;
        }
        impl<T: serde::Serialize> Yaml for T {
            fn to_yaml(&self) -> String {
                serde_norway::to_string(self).unwrap()
            }
        }
    }
}
