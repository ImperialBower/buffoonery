//! The joker catalog: a display name and a stable identifier for every joker
//! in [`Joker::ALL_JOKERS`] (EPIC-05 Story 2).
//!
//! The names are a plain table, not a translation file: this crate has no
//! i18n feature, and a table costs nothing. Each identifier is the joker
//! const's own name, taken by `stringify!`, so an identifier cannot drift
//! from its card. The table must list every joker in `ALL_JOKERS` order, or
//! it does not compile (its length is `ALL_JOKERS_SIZE`) or a test fails.

use crate::decks::joker::{Joker, card};
use crate::types::buffoon_card::{BCardType, BuffoonCard};
use crate::types::mpip::MPip;
use serde::Serialize;

/// What the calculator shows about a joker. Built from the card itself, so it
/// cannot disagree with what the engine scores.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct JokerInfo {
    /// The joker const's name, e.g. `"JOLLY_JOKER"`. Stable: boards name
    /// jokers by it.
    pub ident: &'static str,
    /// The joker's name in Balatro, e.g. `"Jolly Joker"`.
    pub name: &'static str,
    pub rarity: BCardType,
    /// The shop price in dollars.
    pub cost: usize,
    /// The joker's effect, as data.
    pub enhancement: MPip,
    /// Why the joker has no effect in this engine, if it is `Blank`.
    pub blank_reason: Option<&'static str>,
}

/// `(ident, name, card)` for one joker; the ident comes from the const name.
macro_rules! entries {
    ($($ident:ident => $name:literal),* $(,)?) => {
        [$((stringify!($ident), $name, card::$ident)),*]
    };
}

/// Every joker, in [`Joker::ALL_JOKERS`] order.
const CATALOG: [(&str, &str, BuffoonCard); Joker::ALL_JOKERS_SIZE] = entries![
    JOKER => "Joker",
    GREEDY_JOKER => "Greedy Joker",
    LUSTY_JOKER => "Lusty Joker",
    WRATHFUL_JOKER => "Wrathful Joker",
    GLUTTONOUS_JOKER => "Gluttonous Joker",
    JOLLY_JOKER => "Jolly Joker",
    ZANY_JOKER => "Zany Joker",
    MAD_JOKER => "Mad Joker",
    CRAZY_JOKER => "Crazy Joker",
    DROLL_JOKER => "Droll Joker",
    SLY_JOKER => "Sly Joker",
    WILY_JOKER => "Wily Joker",
    CLEVER_JOKER => "Clever Joker",
    DEVIOUS_JOKER => "Devious Joker",
    CRAFTY_JOKER => "Crafty Joker",
    HALF_JOKER => "Half Joker",
    JOKER_STENCIL => "Joker Stencil",
    FOUR_FINGERS => "Four Fingers",
    MIME => "Mime",
    CREDIT_CARD => "Credit Card",
    CEREMONIAL_DAGGER => "Ceremonial Dagger",
    BANNER => "Banner",
    MYSTIC_SUMMIT => "Mystic Summit",
    MARBLE_JOKER => "Marble Joker",
    LOYALTY_CARD => "Loyalty Card",
    EIGHT_BALL => "8 Ball",
    MISPRINT => "Misprint",
    DUSK => "Dusk",
    RAISED_FIST => "Raised Fist",
    CHAOS_THE_CLOWN => "Chaos the Clown",
    FIBONACCI => "Fibonacci",
    STEEL_JOKER => "Steel Joker",
    SCARY_FACE => "Scary Face",
    ABSTRACT_JOKER => "Abstract Joker",
    DELAYED_GRATIFICATION => "Delayed Gratification",
    HACK => "Hack",
    PAREIDOLIA => "Pareidolia",
    GROS_MICHEL => "Gros Michel",
    EVEN_STEVEN => "Even Steven",
    ODD_TODD => "Odd Todd",
    SCHOLAR => "Scholar",
    BUSINESS_CARD => "Business Card",
    SUPERNOVA => "Supernova",
    RIDE_THE_BUS => "Ride the Bus",
    SPACE_JOKER => "Space Joker",
    EGG => "Egg",
    BURGLAR => "Burglar",
    BLACKBOARD => "Blackboard",
    RUNNER => "Runner",
    ICE_CREAM => "Ice Cream",
    DNA => "DNA",
    SPLASH => "Splash",
    BLUE_JOKER => "Blue Joker",
    SIXTH_SENSE => "Sixth Sense",
    CONSTELLATION => "Constellation",
    HIKER => "Hiker",
    FACELESS_JOKER => "Faceless Joker",
    GREEN_JOKER => "Green Joker",
    SUPERPOSITION => "Superposition",
    TO_DO_LIST => "To Do List",
    CAVENDISH => "Cavendish",
    CARD_SHARP => "Card Sharp",
    RED_CARD => "Red Card",
    MADNESS => "Madness",
    SQUARE_JOKER => "Square Joker",
    SEANCE => "Séance",
    RIFF_RAFF => "Riff-raff",
    VAMPIRE => "Vampire",
    SHORTCUT => "Shortcut",
    HOLOGRAM => "Hologram",
    VAGABOND => "Vagabond",
    BARON => "Baron",
    CLOUD_9 => "Cloud 9",
    ROCKET => "Rocket",
    EROSION => "Erosion",
    RESERVED_PARKING => "Reserved Parking",
    MAIL_IN_REBATE => "Mail-In Rebate",
    TO_THE_MOON => "To the Moon",
    HALLUCINATION => "Hallucination",
    FORTUNE_TELLER => "Fortune Teller",
    JUGGLER => "Juggler",
    DRUNKARD => "Drunkard",
    STONE_JOKER => "Stone Joker",
    GOLDEN_JOKER => "Golden Joker",
    LUCKY_CAT => "Lucky Cat",
    BASEBALL_CARD => "Baseball Card",
    BULL => "Bull",
    DIET_COLA => "Diet Cola",
    TRADING_CARD => "Trading Card",
    FLASH_CARD => "Flash Card",
    POPCORN => "Popcorn",
    SPARE_TROUSERS => "Spare Trousers",
    ANCIENT_JOKER => "Ancient Joker",
    RAMEN => "Ramen",
    WALKIE_TALKIE => "Walkie Talkie",
    SOCK_AND_BUSKIN => "Sock and Buskin",
    HANGING_CHAD => "Hanging Chad",
    SMEARED_JOKER => "Smeared Joker",
    OOPS_ALL_6S => "Oops! All 6s",
    LUCHADOR => "Luchador",
    MATADOR => "Matador",
    SELTZER => "Seltzer",
    THE_DUO => "The Duo",
    THE_TRIO => "The Trio",
    THE_FAMILY => "The Family",
    THE_ORDER => "The Order",
    THE_TRIBE => "The Tribe",
    CANIO => "Canio",
    TRIBOULET => "Triboulet",
    YORICK => "Yorick",
    CHICOT => "Chicot",
    PERKEO => "Perkeo",
];

