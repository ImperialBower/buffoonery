//! One token per joker: `BARON+FOIL`, `LUCKYCATx3.25`, `JOKER+50c+OFF`
//! (EPIC-08).
//!
//! A token is a second spelling of a [`JokerSpec`], the way `AS` is a
//! spelling of a card. A line of tokens is a joker row, left to right.
//!
//! ```text
//! line     = token { separator token }        ; separator: spaces, tabs, commas
//! token    = ident { modifier }
//! ident    = letter { letter | digit | "_" }  ; any case; "_" is ignored
//! modifier = "+" edition                      ; FOIL HOLO POLY NEG, or the long word
//!          | "+" integer "c"                  ; chips override: MPip::Chips
//!          | "+" integer "m"                  ; +mult override: MPip::MultPlus
//!          | "x" number                       ; ×mult override: MultTimes, 1Dot, 2Dot
//!          | "+OFF"                           ; debuffed
//! number   = integer [ "." digit [ digit ] ]
//! ```
//!
//! Case is ignored on input. Modifiers come in any order; each kind at most
//! once. The decimal count picks the variant (`x3` → `MultTimes(3)`, `x3.5` →
//! `MultTimes1Dot(35)`, `x3.25` → `MultTimes2Dot(325)`), and
//! [`JokerSpec::shorthand`] writes it back the same way, so nothing is
//! rounded or dropped.
//!
//! ```
//! use buffoonery::prelude::*;
//!
//! let jokers = parse_jokers("BARON+FOIL LUCKYCATx3.25").unwrap();
//! assert_eq!(jokers[0].ident, "BARON");
//! assert_eq!(jokers[0].edition, Some(Edition::Foil));
//! assert_eq!(jokers[1].enhancement, Some(MPip::MultTimes2Dot(325)));
//! assert_eq!(shorthand_line(&jokers).unwrap(), "BARON+FOIL LUCKYCATx3.25");
//! ```

use crate::decks::joker::Joker;
use crate::types::board_spec::JokerSpec;
use crate::types::edition::Edition;
use crate::types::mpip::MPip;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Why one token could not become a [`JokerSpec`], or a spec a token.
///
/// No position: a token does not know where it sits in a line; [`LineError`]
/// adds that. Every variant but `Empty` and `NoShorthand` carries the token,
/// and `Display` quotes it first: `"LUCKYCAR": unknown joker`.
///
/// ```
/// use buffoonery::prelude::*;
///
/// let error = parse_joker("BARON+FOIL+HOLO").unwrap_err();
/// assert_eq!(error.to_string(), r#""BARON+FOIL+HOLO": two editions"#);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShorthandError {
    /// The token was empty.
    Empty,
    /// No catalog ident matches `ident`, with underscores dropped and case
    /// ignored (or, from [`JokerSpec::shorthand`], exactly).
    UnknownJoker { token: String, ident: String },
    /// `modifier` is none of the grammar's forms.
    UnknownModifier { token: String, modifier: String },
    /// A second modifier of one kind: `what` is `"editions"`, `"overrides"`
    /// or `"+OFFs"`.
    Duplicate { token: String, what: &'static str },
    /// `number` is not a whole number where one is needed, is negative, or
    /// has more than two decimals.
    BadNumber { token: String, number: String },
    /// From [`JokerSpec::shorthand`]: `value` (an override such as
    /// `Lucky(5, 15)`, or `edition None`) has no token form.
    NoShorthand { ident: String, value: String },
}

impl Display for ShorthandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "empty joker token"),
            Self::UnknownJoker { token, .. } => write!(f, "\"{token}\": unknown joker"),
            Self::UnknownModifier { token, modifier } => {
                write!(f, "\"{token}\": unknown modifier \"{modifier}\"")
            }
            Self::Duplicate { token, what } => write!(f, "\"{token}\": two {what}"),
            Self::BadNumber { token, number } => write!(f, "\"{token}\": bad number \"{number}\""),
            Self::NoShorthand { ident, value } => write!(f, "{ident}: {value} has no shorthand"),
        }
    }
}

impl std::error::Error for ShorthandError {}

