//! A whole board on one line (EPIC-09): `KS KH 2C / KD KC / BARON+FOIL`.
//!
//! A card token is a card index, then modifiers: `KS`, `KS+STEEL`,
//! `AH+GLASS+POLY`.
//!
//! ```text
//! card        = index { "+" modifier }      ; any case
//! index       = rank suit                   ; one of the 52 playing cards
//! modifier    = enhancement | edition       ; each at most once, any order
//! enhancement = BONUS MULT WILD GLASS STEEL STONE GOLD LUCKY
//! edition     = FOIL HOLO POLY NEG, or the long word
//! ```
//!
//! Each enhancement word is the fixed value its tarot puts on a card
//! (`STEEL` is The Chariot's `MultTimes1Dot(15)`). A card with another
//! enhancement has no token; [`CardSpec::shorthand`] says so instead of
//! rounding.
//!
//! ```
//! use buffoonery::prelude::*;
//!
//! let card = parse_card("ah+poly+glass").unwrap();
//! assert_eq!(card.shorthand().unwrap(), "AH+GLASS+POLY");
//! ```

use crate::decks::tarot::card::{EMPRESS, JUSTICE, LOVERS, MAGICIAN};
use crate::types::board_spec::CardSpec;
use crate::types::buffoon_card::{BCardType, BuffoonCard};
use crate::types::edition::Edition;
use crate::types::joker_shorthand::{ShorthandError, edition_word, parse_edition, set_once};
use crate::types::mpip::MPip;
use std::str::FromStr;

/// The eight card enhancements, by word. Read from the tarots and `MPip`'s
/// named values, so a token and a tarot cannot disagree.
const ENHANCEMENTS: [(&str, MPip); 8] = [
    ("BONUS", MPip::BONUS),
    ("MULT", EMPRESS.enhancement),
    ("WILD", LOVERS.enhancement),
    ("GLASS", JUSTICE.enhancement),
    ("STEEL", MPip::STEEL),
    ("STONE", MPip::TOWER),
    ("GOLD", MPip::DEVIL),
    ("LUCKY", MAGICIAN.enhancement),
];

/// Reads one card token into a [`CardSpec`]: `Index` when it has no
/// modifiers, `Detailed` when it has any. The index is written upper case.
///
/// # Errors
///
/// [`ShorthandError::Empty`] for an empty token; `UnknownCard` when the
/// index is not one of the 52 playing cards; `UnknownModifier` for a word
/// that is no enhancement or edition; `Duplicate` for a second enhancement
/// or edition.
///
/// # Examples
///
/// ```
/// use buffoonery::prelude::*;
///
/// let card = parse_card("KS+STEEL").unwrap();
/// assert_eq!(
///     card,
///     CardSpec::Detailed {
///         card: "KS".to_string(),
///         enhancement: Some(MPip::STEEL),
///         edition: None,
///     }
/// );
/// ```
pub fn parse_card(token: &str) -> Result<CardSpec, ShorthandError> {
    if token.is_empty() {
        return Err(ShorthandError::Empty);
    }
    let (index, modifiers) = token.split_at(token.find('+').unwrap_or(token.len()));
    let index = index.to_ascii_uppercase();
    if !is_playing_card(&index) {
        return Err(ShorthandError::UnknownCard {
            token: token.to_string(),
            index: token.split('+').next().unwrap_or_default().to_string(),
        });
    }
    if modifiers.is_empty() {
        return Ok(CardSpec::Index(index));
    }
    let mut enhancement = None;
    let mut edition = None;
    for word in modifiers.split('+').skip(1) {
        let duplicate = |what| ShorthandError::Duplicate {
            token: token.to_string(),
            what,
        };
        if let Some(value) = parse_enhancement(word) {
            set_once(&mut enhancement, value).ok_or_else(|| duplicate("enhancements"))?;
        } else if let Some(value) = parse_edition(word) {
            set_once(&mut edition, value).ok_or_else(|| duplicate("editions"))?;
        } else {
            return Err(ShorthandError::UnknownModifier {
                token: token.to_string(),
                modifier: format!("+{word}"),
            });
        }
    }
    Ok(CardSpec::Detailed {
        card: index,
        enhancement,
        edition,
    })
}

