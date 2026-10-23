//! A whole board on one line (EPIC-09): `KS KH 2C / KD KC / BARON+FOIL / FLUSH=2`.
//!
//! A board line is a third spelling of a [`BoardSpec`], after YAML and JSON.
//! Four sections, split by `/`, by place: played cards, held cards, jokers,
//! settings. A section may be empty; trailing sections may be left out.
//!
//! ```text
//! board       = section [ "/" section [ "/" section [ "/" section ] ] ]
//! section     = [ word { sep word } ]            ; sep: any run of spaces, tabs, commas
//!
//! played, held: card  = index { "+" modifier }   ; any case
//!               index = rank suit                ; one of the 52 playing cards
//!               modifier = enhancement | edition ; each at most once, any order
//!               enhancement = BONUS MULT WILD GLASS STEEL STONE GOLD LUCKY
//!               edition = FOIL HOLO POLY NEG, or the long word
//! jokers:       a joker token (see joker_shorthand)
//! settings:     HAND=level | money=[-]n | hands=n | discards=n | blind=BLIND
//!               HAND  = HIGHCARD PAIR TWOPAIR … FLUSHFIVE ; "_" ignored
//!               BLIND = SMALL BIG [THE]NEEDLE [THE]WATER [THE]MANACLE
//! ```
//!
//! Each enhancement word is the fixed value its tarot puts on a card
//! (`STEEL` is The Chariot's `MultTimes1Dot(15)`). Case is ignored on input.
//! [`BoardSpec::line`] writes the canonical form, and refuses a spec it
//! cannot write exactly instead of rounding.
//!
//! ```
//! use buffoonery::prelude::*;
//!
//! let spec: BoardSpec = "ks kh 2c / kd kc / baron+foil / flush=2 money=20".parse().unwrap();
//! assert_eq!(spec.money, Some(20));
//! assert_eq!(spec.line().unwrap(), "KS KH 2C / KD KC / BARON+FOIL / FLUSH=2 money=20");
//!
//! let card = parse_card("ah+poly+glass").unwrap();
//! assert_eq!(card.shorthand().unwrap(), "AH+GLASS+POLY");
//! ```

use crate::decks::tarot::card::{EMPRESS, JUSTICE, LOVERS, MAGICIAN};
use crate::types::blind::{Blind, BossBlind};
use crate::types::board_spec::{BoardSpec, CardSpec};
use crate::types::buffoon_card::{BCardType, BuffoonCard};
use crate::types::edition::Edition;
use crate::types::hands::HandType;
use crate::types::joker_shorthand::{
    LineError, ShorthandError, edition_word, parse_edition, parse_jokers, parse_whole, set_once,
    shorthand_line,
};
use crate::types::mpip::MPip;
use std::fmt::{Display, Formatter};
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

/// The sections of a board line, in place order.
const SECTIONS: [&str; 4] = ["played", "held", "jokers", "settings"];

/// Every hand type, in `HandType` order. With [`hand_word`]'s exhaustive
/// match, a new hand fails to compile or fails `hands__list_has_every_variant`.
const HANDS: [HandType; 13] = [
    HandType::HighCard,
    HandType::Pair,
    HandType::TwoPair,
    HandType::ThreeOfAKind,
    HandType::Straight,
    HandType::Flush,
    HandType::FullHouse,
    HandType::FourOfAKind,
    HandType::StraightFlush,
    HandType::RoyalFlush,
    HandType::FiveOfAKind,
    HandType::FlushHouse,
    HandType::FlushFive,
];

/// Every blind. With [`blind_word`]'s exhaustive match, a new boss fails to
/// compile or fails `blinds__every_blind_has_a_word`.
const BLINDS: [Blind; 5] = [
    Blind::Small,
    Blind::Big,
    Blind::Boss(BossBlind::TheNeedle),
    Blind::Boss(BossBlind::TheWater),
    Blind::Boss(BossBlind::TheManacle),
];

/// Why a board line could not be read, or a spec written as one.
///
/// ```
/// use buffoonery::prelude::*;
///
/// let error = "AS KX".parse::<BoardSpec>().unwrap_err();
/// assert_eq!(error.to_string(), r#"played[1]: "KX": unknown card"#);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoardLineError {
    /// More than four `/`-separated sections.
    TooManySections { found: usize },
    /// One token failed, with its section and position.
    Token(LineError),
}