/// A [`ShorthandError`] with its place in a line, counted from 0:
/// `jokers[2]: "LUCKYCAR": unknown joker`.
///
/// ```
/// use buffoonery::prelude::*;
///
/// let error = parse_jokers("BARON LUCKYCAR").unwrap_err();
/// assert_eq!(error.position, 1);
/// assert_eq!(error.to_string(), r#"jokers[1]: "LUCKYCAR": unknown joker"#);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineError {
    /// The failing token's index in the line.
    pub position: usize,
    /// Why it failed.
    pub error: ShorthandError,
}

impl Display for LineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "jokers[{}]: {}", self.position, self.error)
    }
}

impl std::error::Error for LineError {}

/// Reads one token into a [`JokerSpec`] whose `ident` is the catalog's
/// (`luckycat` → `LUCKY_CAT`).
///
/// # Errors
///
/// [`ShorthandError::Empty`] for an empty token; `UnknownJoker`,
/// `UnknownModifier`, `Duplicate` or `BadNumber` as the grammar in the
/// [module docs](self) rules. Never `NoShorthand`.
///
/// # Examples
///
/// ```
/// use buffoonery::prelude::*;
///
/// let joker = parse_joker("JOKER+50c+OFF").unwrap();
/// assert_eq!(joker.ident, "JOKER");
/// assert_eq!(joker.enhancement, Some(MPip::Chips(50)));
/// assert!(joker.debuffed);
/// ```
pub fn parse_joker(token: &str) -> Result<JokerSpec, ShorthandError> {
    if token.is_empty() {
        return Err(ShorthandError::Empty);
    }
    let (ident, modifiers) = token.split_at(ident_end(token));
    let ident = lookup_ident(ident).ok_or_else(|| ShorthandError::UnknownJoker {
        token: token.to_string(),
        ident: ident.to_string(),
    })?;
    let mut joker = JokerSpec {
        ident: ident.to_string(),
        ..JokerSpec::default()
    };
    for modifier in split_modifiers(modifiers) {
        apply_modifier(&mut joker, token, modifier)?;
    }
    Ok(joker)
}

/// Reads a line of tokens, left to right. Any run of spaces, tabs and commas
/// separates tokens; an empty line is an empty row.
///
/// # Errors
///
/// The first token that fails, as a [`LineError`] with its position.
///
/// # Examples
///
/// ```
/// use buffoonery::prelude::*;
///
/// let jokers = parse_jokers("BARON, JOKER+OFF").unwrap();
/// assert_eq!(jokers.len(), 2);
/// assert!(jokers[1].debuffed);
/// ```
pub fn parse_jokers(line: &str) -> Result<Vec<JokerSpec>, LineError> {
    line.split(|c: char| c.is_whitespace() || c == ',')
        .filter(|token| !token.is_empty())
        .enumerate()
        .map(|(position, token)| parse_joker(token).map_err(|error| LineError { position, error }))
        .collect()
}

