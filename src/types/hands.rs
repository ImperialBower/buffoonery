use crate::prelude::{BCardType, BuffoonCard, MPip};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Ord, PartialOrd, Serialize, Deserialize,
)]
pub enum HandType {
    #[default]
    HighCard,
    Pair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
    RoyalFlush,
    FiveOfAKind,
    FlushHouse,
    FlushFive,
}

#[derive(
    Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Ord, PartialOrd, Serialize, Deserialize,
)]
pub struct PokerHand {
    pub hand_type: HandType,
    pub level: usize,
    pub chips: usize,
    pub mult: usize,
    pub times_played: usize,
}

impl PokerHand {
    #[must_use]
    pub fn new(hand_type: HandType, chips: usize, mult: usize) -> Self {
        Self {
            hand_type,
            level: 1,
            chips,
            mult,
            times_played: 0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PokerHands {
    pub hands: HashMap<HandType, PokerHand>,
}

impl PokerHands {
    #[must_use]
    pub fn new() -> Self {
        let mut hands = HashMap::new();
        hands.insert(HandType::HighCard, PokerHand::new(HandType::HighCard, 5, 1));
        hands.insert(HandType::Pair, PokerHand::new(HandType::Pair, 10, 2));
        hands.insert(HandType::TwoPair, PokerHand::new(HandType::TwoPair, 20, 2));
        hands.insert(
            HandType::ThreeOfAKind,
            PokerHand::new(HandType::ThreeOfAKind, 30, 3),
        );
        hands.insert(
            HandType::Straight,
            PokerHand::new(HandType::Straight, 30, 4),
        );
        hands.insert(HandType::Flush, PokerHand::new(HandType::Flush, 35, 4));
        hands.insert(
            HandType::FullHouse,
            PokerHand::new(HandType::FullHouse, 40, 4),
        );
        hands.insert(
            HandType::FourOfAKind,
            PokerHand::new(HandType::FourOfAKind, 50, 7),
        );
        hands.insert(
            HandType::StraightFlush,
            PokerHand::new(HandType::StraightFlush, 100, 8),
        );
        hands.insert(
            HandType::FiveOfAKind,
            PokerHand::new(HandType::FiveOfAKind, 120, 12),
        );
        hands.insert(
            HandType::FlushHouse,
            PokerHand::new(HandType::FlushHouse, 140, 14),
        );
        hands.insert(
            HandType::FlushFive,
            PokerHand::new(HandType::FlushFive, 160, 16),
        );
        Self { hands }
    }

    #[must_use]
    pub fn get(&self, hand_type: &HandType) -> Option<&PokerHand> {
        self.hands.get(hand_type)
    }

    pub fn get_mut(&mut self, hand_type: &HandType) -> Option<&mut PokerHand> {
        self.hands.get_mut(hand_type)
    }

    pub fn increment(&mut self, planet_card: BuffoonCard) {
        if planet_card.card_type == BCardType::Planet
            && let MPip::ChipsMultPlusOnHand(chips, mult, hand_type) = planet_card.enhancement
            && let Some(poker_hand) = self.get_mut(&hand_type)
        {
            poker_hand.chips += chips;
            poker_hand.mult += mult;
            poker_hand.level += 1;
        }
    }

    pub fn play_hand(&mut self, hand_type: &HandType) {
        if let Some(poker_hand) = self.get_mut(hand_type) {
            poker_hand.times_played += 1;
        }
    }

    /// The per-level chips/mult each hand gains — Balatro's level-up table.
    ///
    /// Kept here rather than read off the planet cards because the planet data is
    /// not a clean per-hand map (two planets both target `FlushFive`, and
    /// `FlushHouse` has none), so this is the one source of truth a whole-table
    /// level-up can trust. Reconciling the planet consts against it is EPIC-01
    /// Story 3 debt, out of scope here.
    const LEVEL_UP: [(HandType, usize, usize); 12] = [
        (HandType::HighCard, 10, 1),
        (HandType::Pair, 15, 1),
        (HandType::TwoPair, 20, 1),
        (HandType::ThreeOfAKind, 20, 2),
        (HandType::Straight, 30, 3),
        (HandType::Flush, 15, 2),
        (HandType::FullHouse, 25, 2),
        (HandType::FourOfAKind, 30, 3),
        (HandType::StraightFlush, 40, 4),
        (HandType::FiveOfAKind, 35, 3),
        (HandType::FlushHouse, 40, 4),
        (HandType::FlushFive, 50, 3),
    ];

    /// Sets `hand_type` to `level`: its base chips and mult plus `level - 1`
    /// level-ups from the level-up table, as if levelled one step at a time.
    /// Keeps `times_played`. Returns false, and changes nothing, for level 0
    /// or a hand with no entry of its own (a Royal Flush levels as a Straight
    /// Flush).
    pub fn set_level(&mut self, hand_type: HandType, level: usize) -> bool {
        let Some(base) = Self::new().get(&hand_type).copied() else {
            return false;
        };
        let Some((_, chips, mult)) = Self::LEVEL_UP.iter().find(|(h, _, _)| *h == hand_type) else {
            return false;
        };
        if level == 0 {
            return false;
        }
        let Some(hand) = self.get_mut(&hand_type) else {
            return false;
        };
        let ups = level - 1;
        hand.level = level;
        hand.chips = base.chips + chips * ups;
        hand.mult = base.mult + mult * ups;
        true
    }

    /// Level **every** poker hand up by one — Black Hole's effect. Each hand
    /// gains its canonical per-level chips/mult and one level, exactly as its
    /// planet card would, but for all twelve at once.
    pub fn increment_all(&mut self) {
        for (hand_type, chips, mult) in Self::LEVEL_UP {
            if let Some(poker_hand) = self.get_mut(&hand_type) {
                poker_hand.chips += chips;
                poker_hand.mult += mult;
                poker_hand.level += 1;
            }
        }
    }
}

impl Default for PokerHands {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(non_snake_case, unused_imports)]
mod funky__types__hands_tests {
    use super::*;
    use crate::decks::planet;
    use crate::prelude::*;
    use rstest::rstest;

    /// Setting a level matches levelling up one step at a time.
    #[test]
    fn set_level__matches_levelling_up_step_by_step() {
        let mut stepped = PokerHands::default();
        stepped.increment_all();
        stepped.increment_all();

        for hand_type in [HandType::HighCard, HandType::Flush, HandType::FlushFive] {
            let mut set = PokerHands::default();
            assert!(set.set_level(hand_type, 3));
            assert_eq!(
                set.get(&hand_type),
                stepped.get(&hand_type),
                "{hand_type:?}"
            );
        }
    }

    #[test]
    fn set_level__one_resets_to_the_base() {
        let mut hands = PokerHands::default();
        hands.increment_all();
        assert!(hands.set_level(HandType::Pair, 1));
        assert_eq!(
            hands.get(&HandType::Pair),
            PokerHands::default().get(&HandType::Pair)
        );
    }

    #[test]
    fn set_level__keeps_times_played() {
        let mut hands = PokerHands::default();
        hands.play_hand(&HandType::Pair);
        assert!(hands.set_level(HandType::Pair, 2));
        assert_eq!(hands.get(&HandType::Pair).unwrap().times_played, 1);
    }

    #[test]
    fn set_level__refuses_level_zero_and_unlevelled_hands() {
        let mut hands = PokerHands::default();
        assert!(!hands.set_level(HandType::Pair, 0));
        // A Royal Flush levels as a Straight Flush; it has no entry of its own.
        assert!(!hands.set_level(HandType::RoyalFlush, 2));
        assert_eq!(hands, PokerHands::default());
    }

    #[test]
    fn increment() {
        let mut hands = PokerHands::default();
        let expected = PokerHand {
            hand_type: HandType::HighCard,
            level: 2,
            chips: 15,
            mult: 2,
            times_played: 0,
        };

        hands.increment(planet::card::PLUTO);

        assert_eq!(hands.get(&HandType::HighCard).unwrap(), &expected);
    }

    #[test]
    fn increment_all__levels_every_hand_by_one() {
        let mut hands = PokerHands::default();
        hands.increment_all();

        // HighCard: level 1→2, chips 5→15 (+10), mult 1→2 (+1).
        let high = hands.get(&HandType::HighCard).unwrap();
        assert_eq!((high.level, high.chips, high.mult), (2, 15, 2));

        // Every hand type gained exactly one level.
        for hand_type in [
            HandType::Pair,
            HandType::TwoPair,
            HandType::ThreeOfAKind,
            HandType::Straight,
            HandType::Flush,
            HandType::FullHouse,
            HandType::FourOfAKind,
            HandType::StraightFlush,
            HandType::FiveOfAKind,
            HandType::FlushHouse,
            HandType::FlushFive,
        ] {
            assert_eq!(hands.get(&hand_type).unwrap().level, 2, "{hand_type:?}");
        }
    }

    #[test]
    fn play_hand() {
        let mut hands = PokerHands::default();
        let expected = PokerHand {
            hand_type: HandType::HighCard,
            level: 1,
            chips: 5,
            mult: 1,
            times_played: 1,
        };

        hands.play_hand(&HandType::HighCard);

        assert_eq!(hands.get(&HandType::HighCard).unwrap(), &expected);
    }

    #[test]
    fn hand_type__serde_round_trip() {
        for hand_type in [
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
        ] {
            let json = serde_json::to_string(&hand_type).unwrap();

            assert_eq!(serde_json::from_str::<HandType>(&json).unwrap(), hand_type);
        }
    }

    /// Pins the wire format: a consumer parsing this JSON breaks if it changes.
    #[test]
    fn hand_type__serde_format() {
        assert_eq!(
            serde_json::to_string(&HandType::FlushFive).unwrap(),
            r#""FlushFive""#
        );
    }
}