impl Joker {
    /// Every joker's [`JokerInfo`], in [`Joker::ALL_JOKERS`] order.
    #[must_use]
    pub fn catalog() -> Vec<JokerInfo> {
        CATALOG.iter().map(info_of).collect()
    }

    /// The joker named by `ident` (the const name, e.g. `"JOLLY_JOKER"`), as
    /// its unedited card. Exact match only.
    #[must_use]
    pub fn by_ident(ident: &str) -> Option<BuffoonCard> {
        CATALOG
            .iter()
            .find(|(id, _, _)| *id == ident)
            .map(|(_, _, card)| *card)
    }

    /// The identifier of the joker `card` is. An edited joker (edition,
    /// effect override, debuff) is still the same joker: lookup is by its
    /// rank pip, which every joker has its own of.
    #[must_use]
    pub fn ident(card: &BuffoonCard) -> Option<&'static str> {
        entry_of(card).map(|(ident, _, _)| *ident)
    }

    /// The Balatro name of the joker `card` is, e.g. `"Jolly Joker"`.
    #[must_use]
    pub fn name(card: &BuffoonCard) -> Option<&'static str> {
        entry_of(card).map(|(_, name, _)| *name)
    }

    /// The [`JokerInfo`] of the joker `card` is, read from the unedited card.
    #[must_use]
    pub fn info(card: &BuffoonCard) -> Option<JokerInfo> {
        entry_of(card).map(info_of)
    }
}

fn entry_of(card: &BuffoonCard) -> Option<&'static (&'static str, &'static str, BuffoonCard)> {
    CATALOG.iter().find(|(_, _, joker)| joker.rank == card.rank)
}

