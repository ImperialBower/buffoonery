#[macro_export]
macro_rules! bcard {
    (AS) => {
        $crate::decks::basic::card::ACE_SPADES
    };
    (2S) => {
        $crate::decks::basic::card::DEUCE_SPADES
    };
    (3S) => {
        $crate::decks::basic::card::TREY_SPADES
    };
    (4S) => {
        $crate::decks::basic::card::FOUR_SPADES
    };
    (5S) => {
        $crate::decks::basic::card::FIVE_SPADES
    };
    (6S) => {
        $crate::decks::basic::card::SIX_SPADES
    };
    (7S) => {
        $crate::decks::basic::card::SEVEN_SPADES
    };
    (8S) => {
        $crate::decks::basic::card::EIGHT_SPADES
    };
    (9S) => {
        $crate::decks::basic::card::NINE_SPADES
    };
    (TS) => {
        $crate::decks::basic::card::TEN_SPADES
    };
    (JS) => {
        $crate::decks::basic::card::JACK_SPADES
    };
    (QS) => {
        $crate::decks::basic::card::QUEEN_SPADES
    };
    (KS) => {
        $crate::decks::basic::card::KING_SPADES
    };
    (AD) => {
        $crate::decks::basic::card::ACE_DIAMONDS
    };
    (2D) => {
        $crate::decks::basic::card::DEUCE_DIAMONDS
    };
    (3D) => {
        $crate::decks::basic::card::TREY_DIAMONDS
    };
    (4D) => {
        $crate::decks::basic::card::FOUR_DIAMONDS
    };
    (5D) => {
        $crate::decks::basic::card::FIVE_DIAMONDS
    };
    (6D) => {
        $crate::decks::basic::card::SIX_DIAMONDS
    };
    (7D) => {
        $crate::decks::basic::card::SEVEN_DIAMONDS
    };
    (8D) => {
        $crate::decks::basic::card::EIGHT_DIAMONDS
    };
    (9D) => {
        $crate::decks::basic::card::NINE_DIAMONDS
    };
    (TD) => {
        $crate::decks::basic::card::TEN_DIAMONDS
    };
    (JD) => {
        $crate::decks::basic::card::JACK_DIAMONDS
    };
    (QD) => {
        $crate::decks::basic::card::QUEEN_DIAMONDS
    };
    (KD) => {
        $crate::decks::basic::card::KING_DIAMONDS
    };
    (AH) => {
        $crate::decks::basic::card::ACE_HEARTS
    };
    (2H) => {
        $crate::decks::basic::card::DEUCE_HEARTS
    };
    (3H) => {
        $crate::decks::basic::card::TREY_HEARTS
    };
    (4H) => {
        $crate::decks::basic::card::FOUR_HEARTS
    };
    (5H) => {
        $crate::decks::basic::card::FIVE_HEARTS
    };
    (6H) => {
        $crate::decks::basic::card::SIX_HEARTS
    };
    (7H) => {
        $crate::decks::basic::card::SEVEN_HEARTS
    };
    (8H) => {
        $crate::decks::basic::card::EIGHT_HEARTS
    };
    (9H) => {
        $crate::decks::basic::card::NINE_HEARTS
    };
    (TH) => {
        $crate::decks::basic::card::TEN_HEARTS
    };
    (JH) => {
        $crate::decks::basic::card::JACK_HEARTS
    };
    (QH) => {
        $crate::decks::basic::card::QUEEN_HEARTS
    };
    (KH) => {
        $crate::decks::basic::card::KING_HEARTS
    };
    (AC) => {
        $crate::decks::basic::card::ACE_CLUBS
    };
    (2C) => {
        $crate::decks::basic::card::DEUCE_CLUBS
    };
    (3C) => {
        $crate::decks::basic::card::TREY_CLUBS
    };
    (4C) => {
        $crate::decks::basic::card::FOUR_CLUBS
    };
    (5C) => {
        $crate::decks::basic::card::FIVE_CLUBS
    };
    (6C) => {
        $crate::decks::basic::card::SIX_CLUBS
    };
    (7C) => {
        $crate::decks::basic::card::SEVEN_CLUBS
    };
    (8C) => {
        $crate::decks::basic::card::EIGHT_CLUBS
    };
    (9C) => {
        $crate::decks::basic::card::NINE_CLUBS
    };
    (TC) => {
        $crate::decks::basic::card::TEN_CLUBS
    };
    (JC) => {
        $crate::decks::basic::card::JACK_CLUBS
    };
    (QC) => {
        $crate::decks::basic::card::QUEEN_CLUBS
    };
    (KC) => {
        $crate::decks::basic::card::KING_CLUBS
    };
    (FOOL) => {
        $crate::decks::tarot::card::FOOL
    };
    (MAGICIAN) => {
        $crate::decks::tarot::card::MAGICIAN
    };
    (HIGH_PRIESTESS) => {
        $crate::decks::tarot::card::HIGH_PRIESTESS
    };
    (EMPRESS) => {
        $crate::decks::tarot::card::EMPRESS
    };
    (EMPEROR) => {
        $crate::decks::tarot::card::EMPEROR
    };
    (HIEROPHANT) => {
        $crate::decks::tarot::card::HIEROPHANT
    };
    (LOVERS) => {
        $crate::decks::tarot::card::LOVERS
    };
    (THE_CHARIOT) => {
        $crate::decks::tarot::card::THE_CHARIOT
    };
    (STRENGTH) => {
        $crate::decks::tarot::card::STRENGTH
    };
    (HERMIT) => {
        $crate::decks::tarot::card::HERMIT
    };
    (WHEEL_OF_FORTUNE) => {
        $crate::decks::tarot::card::WHEEL_OF_FORTUNE
    };
    (JUSTICE) => {
        $crate::decks::tarot::card::JUSTICE
    };
    (HANGED_MAN) => {
        $crate::decks::tarot::card::HANGED_MAN
    };
    (DEATH) => {
        $crate::decks::tarot::card::DEATH
    };
    (TEMPERANCE) => {
        $crate::decks::tarot::card::TEMPERANCE
    };
    (DEVIL) => {
        $crate::decks::tarot::card::DEVIL
    };
    (TOWER) => {
        $crate::decks::tarot::card::TOWER
    };
    (STAR) => {
        $crate::decks::tarot::card::STAR
    };
    (MOON) => {
        $crate::decks::tarot::card::MOON
    };
    (SUN) => {
        $crate::decks::tarot::card::SUN
    };
    (JUDGEMENT) => {
        $crate::decks::tarot::card::JUDGEMENT
    };
    (WORLD) => {
        $crate::decks::tarot::card::WORLD
    };

    (PLUTO) => {
        $crate::decks::planet::card::PLUTO
    };
    (MERCURY) => {
        $crate::decks::planet::card::MERCURY
    };
    (URANUS) => {
        $crate::decks::planet::card::URANUS
    };
    (VENUS) => {
        $crate::decks::planet::card::VENUS
    };
    (SATURN) => {
        $crate::decks::planet::card::SATURN
    };
    (JUPITER) => {
        $crate::decks::planet::card::JUPITER
    };
    (EARTH) => {
        $crate::decks::planet::card::EARTH
    };
    (MARS) => {
        $crate::decks::planet::card::MARS
    };
    (NEPTUNE) => {
        $crate::decks::planet::card::NEPTUNE
    };
    (PLANET_X) => {
        $crate::decks::planet::card::PLANET_X
    };
    (CERES) => {
        $crate::decks::planet::card::CERES
    };
    (ERIS) => {
        $crate::decks::planet::card::ERIS
    };

    (JOKER) => {
        $crate::decks::joker::card::JOKER
    };
    (GREEDY) => {
        $crate::decks::joker::card::GREEDY_JOKER
    };
    (LUSTY) => {
        $crate::decks::joker::card::LUSTY_JOKER
    };
    (WRATHFUL) => {
        $crate::decks::joker::card::WRATHFUL_JOKER
    };
    (GLUTTONOUS) => {
        $crate::decks::joker::card::GLUTTONOUS_JOKER
    };
    (JOLLY) => {
        $crate::decks::joker::card::JOLLY_JOKER
    };
    (ZANY) => {
        $crate::decks::joker::card::ZANY_JOKER
    };
    (MAD) => {
        $crate::decks::joker::card::MAD_JOKER
    };
    (CRAZY) => {
        $crate::decks::joker::card::CRAZY_JOKER
    };
    (DROLL) => {
        $crate::decks::joker::card::DROLL_JOKER
    };
    (SLY) => {
        $crate::decks::joker::card::SLY_JOKER
    };
    (WILY) => {
        $crate::decks::joker::card::WILY_JOKER
    };
    (CLEVER) => {
        $crate::decks::joker::card::CLEVER_JOKER
    };
    (DEVIOUS) => {
        $crate::decks::joker::card::DEVIOUS_JOKER
    };
    (CRAFTY) => {
        $crate::decks::joker::card::CRAFTY_JOKER
    };
    (HALF) => {
        $crate::decks::joker::card::HALF_JOKER
    };
    (STENCIL) => {
        $crate::decks::joker::card::JOKER_STENCIL
    };
    (FOUR_FINGERS) => {
        $crate::decks::joker::card::FOUR_FINGERS
    };
    (MIME) => {
        $crate::decks::joker::card::MIME
    };
    (CREDIT_CARD) => {
        $crate::decks::joker::card::CREDIT_CARD
    };
    (CEREMONIAL_DAGGER) => {
        $crate::decks::joker::card::CEREMONIAL_DAGGER
    };
    (BANNER) => {
        $crate::decks::joker::card::BANNER
    };
    (MARBLE) => {
        $crate::decks::joker::card::MARBLE_JOKER
    };
    (LOYALTY_CARD) => {
        $crate::decks::joker::card::LOYALTY_CARD
    };
    (EIGHT_BALL) => {
        $crate::decks::joker::card::EIGHT_BALL
    };
    (MISPRINT) => {
        $crate::decks::joker::card::MISPRINT
    };
    (DUSK) => {
        $crate::decks::joker::card::MISPRINT
    };
    (RAISED_FIST) => {
        $crate::decks::joker::card::RAISED_FIST
    };
    (CHAOS_THE_CLOWN) => {
        $crate::decks::joker::card::CHAOS_THE_CLOWN
    };
    (FIBONACCI) => {
        $crate::decks::joker::card::FIBONACCI
    };
    (STEEL_JOKER) => {
        $crate::decks::joker::card::STEEL_JOKER
    };
    (SCARY_FACE) => {
        $crate::decks::joker::card::SCARY_FACE
    };
    (ABSTRACT_JOKER) => {
        $crate::decks::joker::card::ABSTRACT_JOKER
    };
    (DELAYED_GRATIFICATION) => {
        $crate::decks::joker::card::DELAYED_GRATIFICATION
    };
    (PAREIDOLIA) => {
        $crate::decks::joker::card::PAREIDOLIA
    };
    (GROS_MICHEL) => {
        $crate::decks::joker::card::GROS_MICHEL
    };
    (ODD_TODD) => {
        $crate::decks::joker::card::ODD_TODD
    };
    (SCHOLAR) => {
        $crate::decks::joker::card::SCHOLAR
    };
    (BUSINESS_CARD) => {
        $crate::decks::joker::card::BUSINESS_CARD
    };
    (SUPERNOVA) => {
        $crate::decks::joker::card::SUPERNOVA
    };
    (RIDE_THE_BUS) => {
        $crate::decks::joker::card::RIDE_THE_BUS
    };
    (SPACE_JOKER) => {
        $crate::decks::joker::card::SPACE_JOKER
    };
    (EGG) => {
        $crate::decks::joker::card::EGG
    };
    (BURGLAR) => {
        $crate::decks::joker::card::BURGLAR
    };
    (BLACKBOARD) => {
        $crate::decks::joker::card::BLACKBOARD
    };
    (RUNNER) => {
        $crate::decks::joker::card::RUNNER
    };
    (ICE_CREAM) => {
        $crate::decks::joker::card::ICE_CREAM
    };
    (DNA) => {
        $crate::decks::joker::card::DNA
    };
    (SPLASH) => {
        $crate::decks::joker::card::SPLASH
    };
    (BLUE_JOKER) => {
        $crate::decks::joker::card::BLUE_JOKER
    };
    (SIXTH_SENSE) => {
        $crate::decks::joker::card::SIXTH_SENSE
    };
    (CONSTELLATION) => {
        $crate::decks::joker::card::CONSTELLATION
    };
    (HIKER) => {
        $crate::decks::joker::card::HIKER
    };
    (FACELESS_JOKER) => {
        $crate::decks::joker::card::FACELESS_JOKER
    };
    (GREEN_JOKER) => {
        $crate::decks::joker::card::GREEN_JOKER
    };
    (SUPERPOSITION) => {
        $crate::decks::joker::card::SUPERPOSITION
    };
    (TO_DO_LIST) => {
        $crate::decks::joker::card::TO_DO_LIST
    };
    (CAVENDISH) => {
        $crate::decks::joker::card::CAVENDISH
    };
    (CARD_SHARP) => {
        $crate::decks::joker::card::CARD_SHARP
    };
    (RED_CARD) => {
        $crate::decks::joker::card::RED_CARD
    };
    (MADNESS) => {
        $crate::decks::joker::card::MADNESS
    };
    (SQUARE_JOKER) => {
        $crate::decks::joker::card::SQUARE_JOKER
    };
    (SEANCE) => {
        $crate::decks::joker::card::SEANCE
    };
    (RIFF_RAFF) => {
        $crate::decks::joker::card::RIFF_RAFF
    };
    (VAMPIRE) => {
        $crate::decks::joker::card::VAMPIRE
    };
    (SHORTCUT) => {
        $crate::decks::joker::card::SHORTCUT
    };
    (HOLOGRAM) => {
        $crate::decks::joker::card::HOLOGRAM
    };
    (VAGABOND) => {
        $crate::decks::joker::card::VAGABOND
    };
    (BARON) => {
        $crate::decks::joker::card::BARON
    };
    (CLOUD_9) => {
        $crate::decks::joker::card::CLOUD_9
    };
    (ROCKET) => {
        $crate::decks::joker::card::ROCKET
    };
    (EROSION) => {
        $crate::decks::joker::card::EROSION
    };
    (RESERVED_PARKING) => {
        $crate::decks::joker::card::RESERVED_PARKING
    };
    (MAIL_IN_REBATE) => {
        $crate::decks::joker::card::MAIL_IN_REBATE
    };
    (TO_THE_MOON) => {
        $crate::decks::joker::card::TO_THE_MOON
    };
    (HALLUCINATION) => {
        $crate::decks::joker::card::HALLUCINATION
    };
    (FORTUNE_TELLER) => {
        $crate::decks::joker::card::FORTUNE_TELLER
    };
    (JUGGLER) => {
        $crate::decks::joker::card::JUGGLER
    };
    (DRUNKARD) => {
        $crate::decks::joker::card::DRUNKARD
    };
    (STONE_JOKER) => {
        $crate::decks::joker::card::STONE_JOKER
    };
    (GOLDEN_JOKER) => {
        $crate::decks::joker::card::GOLDEN_JOKER
    };
    (LUCKY_CAT) => {
        $crate::decks::joker::card::LUCKY_CAT
    };
    (BASEBALL_CARD) => {
        $crate::decks::joker::card::BASEBALL_CARD
    };
    (BULL) => {
        $crate::decks::joker::card::BULL
    };
    (DIET_COLA) => {
        $crate::decks::joker::card::DIET_COLA
    };
    (TRADING_CARD) => {
        $crate::decks::joker::card::TRADING_CARD
    };
    (FLASH_CARD) => {
        $crate::decks::joker::card::FLASH_CARD
    };
    (POPCORN) => {
        $crate::decks::joker::card::POPCORN
    };
    (SPARE_TROUSERS) => {
        $crate::decks::joker::card::SPARE_TROUSERS
    };
    (ANCIENT_JOKER) => {
        $crate::decks::joker::card::ANCIENT_JOKER
    };
    (RAMEN) => {
        $crate::decks::joker::card::RAMEN
    };
    (WALKIE_TALKIE) => {
        $crate::decks::joker::card::WALKIE_TALKIE
    };
}