/// Writes a line of canonical tokens, one space between.
///
/// # Errors
///
/// The first spec with no token form (see [`JokerSpec::shorthand`]), as a
/// [`LineError`] with its position.
///
/// # Examples
///
/// ```
/// use buffoonery::prelude::*;
///
/// let jokers = parse_jokers("baron+foil lucky_catx3.25").unwrap();
/// assert_eq!(shorthand_line(&jokers).unwrap(), "BARON+FOIL LUCKYCATx3.25");
/// ```
pub fn shorthand_line(jokers: &[JokerSpec]) -> Result<String, LineError> {
    let tokens = jokers
        .iter()
        .enumerate()
        .map(|(position, joker)| {
            joker
                .shorthand()
                .map_err(|error| LineError { position, error })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tokens.join(" "))
}

/// Reads one token, as [`parse_joker`] does.
///
/// ```
/// use buffoonery::prelude::*;
///
/// let joker: JokerSpec = "BARON+FOIL".parse().unwrap();
/// assert_eq!(joker.edition, Some(Edition::Foil));
/// ```
impl FromStr for JokerSpec {
    type Err = ShorthandError;

    fn from_str(token: &str) -> Result<Self, Self::Err> {
        parse_joker(token)
    }
}

impl JokerSpec {
    /// The canonical token: the ident without underscores, then the edition
    /// (short spelling), then the override, then `+OFF`. `parse_joker` reads
    /// it back to an equal spec.
    ///
    /// A `Result`, not `Display`: a spec the token cannot carry must say so,
    /// not lose a field.
    ///
    /// # Errors
    ///
    /// [`ShorthandError::UnknownJoker`] if `ident` is not exactly a catalog
    /// ident; [`ShorthandError::NoShorthand`] if the override is not `Chips`,
    /// `MultPlus`, `MultTimes`, `MultTimes1Dot` or `MultTimes2Dot`, or the
    /// edition is `Some(Edition::None)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use buffoonery::prelude::*;
    ///
    /// let joker = JokerSpec {
    ///     ident: "LUCKY_CAT".to_string(),
    ///     enhancement: Some(MPip::MultTimes2Dot(325)),
    ///     ..JokerSpec::default()
    /// };
    /// assert_eq!(joker.shorthand().unwrap(), "LUCKYCATx3.25");
    /// ```
    pub fn shorthand(&self) -> Result<String, ShorthandError> {
        if Joker::by_ident(&self.ident).is_none() {
            return Err(ShorthandError::UnknownJoker {
                token: self.ident.clone(),
                ident: self.ident.clone(),
            });
        }
        let no_shorthand = |value: String| ShorthandError::NoShorthand {
            ident: self.ident.clone(),
            value,
        };
        let edition = match self.edition {
            None => String::new(),
            Some(Edition::None) => return Err(no_shorthand("edition None".to_string())),
            Some(edition) => format!("+{}", edition_word(edition)),
        };
        let enhancement = match self.enhancement {
            None => String::new(),
            Some(MPip::Chips(chips)) => format!("+{chips}c"),
            Some(MPip::MultPlus(mult)) => format!("+{mult}m"),
            Some(MPip::MultTimes(times)) => format!("x{times}"),
            Some(MPip::MultTimes1Dot(tenths)) => format!("x{}.{}", tenths / 10, tenths % 10),
            Some(MPip::MultTimes2Dot(hundredths)) => {
                format!("x{}.{:02}", hundredths / 100, hundredths % 100)
            }
            Some(other) => return Err(no_shorthand(other.to_string())),
        };
        let off = if self.debuffed { "+OFF" } else { "" };
        Ok(format!(
            "{}{edition}{enhancement}{off}",
            self.ident.replace('_', "")
        ))
    }
}

/// Where the ident ends: at the first `+`, or at an `x`/`X` followed by a
/// digit or a dot. No ident holds either (pinned by the catalog's tests).
fn ident_end(token: &str) -> usize {
    let bytes = token.as_bytes();
    bytes
        .iter()
        .enumerate()
        .position(|(index, &byte)| {
            byte == b'+'
                || (matches!(byte, b'x' | b'X')
                    && bytes
                        .get(index + 1)
                        .is_some_and(|next| next.is_ascii_digit() || *next == b'.'))
        })
        .unwrap_or(token.len())
}

/// The catalog ident `ident` names, with underscores dropped and case
/// ignored on both sides.
fn lookup_ident(ident: &str) -> Option<&'static str> {
    let wanted = ident.replace('_', "").to_ascii_uppercase();
    Joker::catalog()
        .into_iter()
        .map(|info| info.ident)
        .find(|candidate| candidate.replace('_', "") == wanted)
}

/// Splits what follows the ident into modifiers, each starting at a `+` or
/// an `x`/`X`. No modifier word holds an `x`.
fn split_modifiers(modifiers: &str) -> Vec<&str> {
    let starts: Vec<usize> = modifiers
        .char_indices()
        .filter(|(_, c)| matches!(c, '+' | 'x' | 'X'))
        .map(|(index, _)| index)
        .collect();
    starts
        .iter()
        .enumerate()
        .map(|(i, &start)| &modifiers[start..starts.get(i + 1).copied().unwrap_or(modifiers.len())])
        .collect()
}

/// Lays one modifier onto `joker`, refusing a second of its kind.
fn apply_modifier(
    joker: &mut JokerSpec,
    token: &str,
    modifier: &str,
) -> Result<(), ShorthandError> {
    let duplicate = |what| ShorthandError::Duplicate {
        token: token.to_string(),
        what,
    };
    let bad_number = |number: &str| ShorthandError::BadNumber {
        token: token.to_string(),
        number: number.to_string(),
    };
    let (sign, body) = modifier.split_at(1);
    if sign != "+" {
        let times = parse_times(body).ok_or_else(|| bad_number(body))?;
        return set_once(&mut joker.enhancement, times).ok_or_else(|| duplicate("overrides"));
    }
    if body.eq_ignore_ascii_case("OFF") {
        if joker.debuffed {
            return Err(duplicate("+OFFs"));
        }
        joker.debuffed = true;
        return Ok(());
    }
    if let Some(edition) = parse_edition(body) {
        return set_once(&mut joker.edition, edition).ok_or_else(|| duplicate("editions"));
    }
    if let Some((number, kind)) = split_counted(body) {
        let count = parse_whole(number).ok_or_else(|| bad_number(number))?;
        let enhancement = if kind == 'c' {
            MPip::Chips(count)
        } else {
            MPip::MultPlus(count)
        };
        return set_once(&mut joker.enhancement, enhancement).ok_or_else(|| duplicate("overrides"));
    }
    Err(ShorthandError::UnknownModifier {
        token: token.to_string(),
        modifier: modifier.to_string(),
    })
}

