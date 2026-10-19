//! Run it with `cargo run --example buffoon`.

use buffoonery::prelude::*;

/// End-to-end demonstration of funky (Balatro-style) card scoring.
///
/// It builds a board, plays a hand, holds a card, detects the poker-hand type,
/// then runs the full four-phase scoring pipeline and prints the chips × mult:
///   * **phase 1** — base chips/mult from the hand type at its current level;
///   * **phase 2** — played-card chips (base rank + enhancements);
///   * **phase 3** — held-card ×mult (a Steel card held in hand);
///   * **phase 4** — joker contributions;
///   * `board.score()` — the combined pipeline a solver would call;
///   * `board.score_traced()` — the same score, step by step (EPIC-05 Story 1).
fn main() {
    env_logger::init();

    // Deal a fresh, shuffled Buffoon deck onto a board with 4 hands / 3 discards.
    let draws = Draws::new(4, 3);
    let mut board = BuffoonBoard::new(draws, Deck::basic_buffoon_pile().shuffled());

    // Play a royal flush — it satisfies both the "straight" and "flush"
    // conditions the jokers below care about.
    board.played = bcards!("AS KS QS JS TS");

    // Hold a Steel King in hand: Steel gives x1.5 mult while held (phase 3).
    board.in_hand = BuffoonPile::from(vec![BuffoonCard {
        enhancement: MPip::STEEL,
        ..KING_HEARTS
    }]);

    // Four jokers whose effects are wired and scored today. A royal flush is
    // both a straight and a flush, so all four fire:
    //   CRAZY   = MPip::MultPlusOnStraight(12)
    //   DROLL   = MPip::MultPlusOnFlush(10)
    //   DEVIOUS = MPip::ChipsOnStraight(100)
    //   CRAFTY  = MPip::ChipsOnFlush(80)
    board.jokers.push(bcard!(CRAZY));
    board.jokers.push(bcard!(DROLL));
    board.jokers.push(bcard!(DEVIOUS));
    board.jokers.push(bcard!(CRAFTY));

    // Each phase folds into the running score, in Balatro order.
    let hand_type = board.played.determine_hand_type();
    let base = board.scoring_phase1_pre_scoring();
    let after_cards = board.scoring_phase2_dealt_hand_scoring(base);
    let after_held = board.scoring_phase3_effects_in_hand(after_cards);
    let total = board.scoring_phase4_joker_scoring(after_held); // == board.score()

    println!("Played hand : {}", board.played);
    println!("Held in hand: {}", board.in_hand);
    println!("Hand type   : {hand_type:?}");
    println!("Jokers      : {}", board.jokers);
    println!("Phase 1 (base)             {base}");
    println!("Phase 2 (+played cards)    {after_cards}");
    println!("Phase 3 (+held x mult)     {after_held}");
    println!("Phase 4 (+jokers) = final  {total}");

    // The same score as a trace: one row per op that changed the score.
    let trace = board.score_traced();
    println!();
    println!(
        "Trace ({} steps, {} fired):",
        trace.steps.len(),
        fired(&trace)
    );
    for step in trace.steps.iter().filter(|step| step.fired) {
        println!(
            "  {:<7} {:<16} {:<8} {:<16} -> {} chips x {} mult",
            format!("{:?}", step.phase),
            source(&step.source),
            format!("{:?}", step.part),
            step.op.to_string(),
            step.after.chips,
            step.after.mult,
        );
    }
    println!();
    println!("Final score = {}", total.score());
}

fn fired(trace: &ScoreTrace) -> usize {
    trace.steps.iter().filter(|step| step.fired).count()
}

fn source(source: &StepSource) -> String {
    match source {
        StepSource::Hand { hand_type, level } => format!("{hand_type:?} L{level}"),
        StepSource::Card { card, .. } | StepSource::Joker { card, .. } => card.to_string(),
    }
}