/// Reads one card token, as [`parse_card`] does.
///
/// ```
/// use buffoonery::prelude::*;
///
/// let card: CardSpec = "QS+FOIL".parse().unwrap();
/// assert_eq!(card.shorthand().unwrap(), "QS+FOIL");
/// ```
impl FromStr for CardSpec {
    type Err = ShorthandError;

    fn from_str(token: &str) -> Result<Self, Self::Err> {
        parse_card(token)
    }
}

impl CardSpec {
    /// The canonical token: the index, then the enhancement word, then the
    /// edition (short spelling). [`parse_card`] reads it back to an equal
    /// spec.
    ///
    /// # Errors
    ///
    /// [`ShorthandError::UnknownCard`] when the index is not one of the 52
    /// playing cards written upper case (`"ks"` would read back as `"KS"`);
    /// [`ShorthandError::NoShorthand`] when the enhancement is not one of the
    /// eight words, the edition is `Some(Edition::None)`, or a `Detailed`
    /// card sets neither (it would read back as `Index`).
    ///
    /// # Examples
    ///
    /// ```
    /// use buffoonery::prelude::*;
    ///
    /// let card = CardSpec::Detailed {
    ///     card: "AH".to_string(),
    ///     enhancement: Some(MPip::Glass(2, 4)),
    ///     edition: Some(Edition::Polychrome),
    /// };
    /// assert_eq!(card.shorthand().unwrap(), "AH+GLASS+POLY");
    /// ```
    pub fn shorthand(&self) -> Result<String, ShorthandError> {
        let (index, enhancement, edition) = match self {
            Self::Index(index) => (index, None, None),
            Self::Detailed {
                card,
                enhancement,
                edition,
            } => (card, *enhancement, *edition),
        };
        if !is_playing_card(index) {
            return Err(ShorthandError::UnknownCard {
                token: index.clone(),
                index: index.clone(),
            });
        }
        let no_shorthand = |value: String| ShorthandError::NoShorthand {
            ident: index.clone(),
            value,
        };
        if matches!(self, Self::Detailed { .. }) && enhancement.is_none() && edition.is_none() {
            return Err(no_shorthand(
                "a map with no enhancement or edition".to_string(),
            ));
        }
        let enhancement = match enhancement {
            None => String::new(),
            Some(value) => {
                let word =
                    enhancement_word(value).ok_or_else(|| no_shorthand(value.to_string()))?;
                format!("+{word}")
            }
        };
        let edition = match edition {
            None => String::new(),
            Some(Edition::None) => return Err(no_shorthand("edition None".to_string())),
            Some(edition) => format!("+{}", edition_word(edition)),
        };
        Ok(format!("{index}{enhancement}{edition}"))
    }
}

/// Whether `index` names one of the 52 playing cards, exactly as written.
/// `BuffoonCard::from_str` also reads tarots (`FOOL`, `1M`), the blank card
/// (`__`) and lower case; a card token takes none of them.
fn is_playing_card(index: &str) -> bool {
    index == index.to_ascii_uppercase()
        && index != "__"
        && BuffoonCard::from_str(index).is_ok_and(|card| card.card_type == BCardType::Basic)
}

/// The enhancement a word names, any case.
fn parse_enhancement(word: &str) -> Option<MPip> {
    ENHANCEMENTS
        .iter()
        .find(|(name, _)| word.eq_ignore_ascii_case(name))
        .map(|(_, value)| *value)
}

/// The word for an enhancement, if it is one of the eight.
fn enhancement_word(value: MPip) -> Option<&'static str> {
    ENHANCEMENTS
        .iter()
        .find(|(_, candidate)| *candidate == value)
        .map(|(name, _)| *name)
}