/// Sets `slot` if it is empty; `None` if it already held a value.
fn set_once<T>(slot: &mut Option<T>, value: T) -> Option<()> {
    if slot.is_some() {
        return None;
    }
    *slot = Some(value);
    Some(())
}

/// The edition a word names, short or long, any case.
fn parse_edition(word: &str) -> Option<Edition> {
    [
        Edition::Foil,
        Edition::Holographic,
        Edition::Polychrome,
        Edition::Negative,
    ]
    .into_iter()
    .find(|edition| {
        word.eq_ignore_ascii_case(edition_word(*edition))
            || word.eq_ignore_ascii_case(&edition.to_string())
    })
}

/// The short spelling the canonical token writes.
fn edition_word(edition: Edition) -> &'static str {
    match edition {
        Edition::Foil => "FOIL",
        Edition::Holographic => "HOLO",
        Edition::Polychrome => "POLY",
        Edition::Negative => "NEG",
        Edition::None => "NONE",
    }
}

/// Splits `50c` / `8m` into the number and its kind (`'c'` or `'m'`), when
/// the body looks like a number: it starts with a digit, `-` or `.`.
fn split_counted(body: &str) -> Option<(&str, char)> {
    let last = body.chars().last()?;
    let kind = last.to_ascii_lowercase();
    let number = &body[..body.len() - last.len_utf8()];
    let looks_numeric = number
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_digit() || first == '-' || first == '.');
    (matches!(kind, 'c' | 'm') && looks_numeric).then_some((number, kind))
}

/// A whole number: digits only.
fn parse_whole(number: &str) -> Option<usize> {
    if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    number.parse().ok()
}