impl Display for BoardLineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManySections { found } => {
                write!(f, "a board line has at most 4 sections; found {found}")
            }
            Self::Token(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for BoardLineError {}

/// Reads a board line, section by section.
///
/// ```
/// use buffoonery::prelude::*;
///
/// let spec: BoardSpec = "KS KH 2C / KD KC / BARON+FOIL LUCKYCATx3.25".parse().unwrap();
/// assert_eq!(spec.played.len(), 3);
/// assert_eq!(spec.held.len(), 2);
/// assert_eq!(spec.jokers[1].enhancement, Some(MPip::MultTimes2Dot(325)));
/// ```
impl FromStr for BoardSpec {
    type Err = BoardLineError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let sections: Vec<&str> = line.split('/').collect();
        if sections.len() > SECTIONS.len() {
            return Err(BoardLineError::TooManySections {
                found: sections.len(),
            });
        }
        let mut spec = Self::default();
        for (section, text) in SECTIONS.into_iter().zip(sections) {
            match section {
                "played" => spec.played = parse_cards(section, text)?,
                "held" => spec.held = parse_cards(section, text)?,
                "jokers" => spec.jokers = parse_jokers(text).map_err(BoardLineError::Token)?,
                _ => {
                    for (position, word) in words(text).enumerate() {
                        apply_setting(&mut spec, word).map_err(|error| {
                            BoardLineError::Token(LineError {
                                section,
                                position,
                                error,
                            })
                        })?;
                    }
                }
            }
        }
        Ok(spec)
    }
}

impl BoardSpec {
    /// The canonical board line: sections joined by ` / `, an empty middle
    /// section written as nothing (`AS / / JOKER`), trailing empty sections
    /// dropped. Settings in this order: hand levels in `HandType` order,
    /// then `money`, `hands`, `discards`, `blind`. Reading it back gives an
    /// equal spec.
    ///
    /// # Errors
    ///
    /// [`BoardLineError::Token`] for the first card or joker with no token
    /// form (see [`CardSpec::shorthand`] and [`JokerSpec::shorthand`]), with
    /// its section and position.
    ///
    /// [`JokerSpec::shorthand`]: crate::types::board_spec::JokerSpec::shorthand
    ///
    /// # Examples
    ///
    /// ```
    /// use buffoonery::prelude::*;
    ///
    /// let spec = BoardSpec {
    ///     played: vec![CardSpec::Index("AS".to_string())],
    ///     money: Some(20),
    ///     ..BoardSpec::default()
    /// };
    /// assert_eq!(spec.line().unwrap(), "AS / / / money=20");
    /// ```
    pub fn line(&self) -> Result<String, BoardLineError> {
        let mut parts = vec![
            cards_line("played", &self.played)?,
            cards_line("held", &self.held)?,
            shorthand_line(&self.jokers).map_err(BoardLineError::Token)?,
            self.settings_line(),
        ];
        while parts.last().is_some_and(String::is_empty) {
            parts.pop();
        }
        // Tokens hold no spaces, so collapsing runs only tidies empty sections.
        Ok(parts
            .join(" / ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "))
    }

    /// The settings section: every setting has a word, so it cannot fail.
    fn settings_line(&self) -> String {
        let mut words: Vec<String> = self
            .hand_levels
            .iter()
            .map(|(hand, level)| format!("{}={level}", hand_word(*hand)))
            .collect();
        if let Some(money) = self.money {
            words.push(format!("money={money}"));
        }
        if let Some(hands) = self.hands_left {
            words.push(format!("hands={hands}"));
        }
        if let Some(discards) = self.discards_left {
            words.push(format!("discards={discards}"));
        }
        if let Some(blind) = self.blind {
            words.push(format!("blind={}", blind_word(blind)));
        }
        words.join(" ")
    }
}

/// The words of one section: any run of spaces, tabs and commas separates.
fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| c.is_whitespace() || c == ',')
        .filter(|word| !word.is_empty())
}