#[cfg(test)]
#[allow(non_snake_case)]
mod funky__types__board_line_tests {
    use super::*;
    use crate::decks::basic::card::KING_SPADES;
    use crate::decks::tarot::card::{
        DEVIL, EMPRESS, HIEROPHANT, JUSTICE, LOVERS, MAGICIAN, THE_CHARIOT, TOWER,
    };
    use crate::types::board_spec::CardSpec;
    use crate::types::edition::Edition;
    use crate::types::joker_shorthand::ShorthandError;
    use crate::types::mpip::MPip;

    fn index(card: &str) -> CardSpec {
        CardSpec::Index(card.to_string())
    }

    fn detailed(card: &str, enhancement: Option<MPip>, edition: Option<Edition>) -> CardSpec {
        CardSpec::Detailed {
            card: card.to_string(),
            enhancement,
            edition,
        }
    }

    const WORDS: [&str; 8] = [
        "BONUS", "MULT", "WILD", "GLASS", "STEEL", "STONE", "GOLD", "LUCKY",
    ];

    #[test]
    fn enhancements__are_the_tarot_values() {
        let tarots = [
            ("BONUS", HIEROPHANT),
            ("MULT", EMPRESS),
            ("WILD", LOVERS),
            ("GLASS", JUSTICE),
            ("STEEL", THE_CHARIOT),
            ("STONE", TOWER),
            ("GOLD", DEVIL),
            ("LUCKY", MAGICIAN),
        ];
        for (word, tarot) in tarots {
            let expected = KING_SPADES.enhance(tarot).enhancement;
            assert_eq!(
                parse_card(&format!("KS+{word}")),
                Ok(detailed("KS", Some(expected), None)),
                "{word}"
            );
        }
    }

    #[test]
    fn parse_card__index_alone_is_index() {
        assert_eq!(parse_card("KS"), Ok(index("KS")));
        assert_eq!(parse_card("TD"), Ok(index("TD")));
    }

    #[test]
    fn parse_card__case_ignored() {
        assert_eq!(parse_card("ks"), Ok(index("KS")));
        assert_eq!(
            parse_card("ks+steel+Foil"),
            Ok(detailed("KS", Some(MPip::STEEL), Some(Edition::Foil)))
        );
    }

    #[test]
    fn parse_card__edition_words() {
        let cases = [
            ("FOIL", Edition::Foil),
            ("HOLO", Edition::Holographic),
            ("HOLOGRAPHIC", Edition::Holographic),
            ("POLY", Edition::Polychrome),
            ("POLYCHROME", Edition::Polychrome),
            ("NEG", Edition::Negative),
            ("NEGATIVE", Edition::Negative),
        ];
        for (word, edition) in cases {
            assert_eq!(
                parse_card(&format!("QS+{word}")),
                Ok(detailed("QS", None, Some(edition))),
                "{word}"
            );
        }
    }

    #[test]
    fn parse_card__modifiers_in_any_order() {
        let expected = detailed("AH", Some(MPip::Glass(2, 4)), Some(Edition::Polychrome));
        assert_eq!(parse_card("AH+GLASS+POLY"), Ok(expected.clone()));
        assert_eq!(parse_card("AH+POLY+GLASS"), Ok(expected));
    }

    #[test]
    fn parse_card__unknown_index_is_an_error() {
        for token in ["KX", "K", "FOOL", "1M", "__", "KSS"] {
            assert_eq!(
                parse_card(token),
                Err(ShorthandError::UnknownCard {
                    token: token.to_string(),
                    index: token.to_string(),
                }),
                "{token}"
            );
        }
        assert_eq!(
            parse_card("KX+STEEL"),
            Err(ShorthandError::UnknownCard {
                token: "KX+STEEL".to_string(),
                index: "KX".to_string(),
            })
        );
    }

    #[test]
    fn parse_card__two_enhancements_is_duplicate() {
        assert_eq!(
            parse_card("KS+STEEL+GOLD"),
            Err(ShorthandError::Duplicate {
                token: "KS+STEEL+GOLD".to_string(),
                what: "enhancements",
            })
        );
    }