fn info_of(entry: &(&'static str, &'static str, BuffoonCard)) -> JokerInfo {
    let (ident, name, card) = *entry;
    JokerInfo {
        ident,
        name,
        rarity: card.card_type,
        cost: card.rank.value,
        enhancement: card.enhancement,
        blank_reason: Joker::blank_reason(&card),
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod funky__decks__joker_catalog_tests {
    use super::*;
    use crate::decks::basic::card as basic;
    use crate::decks::joker::{Joker, card};
    use crate::types::buffoon_card::BCardType;
    use crate::types::edition::Edition;
    use crate::types::mpip::MPip;
    use std::collections::HashSet;

    #[test]
    fn catalog__covers_every_joker_in_order() {
        let cards: Vec<_> = Joker::catalog()
            .iter()
            .map(|info| Joker::by_ident(info.ident).unwrap())
            .collect();
        assert_eq!(cards, Joker::ALL_JOKERS.to_vec());
    }

    /// The joker shorthand reads `LUCKYCAT` as `LUCKY_CAT`, so no two idents
    /// may collide once underscores are dropped.
    #[test]
    fn idents__unique_without_underscores() {
        let bare: HashSet<_> = Joker::catalog()
            .iter()
            .map(|info| info.ident.replace('_', ""))
            .collect();
        assert_eq!(bare.len(), Joker::ALL_JOKERS_SIZE);
    }

    /// The joker shorthand ends an ident at an `x` followed by a digit or a
    /// dot (`LUCKYCATx3.25`), so no ident may hold one, with or without its
    /// underscores.
    #[test]
    fn idents__no_x_before_a_digit() {
        let splits = |text: &str| {
            text.as_bytes()
                .windows(2)
                .any(|pair| pair[0] == b'X' && (pair[1].is_ascii_digit() || pair[1] == b'.'))
        };
        for info in Joker::catalog() {
            assert!(!splits(info.ident), "{}", info.ident);
            assert!(!splits(&info.ident.replace('_', "")), "{}", info.ident);
        }
    }

    #[test]
    fn catalog__idents_and_names_are_unique() {
        let catalog = Joker::catalog();
        let idents: HashSet<_> = catalog.iter().map(|i| i.ident).collect();
        let names: HashSet<_> = catalog.iter().map(|i| i.name).collect();
        assert_eq!(idents.len(), Joker::ALL_JOKERS_SIZE);
        assert_eq!(names.len(), Joker::ALL_JOKERS_SIZE);
    }

    #[test]
    fn catalog__names_are_trimmed_and_not_empty() {
        for info in Joker::catalog() {
            assert!(!info.name.is_empty(), "{}", info.ident);
            assert_eq!(info.name, info.name.trim(), "{}", info.ident);
        }
    }

    /// Lookups go by the card's rank pip, so ranks must be unique.
    #[test]
    fn catalog__joker_ranks_are_unique() {
        let ranks: HashSet<_> = Joker::ALL_JOKERS.iter().map(|c| c.rank).collect();
        assert_eq!(ranks.len(), Joker::ALL_JOKERS_SIZE);
    }

    /// Names whose spelling is not the const name in title case.
    #[test]
    fn name__special_spellings() {
        assert_eq!(Joker::name(&card::EIGHT_BALL), Some("8 Ball"));
        assert_eq!(Joker::name(&card::DNA), Some("DNA"));
        assert_eq!(Joker::name(&card::OOPS_ALL_6S), Some("Oops! All 6s"));
        assert_eq!(Joker::name(&card::SEANCE), Some("Séance"));
        assert_eq!(Joker::name(&card::RIFF_RAFF), Some("Riff-raff"));
        assert_eq!(Joker::name(&card::CLOUD_9), Some("Cloud 9"));
        assert_eq!(Joker::name(&card::MAIL_IN_REBATE), Some("Mail-In Rebate"));
        assert_eq!(Joker::name(&card::TO_DO_LIST), Some("To Do List"));
        assert_eq!(Joker::name(&card::CHAOS_THE_CLOWN), Some("Chaos the Clown"));
        assert_eq!(Joker::name(&card::SOCK_AND_BUSKIN), Some("Sock and Buskin"));
        assert_eq!(Joker::name(&card::CANIO), Some("Canio"));
    }

    /// An edited joker (edition, effect override, debuff) is still the same joker.
    #[test]
    fn name__survives_overrides() {
        let edited = BuffoonCard {
            enhancement: MPip::MultPlus(99),
            debuffed: true,
            ..card::JOKER.with_edition(Edition::Foil)
        };
        assert_eq!(Joker::name(&edited), Some("Joker"));
        assert_eq!(Joker::ident(&edited), Some("JOKER"));
    }

    #[test]
    fn name__is_none_for_a_non_joker() {
        assert_eq!(Joker::name(&basic::ACE_SPADES), None);
        assert_eq!(Joker::info(&basic::ACE_SPADES), None);
    }

    #[test]
    fn info__reads_the_card() {
        assert_eq!(
            Joker::info(&card::JOKER),
            Some(JokerInfo {
                ident: "JOKER",
                name: "Joker",
                rarity: BCardType::CommonJoker,
                cost: 2,
                enhancement: MPip::MultPlus(4),
                blank_reason: None,
            })
        );
    }

    #[test]
    fn info__carries_the_blank_reason() {
        let (blank, reason) = Joker::BLANK_WITH_REASON[0];
        assert_eq!(Joker::info(&blank).unwrap().blank_reason, Some(reason));
    }

    #[test]
    fn by_ident__round_trips_every_joker() {
        for joker in Joker::ALL_JOKERS {
            let ident = Joker::ident(&joker).unwrap();
            assert_eq!(Joker::by_ident(ident), Some(joker), "{ident}");
        }
        assert_eq!(Joker::by_ident("JOLLY_JOKER"), Some(card::JOLLY_JOKER));
        assert_eq!(Joker::by_ident("NOT_A_JOKER"), None);
        assert_eq!(Joker::by_ident("jolly_joker"), None, "idents are exact");
    }

    /// Pins the catalog's JSON shape: the calculator reads it.
    #[test]
    fn joker_info__serde_format() {
        assert_eq!(
            serde_json::to_string(&Joker::info(&card::JOKER).unwrap()).unwrap(),
            r#"{"ident":"JOKER","name":"Joker","rarity":"CommonJoker","cost":2,"enhancement":{"MultPlus":4},"blank_reason":null}"#
        );
    }
}