/// A ×mult factor: whole → `MultTimes`, one decimal → `MultTimes1Dot`, two →
/// `MultTimes2Dot`.
fn parse_times(number: &str) -> Option<MPip> {
    let Some((whole, fraction)) = number.split_once('.') else {
        return parse_whole(number).map(MPip::MultTimes);
    };
    let whole = parse_whole(whole)?;
    let digits = parse_whole(fraction)?;
    match fraction.len() {
        1 => whole
            .checked_mul(10)?
            .checked_add(digits)
            .map(MPip::MultTimes1Dot),
        2 => whole
            .checked_mul(100)?
            .checked_add(digits)
            .map(MPip::MultTimes2Dot),
        _ => None,
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod funky__types__joker_shorthand_tests {
    use super::*;
    use crate::decks::joker::Joker;
    use crate::types::edition::Edition;
    use crate::types::mpip::MPip;

    fn spec(ident: &str) -> JokerSpec {
        JokerSpec {
            ident: ident.to_string(),
            ..JokerSpec::default()
        }
    }

    fn with_enhancement(ident: &str, enhancement: MPip) -> JokerSpec {
        JokerSpec {
            enhancement: Some(enhancement),
            ..spec(ident)
        }
    }

    fn with_edition(ident: &str, edition: Edition) -> JokerSpec {
        JokerSpec {
            edition: Some(edition),
            ..spec(ident)
        }
    }

    #[test]
    fn parse__ident_ignores_case_and_underscores() {
        for token in ["LUCKYCAT", "lucky_cat", "Lucky_Cat", "LUCKY_CAT"] {
            assert_eq!(parse_joker(token), Ok(spec("LUCKY_CAT")), "{token}");
        }
    }

    #[test]
    fn parse__unknown_ident_is_an_error() {
        assert_eq!(
            parse_joker("LUCKYCAR"),
            Err(ShorthandError::UnknownJoker {
                token: "LUCKYCAR".to_string(),
                ident: "LUCKYCAR".to_string(),
            })
        );
    }

    #[test]
    fn parse__each_edition_short_and_long() {
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
            let token = format!("BARON+{word}");
            assert_eq!(
                parse_joker(&token),
                Ok(with_edition("BARON", edition)),
                "{token}"
            );
        }
    }

    #[test]
    fn parse__chips_override() {
        assert_eq!(
            parse_joker("JOKER+50c"),
            Ok(with_enhancement("JOKER", MPip::Chips(50)))
        );
    }

    #[test]
    fn parse__mult_override() {
        assert_eq!(
            parse_joker("JOKER+8m"),
            Ok(with_enhancement("JOKER", MPip::MultPlus(8)))
        );
    }

    #[test]
    fn parse__xmult_whole_tenths_hundredths() {
        assert_eq!(
            parse_joker("LUCKYCATx3"),
            Ok(with_enhancement("LUCKY_CAT", MPip::MultTimes(3)))
        );
        assert_eq!(
            parse_joker("LUCKYCATx3.5"),
            Ok(with_enhancement("LUCKY_CAT", MPip::MultTimes1Dot(35)))
        );
        assert_eq!(
            parse_joker("LUCKYCATx3.25"),
            Ok(with_enhancement("LUCKY_CAT", MPip::MultTimes2Dot(325)))
        );
        assert_eq!(
            parse_joker("LUCKYCATX0.05"),
            Ok(with_enhancement("LUCKY_CAT", MPip::MultTimes2Dot(5)))
        );
    }

    #[test]
    fn parse__off_is_debuffed() {
        assert_eq!(
            parse_joker("JOKER+OFF"),
            Ok(JokerSpec {
                debuffed: true,
                ..spec("JOKER")
            })
        );
    }

    #[test]
    fn parse__modifiers_in_any_order() {
        let expected = JokerSpec {
            ident: "JOKER".to_string(),
            enhancement: Some(MPip::Chips(50)),
            edition: Some(Edition::Foil),
            debuffed: true,
        };
        for token in [
            "JOKER+FOIL+50c+OFF",
            "JOKER+OFF+50c+FOIL",
            "joker+50c+off+foil",
        ] {
            assert_eq!(parse_joker(token), Ok(expected.clone()), "{token}");
        }
    }

    #[test]
    fn parse__two_editions_is_duplicate() {
        assert_eq!(
            parse_joker("BARON+FOIL+HOLO"),
            Err(ShorthandError::Duplicate {
                token: "BARON+FOIL+HOLO".to_string(),
                what: "editions",
            })
        );
    }

    #[test]
    fn parse__two_overrides_is_duplicate() {
        assert_eq!(
            parse_joker("JOKER+50cx2"),
            Err(ShorthandError::Duplicate {
                token: "JOKER+50cx2".to_string(),
                what: "overrides",
            })
        );
    }

    #[test]
    fn parse__two_offs_is_duplicate() {
        assert_eq!(
            parse_joker("JOKER+OFF+OFF"),
            Err(ShorthandError::Duplicate {
                token: "JOKER+OFF+OFF".to_string(),
                what: "+OFFs",
            })
        );
    }

    #[test]
    fn parse__three_decimals_is_bad_number() {
        assert_eq!(
            parse_joker("LUCKYCATx3.255"),
            Err(ShorthandError::BadNumber {
                token: "LUCKYCATx3.255".to_string(),
                number: "3.255".to_string(),
            })
        );
    }

    #[test]
    fn parse__xmult_needs_a_whole_part_and_a_decimal() {
        for (token, number) in [("JOKER+FOILx", ""), ("JOKERx3.", "3."), ("JOKERx.5", ".5")] {
            assert_eq!(
                parse_joker(token),
                Err(ShorthandError::BadNumber {
                    token: token.to_string(),
                    number: number.to_string(),
                }),
                "{token}"
            );
        }
    }

    #[test]
    fn parse__decimal_chips_is_bad_number() {
        assert_eq!(
            parse_joker("JOKER+2.5c"),
            Err(ShorthandError::BadNumber {
                token: "JOKER+2.5c".to_string(),
                number: "2.5".to_string(),
            })
        );
    }

    #[test]
    fn parse__negative_mult_is_bad_number() {
        assert_eq!(
            parse_joker("JOKER+-4m"),
            Err(ShorthandError::BadNumber {
                token: "JOKER+-4m".to_string(),
                number: "-4".to_string(),
            })
        );
    }

    #[test]
    fn parse__unknown_modifier_names_it() {
        assert_eq!(
            parse_joker("BARON+SHINY"),
            Err(ShorthandError::UnknownModifier {
                token: "BARON+SHINY".to_string(),
                modifier: "+SHINY".to_string(),
            })
        );
        assert_eq!(
            parse_joker("BARON+"),
            Err(ShorthandError::UnknownModifier {
                token: "BARON+".to_string(),
                modifier: "+".to_string(),
            })
        );
    }

    #[test]
    fn parse__non_ascii_modifier_is_unknown_not_a_panic() {
        for (token, modifier) in [("JOKER+5é", "+5é"), ("JOKER+é", "+é")] {
            assert_eq!(
                parse_joker(token),
                Err(ShorthandError::UnknownModifier {
                    token: token.to_string(),
                    modifier: modifier.to_string(),
                }),
                "{token}"
            );
        }
    }

    #[test]
    fn parse_joker__empty_is_an_error() {
        assert_eq!(parse_joker(""), Err(ShorthandError::Empty));
    }

    #[test]
    fn from_str__same_as_parse_joker() {
        assert_eq!("BARON+FOIL".parse::<JokerSpec>(), parse_joker("BARON+FOIL"));
    }

    #[test]
    fn error__display_quotes_the_token() {
        let message = |token: &str| parse_joker(token).unwrap_err().to_string();
        assert_eq!(message("LUCKYCAR"), r#""LUCKYCAR": unknown joker"#);
        assert_eq!(
            message("BARON+FOIL+HOLO"),
            r#""BARON+FOIL+HOLO": two editions"#
        );
        assert_eq!(
            message("BARON+SHINY"),
            r#""BARON+SHINY": unknown modifier "+SHINY""#
        );
        assert_eq!(
            message("LUCKYCATx3.255"),
            r#""LUCKYCATx3.255": bad number "3.255""#
        );
        assert_eq!(message(""), "empty joker token");
        assert_eq!(
            ShorthandError::NoShorthand {
                ident: "LUCKY_CAT".to_string(),
                value: "Lucky(5, 15)".to_string(),
            }
            .to_string(),
            "LUCKY_CAT: Lucky(5, 15) has no shorthand"
        );
    }

    #[test]
    fn line_error__display_adds_the_position() {
        let error = LineError {
            position: 1,
            error: parse_joker("LUCKYCAR").unwrap_err(),
        };
        assert_eq!(error.to_string(), r#"jokers[1]: "LUCKYCAR": unknown joker"#);
    }

    #[test]
    fn shorthand__canonical_form() {
        assert_eq!(
            with_edition("BARON", Edition::Foil).shorthand(),
            Ok("BARON+FOIL".to_string())
        );
        assert_eq!(
            with_enhancement("LUCKY_CAT", MPip::MultTimes2Dot(325)).shorthand(),
            Ok("LUCKYCATx3.25".to_string())
        );
        let joker = JokerSpec {
            debuffed: true,
            ..with_enhancement("JOKER", MPip::Chips(50))
        };
        assert_eq!(joker.shorthand(), Ok("JOKER+50c+OFF".to_string()));
        let everything = JokerSpec {
            ident: "LUCKY_CAT".to_string(),
            enhancement: Some(MPip::MultPlus(4)),
            edition: Some(Edition::Polychrome),
            debuffed: true,
        };
        assert_eq!(
            everything.shorthand(),
            Ok("LUCKYCAT+POLY+4m+OFF".to_string())
        );
    }

    #[test]
    fn shorthand__each_edition_writes_its_short_spelling() {
        let cases = [
            (Edition::Foil, "BARON+FOIL"),
            (Edition::Holographic, "BARON+HOLO"),
            (Edition::Polychrome, "BARON+POLY"),
            (Edition::Negative, "BARON+NEG"),
        ];
        for (edition, token) in cases {
            assert_eq!(
                with_edition("BARON", edition).shorthand(),
                Ok(token.to_string())
            );
        }
    }

    #[test]
    fn shorthand__keeps_the_variant() {
        let cases = [
            (MPip::MultTimes(3), "JOKERx3"),
            (MPip::MultTimes1Dot(30), "JOKERx3.0"),
            (MPip::MultTimes1Dot(15), "JOKERx1.5"),
            (MPip::MultTimes2Dot(350), "JOKERx3.50"),
            (MPip::MultTimes2Dot(5), "JOKERx0.05"),
        ];
        for (enhancement, token) in cases {
            let joker = with_enhancement("JOKER", enhancement);
            assert_eq!(joker.shorthand(), Ok(token.to_string()));
            assert_eq!(parse_joker(token), Ok(joker), "{token}");
        }
    }

    #[test]
    fn shorthand__round_trips_every_catalog_joker() {
        for info in Joker::catalog() {
            let joker = spec(info.ident);
            let token = joker.shorthand().unwrap();
            assert_eq!(token, info.ident.replace('_', ""));
            assert_eq!(parse_joker(&token), Ok(joker), "{token}");
        }
    }

    #[test]
    fn shorthand__round_trips_every_numeric_override_kind() {
        let kinds = [
            MPip::Chips(50),
            MPip::MultPlus(4),
            MPip::MultTimes(2),
            MPip::MultTimes1Dot(15),
            MPip::MultTimes2Dot(325),
        ];
        for enhancement in kinds {
            let joker = JokerSpec {
                edition: Some(Edition::Holographic),
                debuffed: true,
                ..with_enhancement("SLY_JOKER", enhancement)
            };
            let token = joker.shorthand().unwrap();
            assert_eq!(parse_joker(&token), Ok(joker), "{token}");
        }
    }

    #[test]
    fn shorthand__other_override_is_no_shorthand() {
        for enhancement in [MPip::Lucky(5, 15), MPip::Custom(1)] {
            assert_eq!(
                with_enhancement("LUCKY_CAT", enhancement).shorthand(),
                Err(ShorthandError::NoShorthand {
                    ident: "LUCKY_CAT".to_string(),
                    value: enhancement.to_string(),
                })
            );
        }
    }

    #[test]
    fn shorthand__edition_none_is_no_shorthand() {
        assert_eq!(
            with_edition("BARON", Edition::None).shorthand(),
            Err(ShorthandError::NoShorthand {
                ident: "BARON".to_string(),
                value: "edition None".to_string(),
            })
        );
    }

    #[test]
    fn shorthand__unknown_ident_is_an_error() {
        assert_eq!(
            spec("lucky_cat").shorthand(),
            Err(ShorthandError::UnknownJoker {
                token: "lucky_cat".to_string(),
                ident: "lucky_cat".to_string(),
            })
        );
    }

    #[test]
    fn parse_jokers__any_run_of_separators() {
        let expected = Ok(vec![spec("BARON"), spec("JOKER")]);
        for line in [
            "BARON JOKER",
            "BARON,,JOKER",
            "BARON , JOKER",
            "\tBARON\t\tJOKER\n",
        ] {
            assert_eq!(parse_jokers(line), expected, "{line:?}");
        }
    }

    #[test]
    fn parse_jokers__error_carries_position() {
        assert_eq!(
            parse_jokers("BARON JOKER LUCKYCAR"),
            Err(LineError {
                position: 2,
                error: ShorthandError::UnknownJoker {
                    token: "LUCKYCAR".to_string(),
                    ident: "LUCKYCAR".to_string(),
                },
            })
        );
    }

    #[test]
    fn parse_jokers__empty_line_is_empty_vec() {
        assert_eq!(parse_jokers(""), Ok(Vec::new()));
        assert_eq!(parse_jokers(" ,\t"), Ok(Vec::new()));
    }

    #[test]
    fn shorthand_line__joins_with_spaces() {
        let jokers = [
            with_edition("BARON", Edition::Foil),
            with_enhancement("LUCKY_CAT", MPip::MultTimes2Dot(325)),
        ];
        assert_eq!(
            shorthand_line(&jokers),
            Ok("BARON+FOIL LUCKYCATx3.25".to_string())
        );
        assert_eq!(shorthand_line(&[]), Ok(String::new()));
    }

    #[test]
    fn shorthand_line__error_carries_position() {
        let jokers = [spec("BARON"), with_enhancement("JOKER", MPip::Custom(1))];
        assert_eq!(
            shorthand_line(&jokers),
            Err(LineError {
                position: 1,
                error: ShorthandError::NoShorthand {
                    ident: "JOKER".to_string(),
                    value: "Custom(1)".to_string(),
                },
            })
        );
    }
}