    #[test]
    fn parse_card__two_editions_is_duplicate() {
        assert_eq!(
            parse_card("KS+FOIL+NEG"),
            Err(ShorthandError::Duplicate {
                token: "KS+FOIL+NEG".to_string(),
                what: "editions",
            })
        );
    }

    #[test]
    fn parse_card__unknown_modifier_names_it() {
        for (token, modifier) in [
            ("KS+SHINY", "+SHINY"),
            ("KS+", "+"),
            ("KS+50c", "+50c"),
            ("KS+OFF", "+OFF"),
        ] {
            assert_eq!(
                parse_card(token),
                Err(ShorthandError::UnknownModifier {
                    token: token.to_string(),
                    modifier: modifier.to_string(),
                }),
                "{token}"
            );
        }
    }

    #[test]
    fn parse_card__empty_is_an_error() {
        assert_eq!(parse_card(""), Err(ShorthandError::Empty));
    }

    #[test]
    fn from_str__same_as_parse_card() {
        assert_eq!("KS+STEEL".parse::<CardSpec>(), parse_card("KS+STEEL"));
    }

    #[test]
    fn card_shorthand__canonical_form() {
        assert_eq!(index("KS").shorthand(), Ok("KS".to_string()));
        assert_eq!(
            detailed("AH", Some(MPip::Glass(2, 4)), Some(Edition::Polychrome)).shorthand(),
            Ok("AH+GLASS+POLY".to_string())
        );
        assert_eq!(
            detailed("QS", None, Some(Edition::Foil)).shorthand(),
            Ok("QS+FOIL".to_string())
        );
    }

    #[test]
    fn card_shorthand__round_trips_every_enhancement_and_edition() {
        let editions = [
            None,
            Some(Edition::Foil),
            Some(Edition::Holographic),
            Some(Edition::Polychrome),
            Some(Edition::Negative),
        ];
        let mut enhancements: Vec<Option<MPip>> = vec![None];
        for word in WORDS {
            let Ok(CardSpec::Detailed { enhancement, .. }) = parse_card(&format!("KS+{word}"))
            else {
                panic!("{word} did not parse");
            };
            enhancements.push(enhancement);
        }
        for enhancement in &enhancements {
            for edition in editions {
                if enhancement.is_none() && edition.is_none() {
                    continue;
                }
                let card = detailed("7D", *enhancement, edition);
                let token = card.shorthand().unwrap();
                assert_eq!(parse_card(&token), Ok(card), "{token}");
            }
        }
    }

    #[test]
    fn card_shorthand__other_enhancement_is_no_shorthand() {
        for (enhancement, value) in [
            (MPip::Chips(50), "Chips(50)"),
            (MPip::Custom(1), "Custom(1)"),
        ] {
            assert_eq!(
                detailed("KH", Some(enhancement), None).shorthand(),
                Err(ShorthandError::NoShorthand {
                    ident: "KH".to_string(),
                    value: value.to_string(),
                })
            );
        }
    }

    #[test]
    fn card_shorthand__edition_none_is_no_shorthand() {
        assert_eq!(
            detailed("KH", None, Some(Edition::None)).shorthand(),
            Err(ShorthandError::NoShorthand {
                ident: "KH".to_string(),
                value: "edition None".to_string(),
            })
        );
    }

    #[test]
    fn card_shorthand__empty_map_is_no_shorthand() {
        assert_eq!(
            detailed("KH", None, None).shorthand(),
            Err(ShorthandError::NoShorthand {
                ident: "KH".to_string(),
                value: "a map with no enhancement or edition".to_string(),
            })
        );
    }

    #[test]
    fn card_shorthand__an_index_not_written_exactly_is_unknown_card() {
        for card in ["ks", "KX", "FOOL"] {
            assert_eq!(
                index(card).shorthand(),
                Err(ShorthandError::UnknownCard {
                    token: card.to_string(),
                    index: card.to_string(),
                }),
                "{card}"
            );
        }
    }
}