#[macro_export]
#[allow(clippy::pedantic)]
macro_rules! bcards {
    ($card_str:expr) => {
        BuffoonPile::forgiving_from_str($card_str)
    };
}

#[cfg(test)]
#[allow(non_snake_case)]
mod funky__macros_tests {
    use crate::prelude::*;

    #[test]
    fn bcard() {
        assert_eq!(bcard!(AS), ACE_SPADES);
        assert_eq!(bcard!(2S), DEUCE_SPADES);
        assert_eq!(bcard!(3S), TREY_SPADES);
        assert_eq!(bcard!(4S), FOUR_SPADES);
        assert_eq!(bcard!(5S), FIVE_SPADES);
        assert_eq!(bcard!(6S), SIX_SPADES);
        assert_eq!(bcard!(7S), SEVEN_SPADES);
        assert_eq!(bcard!(8S), EIGHT_SPADES);
        assert_eq!(bcard!(9S), NINE_SPADES);
        assert_eq!(bcard!(TS), TEN_SPADES);
        assert_eq!(bcard!(JS), JACK_SPADES);
        assert_eq!(bcard!(QS), QUEEN_SPADES);
        assert_eq!(bcard!(KS), KING_SPADES);
        assert_eq!(bcard!(AD), ACE_DIAMONDS);
        assert_eq!(bcard!(2D), DEUCE_DIAMONDS);
        assert_eq!(bcard!(3D), TREY_DIAMONDS);
        assert_eq!(bcard!(4D), FOUR_DIAMONDS);
        assert_eq!(bcard!(5D), FIVE_DIAMONDS);
        assert_eq!(bcard!(6D), SIX_DIAMONDS);
        assert_eq!(bcard!(7D), SEVEN_DIAMONDS);
        assert_eq!(bcard!(8D), EIGHT_DIAMONDS);
        assert_eq!(bcard!(9D), NINE_DIAMONDS);
        assert_eq!(bcard!(TD), TEN_DIAMONDS);
        assert_eq!(bcard!(JD), JACK_DIAMONDS);
        assert_eq!(bcard!(QD), QUEEN_DIAMONDS);
        assert_eq!(bcard!(KD), KING_DIAMONDS);
        assert_eq!(bcard!(AH), ACE_HEARTS);
        assert_eq!(bcard!(2H), DEUCE_HEARTS);
        assert_eq!(bcard!(3H), TREY_HEARTS);
        assert_eq!(bcard!(4H), FOUR_HEARTS);
        assert_eq!(bcard!(5H), FIVE_HEARTS);
        assert_eq!(bcard!(6H), SIX_HEARTS);
        assert_eq!(bcard!(7H), SEVEN_HEARTS);
        assert_eq!(bcard!(8H), EIGHT_HEARTS);
        assert_eq!(bcard!(9H), NINE_HEARTS);
        assert_eq!(bcard!(TH), TEN_HEARTS);
        assert_eq!(bcard!(JH), JACK_HEARTS);
        assert_eq!(bcard!(QH), QUEEN_HEARTS);
        assert_eq!(bcard!(KH), KING_HEARTS);
        assert_eq!(bcard!(AC), ACE_CLUBS);
        assert_eq!(bcard!(2C), DEUCE_CLUBS);
        assert_eq!(bcard!(3C), TREY_CLUBS);
        assert_eq!(bcard!(4C), FOUR_CLUBS);
        assert_eq!(bcard!(5C), FIVE_CLUBS);
        assert_eq!(bcard!(6C), SIX_CLUBS);
        assert_eq!(bcard!(7C), SEVEN_CLUBS);
        assert_eq!(bcard!(8C), EIGHT_CLUBS);
        assert_eq!(bcard!(9C), NINE_CLUBS);
        assert_eq!(bcard!(TC), TEN_CLUBS);
        assert_eq!(bcard!(JC), JACK_CLUBS);
        assert_eq!(bcard!(QC), QUEEN_CLUBS);
        assert_eq!(bcard!(KC), KING_CLUBS);
        assert_eq!(bcard!(FOOL), FOOL);
        assert_eq!(bcard!(MAGICIAN), MAGICIAN);
        assert_eq!(bcard!(HIGH_PRIESTESS), HIGH_PRIESTESS);
        assert_eq!(bcard!(EMPRESS), EMPRESS);
        assert_eq!(bcard!(EMPEROR), EMPEROR);
        assert_eq!(bcard!(HIEROPHANT), HIEROPHANT);
        assert_eq!(bcard!(LOVERS), LOVERS);
        assert_eq!(bcard!(THE_CHARIOT), THE_CHARIOT);
        assert_eq!(bcard!(STRENGTH), STRENGTH);
        assert_eq!(bcard!(HERMIT), HERMIT);
        assert_eq!(bcard!(WHEEL_OF_FORTUNE), WHEEL_OF_FORTUNE);
        assert_eq!(bcard!(JUSTICE), JUSTICE);
        assert_eq!(bcard!(HANGED_MAN), HANGED_MAN);
        assert_eq!(bcard!(DEATH), DEATH);
        assert_eq!(bcard!(TEMPERANCE), TEMPERANCE);
        assert_eq!(bcard!(DEVIL), DEVIL);
        assert_eq!(bcard!(TOWER), TOWER);
        assert_eq!(bcard!(STAR), STAR);
        assert_eq!(bcard!(MOON), MOON);
        assert_eq!(bcard!(SUN), SUN);
        assert_eq!(bcard!(JUDGEMENT), JUDGEMENT);
        assert_eq!(bcard!(WORLD), WORLD);
        assert_eq!(bcard!(GREEDY), GREEDY_JOKER);
        assert_eq!(bcard!(LUSTY), LUSTY_JOKER);
        assert_eq!(bcard!(WRATHFUL), WRATHFUL_JOKER);
        assert_eq!(bcard!(GLUTTONOUS), GLUTTONOUS_JOKER);
        assert_eq!(bcard!(JOLLY), JOLLY_JOKER);
        assert_eq!(bcard!(ZANY), ZANY_JOKER);
        assert_eq!(bcard!(MAD), MAD_JOKER);
    }
}