/// Reads a played or held section.
fn parse_cards(section: &'static str, text: &str) -> Result<Vec<CardSpec>, BoardLineError> {
    words(text)
        .enumerate()
        .map(|(position, token)| {
            parse_card(token).map_err(|error| {
                BoardLineError::Token(LineError {
                    section,
                    position,
                    error,
                })
            })
        })
        .collect()
}

/// Writes a played or held section.
fn cards_line(section: &'static str, cards: &[CardSpec]) -> Result<String, BoardLineError> {
    let tokens = cards
        .iter()
        .enumerate()
        .map(|(position, card)| {
            card.shorthand().map_err(|error| {
                BoardLineError::Token(LineError {
                    section,
                    position,
                    error,
                })
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tokens.join(" "))
}

/// Lays one `key=value` setting onto `spec`, refusing a second of a key.
fn apply_setting(spec: &mut BoardSpec, word: &str) -> Result<(), ShorthandError> {
    let unknown = |key: &str| ShorthandError::UnknownSetting {
        token: word.to_string(),
        key: key.to_string(),
    };
    let Some((key, value)) = word.split_once('=') else {
        return Err(unknown(word));
    };
    let duplicate = |what| ShorthandError::Duplicate {
        token: word.to_string(),
        what,
    };
    let bad_number = || ShorthandError::BadNumber {
        token: word.to_string(),
        number: value.to_string(),
    };
    match name(key).as_str() {
        "MONEY" => {
            let money = parse_signed(value).ok_or_else(bad_number)?;
            set_once(&mut spec.money, money).ok_or_else(|| duplicate("money settings"))
        }
        "HANDS" => {
            let hands = parse_whole(value).ok_or_else(bad_number)?;
            set_once(&mut spec.hands_left, hands).ok_or_else(|| duplicate("hands settings"))
        }
        "DISCARDS" => {
            let discards = parse_whole(value).ok_or_else(bad_number)?;
            set_once(&mut spec.discards_left, discards)
                .ok_or_else(|| duplicate("discards settings"))
        }
        "BLIND" => {
            let blind = parse_blind(value).ok_or_else(|| ShorthandError::UnknownBlind {
                token: word.to_string(),
                blind: value.to_string(),
            })?;
            set_once(&mut spec.blind, blind).ok_or_else(|| duplicate("blind settings"))
        }
        hand_name => {
            let hand = HANDS
                .into_iter()
                .find(|hand| hand_word(*hand) == hand_name)
                .ok_or_else(|| unknown(key))?;
            let level = parse_whole(value).ok_or_else(bad_number)?;
            if spec.hand_levels.contains_key(&hand) {
                return Err(duplicate("levels for one hand"));
            }
            spec.hand_levels.insert(hand, level);
            Ok(())
        }
    }
}

/// A name as the grammar compares it: underscores dropped, upper case.
fn name(text: &str) -> String {
    text.replace('_', "").to_ascii_uppercase()
}

/// A whole number with an optional leading `-`.
fn parse_signed(number: &str) -> Option<isize> {
    match number.strip_prefix('-') {
        Some(digits) => isize::try_from(parse_whole(digits)?).ok()?.checked_neg(),
        None => isize::try_from(parse_whole(number)?).ok(),
    }
}

/// The blind a word names; a boss may have `THE` in front.
fn parse_blind(word: &str) -> Option<Blind> {
    let word = name(word);
    BLINDS.into_iter().find(|blind| {
        let canonical = blind_word(*blind);
        word == canonical || (blind.is_boss() && word == format!("THE{canonical}"))
    })
}

/// The canonical word for a hand: `FLUSH`, `TWOPAIR`.
fn hand_word(hand: HandType) -> &'static str {
    match hand {
        HandType::HighCard => "HIGHCARD",
        HandType::Pair => "PAIR",
        HandType::TwoPair => "TWOPAIR",
        HandType::ThreeOfAKind => "THREEOFAKIND",
        HandType::Straight => "STRAIGHT",
        HandType::Flush => "FLUSH",
        HandType::FullHouse => "FULLHOUSE",
        HandType::FourOfAKind => "FOUROFAKIND",
        HandType::StraightFlush => "STRAIGHTFLUSH",
        HandType::RoyalFlush => "ROYALFLUSH",
        HandType::FiveOfAKind => "FIVEOFAKIND",
        HandType::FlushHouse => "FLUSHHOUSE",
        HandType::FlushFive => "FLUSHFIVE",
    }
}

/// The canonical word for a blind: `SMALL`, `BIG`, `NEEDLE`, `WATER`,
/// `MANACLE`.
fn blind_word(blind: Blind) -> &'static str {
    match blind {
        Blind::Small => "SMALL",
        Blind::Big => "BIG",
        Blind::Boss(BossBlind::TheNeedle) => "NEEDLE",
        Blind::Boss(BossBlind::TheWater) => "WATER",
        Blind::Boss(BossBlind::TheManacle) => "MANACLE",
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod funky__types__board_line_tests {
    use super::*;
    use crate::decks::basic::card::KING_SPADES;
    use crate::decks::tarot::card::{
        DEVIL, EMPRESS, HIEROPHANT, JUSTICE, LOVERS, MAGICIAN, THE_CHARIOT, TOWER,
    };
    use crate::types::blind::{Blind, BossBlind};
    use crate::types::board_spec::{BoardSpec, CardSpec, JokerSpec};
    use crate::types::edition::Edition;
    use crate::types::hands::HandType;
    use crate::types::joker_shorthand::{LineError, ShorthandError};
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

    // ---- Settings -----------------------------------------------------------

    fn board(line: &str) -> Result<BoardSpec, BoardLineError> {
        line.parse()
    }

    fn settings(line: &str) -> Result<BoardSpec, BoardLineError> {
        board(&format!("/ / / {line}"))
    }

    fn setting_error(position: usize, error: ShorthandError) -> Result<BoardSpec, BoardLineError> {
        Err(BoardLineError::Token(LineError {
            section: "settings",
            position,
            error,
        }))
    }

    #[test]
    fn hands__list_has_every_variant() {
        assert_eq!(HANDS.len(), 13);
        let distinct: std::collections::BTreeSet<HandType> = HANDS.into_iter().collect();
        assert_eq!(distinct.len(), 13);
        for hand in HANDS {
            let word = hand_word(hand);
            let spec = settings(&format!("{word}=2")).unwrap();
            assert_eq!(spec.hand_levels.get(&hand), Some(&2), "{word}");
        }
    }

    #[test]
    fn blinds__every_blind_has_a_word() {
        let blinds = [
            (Blind::Small, "SMALL"),
            (Blind::Big, "BIG"),
            (Blind::Boss(BossBlind::TheNeedle), "NEEDLE"),
            (Blind::Boss(BossBlind::TheWater), "WATER"),
            (Blind::Boss(BossBlind::TheManacle), "MANACLE"),
        ];
        assert_eq!(BLINDS.len(), blinds.len());
        for (blind, word) in blinds {
            assert_eq!(blind_word(blind), word);
            assert_eq!(
                settings(&format!("blind={word}")).unwrap().blind,
                Some(blind)
            );
        }
    }

    #[test]
    fn settings__the_is_optional() {
        let water = Some(Blind::Boss(BossBlind::TheWater));
        assert_eq!(settings("blind=THEWATER").unwrap().blind, water);
        assert_eq!(settings("blind=WATER").unwrap().blind, water);
        assert_eq!(settings("blind=the_water").unwrap().blind, water);
    }

    #[test]
    fn settings__case_and_underscores_ignored() {
        let spec = settings("two_pair=2 Flush=3 MONEY=4 Hands=1").unwrap();
        assert_eq!(spec.hand_levels.get(&HandType::TwoPair), Some(&2));
        assert_eq!(spec.hand_levels.get(&HandType::Flush), Some(&3));
        assert_eq!(spec.money, Some(4));
        assert_eq!(spec.hands_left, Some(1));
    }

    #[test]
    fn settings__each_key_sets_its_field() {
        let spec = settings("FLUSH=2 money=20 hands=3 discards=2 blind=BIG").unwrap();
        assert_eq!(spec.hand_levels.get(&HandType::Flush), Some(&2));
        assert_eq!(spec.money, Some(20));
        assert_eq!(spec.hands_left, Some(3));
        assert_eq!(spec.discards_left, Some(2));
        assert_eq!(spec.blind, Some(Blind::Big));
    }

    #[test]
    fn settings__money_may_be_negative() {
        assert_eq!(settings("money=-5").unwrap().money, Some(-5));
    }

    #[test]
    fn settings__counts_may_not() {
        for (word, number) in [
            ("hands=-1", "-1"),
            ("discards=x", "x"),
            ("FLUSH=", ""),
            ("money=--5", "--5"),
        ] {
            assert_eq!(
                settings(word),
                setting_error(
                    0,
                    ShorthandError::BadNumber {
                        token: word.to_string(),
                        number: number.to_string(),
                    }
                ),
                "{word}"
            );
        }
    }

    #[test]
    fn settings__each_key_once() {
        let cases = [
            ("money=1 money=2", "money=2", "money settings"),
            ("hands=1 hands=2", "hands=2", "hands settings"),
            ("discards=1 discards=2", "discards=2", "discards settings"),
            ("blind=BIG blind=SMALL", "blind=SMALL", "blind settings"),
            ("FLUSH=2 flush=3", "flush=3", "levels for one hand"),
        ];
        for (line, token, what) in cases {
            assert_eq!(
                settings(line),
                setting_error(
                    1,
                    ShorthandError::Duplicate {
                        token: token.to_string(),
                        what,
                    }
                ),
                "{line}"
            );
        }
    }

    #[test]
    fn settings__unknown_key_and_blind_name_it() {
        for (word, key) in [
            ("speed=3", "speed"),
            ("FLSH=2", "FLSH"),
            ("$20", "$20"),
            ("FLUSH", "FLUSH"),
        ] {
            assert_eq!(
                settings(word),
                setting_error(
                    0,
                    ShorthandError::UnknownSetting {
                        token: word.to_string(),
                        key: key.to_string(),
                    }
                ),
                "{word}"
            );
        }
        assert_eq!(
            settings("blind=WALL"),
            setting_error(
                0,
                ShorthandError::UnknownBlind {
                    token: "blind=WALL".to_string(),
                    blind: "WALL".to_string(),
                }
            )
        );
    }

    // ---- The board line -----------------------------------------------------

    fn jokers(line: &str) -> Vec<JokerSpec> {
        crate::types::joker_shorthand::parse_jokers(line).unwrap()
    }

    #[test]
    fn line__sections_by_place() {
        assert_eq!(
            board("AS AD"),
            Ok(BoardSpec {
                played: vec![index("AS"), index("AD")],
                ..BoardSpec::default()
            })
        );
        assert_eq!(
            board("AS / KD"),
            Ok(BoardSpec {
                played: vec![index("AS")],
                held: vec![index("KD")],
                ..BoardSpec::default()
            })
        );
        assert_eq!(
            board("AS / KD / BARON"),
            Ok(BoardSpec {
                played: vec![index("AS")],
                held: vec![index("KD")],
                jokers: jokers("BARON"),
                ..BoardSpec::default()
            })
        );
        assert_eq!(
            board("AS / KD / BARON / money=3"),
            Ok(BoardSpec {
                played: vec![index("AS")],
                held: vec![index("KD")],
                jokers: jokers("BARON"),
                money: Some(3),
                ..BoardSpec::default()
            })
        );
    }

    #[test]
    fn line__empty_middle_section() {
        assert_eq!(
            board("AS / / JOKER"),
            Ok(BoardSpec {
                played: vec![index("AS")],
                jokers: jokers("JOKER"),
                ..BoardSpec::default()
            })
        );
        assert_eq!(
            board("AS//JOKER"),
            board("AS / / JOKER"),
            "spaces around / are optional"
        );
    }

    #[test]
    fn line__empty_is_the_default_spec() {
        assert_eq!(board(""), Ok(BoardSpec::default()));
        assert_eq!(board("  "), Ok(BoardSpec::default()));
        assert_eq!(board(" / / / "), Ok(BoardSpec::default()));
    }

    #[test]
    fn line__five_sections_is_an_error() {
        assert_eq!(
            board("AS / / / / money=1"),
            Err(BoardLineError::TooManySections { found: 5 })
        );
        assert_eq!(
            BoardLineError::TooManySections { found: 5 }.to_string(),
            "a board line has at most 4 sections; found 5"
        );
    }

    #[test]
    fn line__error_names_section_and_position() {
        let message = |line: &str| board(line).unwrap_err().to_string();
        assert_eq!(message("AS KX"), r#"played[1]: "KX": unknown card"#);
        assert_eq!(
            message("AS / KS+SHINY"),
            r#"held[0]: "KS+SHINY": unknown modifier "+SHINY""#
        );
        assert_eq!(
            message("/ / BARON JOKER LUCKYCAR"),
            r#"jokers[2]: "LUCKYCAR": unknown joker"#
        );
        assert_eq!(
            message("/ / / speed=3"),
            r#"settings[0]: "speed=3": unknown setting"#
        );
    }

    #[test]
    fn line__separators_are_any_run() {
        let expected = board("AS AD / KD / BARON JOKER / money=1 hands=2");
        assert_eq!(board("AS,,AD/KD/BARON , JOKER/money=1\thands=2"), expected);
        assert_eq!(
            board("\tAS  AD /KD/ BARON,JOKER / money=1,hands=2 "),
            expected
        );
    }

    #[test]
    fn line__canonical_form() {
        let fixture_20 = "KS KH 2C / KD KC / BARON+FOIL LUCKYCATx3.25 JOKER+OFF";
        assert_eq!(
            board(fixture_20).unwrap().line(),
            Ok(fixture_20.to_string())
        );
        let fixture_03 = "AH+BONUS JH+MULT 9H 6H 3H / / DROLLJOKER CRAFTYJOKER / FLUSH=2";
        assert_eq!(
            board(fixture_03).unwrap().line(),
            Ok(fixture_03.to_string())
        );
        assert_eq!(BoardSpec::default().line(), Ok(String::new()));
        assert_eq!(
            board("/ KD").unwrap().line(),
            Ok("/ KD".to_string()),
            "an empty first section"
        );
    }

    #[test]
    fn line__settings_in_canonical_order() {
        let spec = settings("blind=the_water discards=2 hands=3 money=-1 flush=2 pair=3").unwrap();
        assert_eq!(
            spec.line(),
            Ok("/ / / PAIR=3 FLUSH=2 money=-1 hands=3 discards=2 blind=WATER".to_string())
        );
    }

    #[test]
    fn line__trailing_empty_sections_dropped() {
        assert_eq!(board("AS / / /").unwrap().line(), Ok("AS".to_string()));
        assert_eq!(
            board("AS / KD / /").unwrap().line(),
            Ok("AS / KD".to_string())
        );
    }

    #[test]
    fn line__round_trips_every_field() {
        let line = "AS+STEEL+FOIL KS / 7D+WILD / BARON+HOLO LUCKYCATx3.25 / TWOPAIR=2 money=7 hands=1 discards=0 blind=SMALL";
        let spec = board(line).unwrap();
        assert_eq!(spec.line(), Ok(line.to_string()));
        assert_eq!(board(&spec.line().unwrap()), Ok(spec));
    }

    #[test]
    fn line__no_line_names_section_and_position() {
        let spec = BoardSpec {
            held: vec![detailed("KH", Some(MPip::Chips(50)), None)],
            ..BoardSpec::default()
        };
        assert_eq!(
            spec.line().unwrap_err().to_string(),
            "held[0]: KH: Chips(50) has no shorthand"
        );
        let spec = BoardSpec {
            jokers: vec![JokerSpec {
                enhancement: Some(MPip::Custom(1)),
                ..jokers("JOKER").remove(0)
            }],
            ..BoardSpec::default()
        };
        assert_eq!(
            spec.line().unwrap_err().to_string(),
            "jokers[0]: JOKER: Custom(1) has no shorthand"
        );
    }

    #[test]
    fn from_str__board_line_error_is_an_error() {
        let error: Box<dyn std::error::Error> =
            Box::new(BoardLineError::TooManySections { found: 6 });
        assert!(error.to_string().contains("found 6"));
    }
}
