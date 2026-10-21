//! A [`ScoreTrace`] as a text table: the same rows the page shows.
//!
//! Presentation only. Every chips, mult and score cell is a field of a
//! [`ScoreStep`], never a sum made here; the
//! score cell is [`Score::score`], the same `floor(chips × mult)` the page
//! shows.

use buffoonery::prelude::*;

/// One printed row of a trace.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    /// The step's number in the trace, from 1.
    pub n: usize,
    /// What made the step: `Pair (level 1)`, `A♠ · played 1`,
    /// `Jolly Joker · slot 1`.
    pub source: String,
    /// The part (when not the effect), the retrigger pass, and `rolled`.
    pub detail: String,
    /// The op as the breakdown writes it, or why nothing happened.
    pub effect: String,
    pub chips: u64,
    pub mult: f64,
    pub score: f64,
}

/// The rows the page's `rows()` shows: a step that changed the score, or a
/// joker's effect (so "did not fire" stays visible). Every step when `all`
/// is true, as the page's "Show every step" does.
#[must_use]
pub fn rows(trace: &ScoreTrace, all: bool) -> Vec<Row> {
    trace
        .steps
        .iter()
        .enumerate()
        .filter(|(_, step)| {
            all || step.fired || (step.phase == Phase::Joker && step.part == StepPart::Effect)
        })
        .map(|(i, step)| Row {
            n: i + 1,
            source: source(step),
            detail: detail(step),
            effect: effect(step),
            chips: step.after.chips,
            mult: step.after.mult,
            score: step.after.score(),
        })
        .collect()
}

/// The rows as fixed-width columns, then a total line:
/// `Pair  82 × 10 = 820`.
#[must_use]
pub fn render(rows: &[Row], trace: &ScoreTrace) -> String {
    let mut cells = vec![
        ["#", "Source", "Detail", "Effect", "Chips", "Mult", "Score"]
            .map(String::from)
            .to_vec(),
    ];
    for row in rows {
        cells.push(vec![
            row.n.to_string(),
            row.source.clone(),
            row.detail.clone(),
            row.effect.clone(),
            row.chips.to_string(),
            number(row.mult),
            number(row.score),
        ]);
    }
    let right = [true, false, false, false, true, true, true];
    format!(
        "{}\n{}  {} × {} = {}",
        columns(&cells, &right),
        hand_label(trace.hand_type),
        trace.total.chips,
        number(trace.total.mult),
        number(trace.total.score())
    )
}

/// Columns two spaces apart, each left- or right-aligned, no trailing spaces.
pub(crate) fn columns(rows: &[Vec<String>], right: &[bool]) -> String {
    let count = rows.iter().map(Vec::len).max().unwrap_or(0);
    let widths: Vec<usize> = (0..count)
        .map(|i| {
            rows.iter()
                .filter_map(|row| row.get(i))
                .map(|cell| cell.chars().count())
                .max()
                .unwrap_or(0)
        })
        .collect();
    rows.iter()
        .map(|row| {
            let line: Vec<String> = row
                .iter()
                .zip(&widths)
                .enumerate()
                .map(|(i, (cell, &width))| {
                    if right.get(i).copied().unwrap_or(false) {
                        format!("{cell:>width$}")
                    } else {
                        format!("{cell:<width$}")
                    }
                })
                .collect();
            line.join("  ").trim_end().to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A number as the page shows it: at most 2 decimals. No thousands
/// separators, so a script can read it back.
fn number(n: f64) -> String {
    let rounded = (n * 100.0).round() / 100.0;
    format!("{}", rounded + 0.0)
}

fn source(step: &ScoreStep) -> String {
    match step.source {
        StepSource::Hand { hand_type, level } => {
            format!("{} (level {level})", hand_label(hand_type))
        }
        StepSource::Card { card, index } => {
            let phase = if step.phase == Phase::Held {
                "held"
            } else {
                "played"
            };
            format!("{} · {phase} {}", card_label(&card), index + 1)
        }
        StepSource::Joker { card, index } => format!(
            "{} · slot {}",
            Joker::name(&card).unwrap_or("Joker"),
            index + 1
        ),
    }
}

fn detail(step: &ScoreStep) -> String {
    let mut parts = Vec::new();
    match step.part {
        StepPart::Effect => {}
        StepPart::Special => parts.push("special".to_string()),
        StepPart::Edition => parts.push("edition".to_string()),
    }
    if step.retrigger > 0 {
        parts.push(format!("retrigger {}", step.retrigger));
    }
    if step.rolled {
        parts.push("rolled".to_string());
    }
    parts.join(", ")
}

/// The op, or what a step that changed nothing says: a joker did not fire
/// (or is debuffed); a card's part is nothing.
fn effect(step: &ScoreStep) -> String {
    if step.fired {
        return step.op.to_string();
    }
    match step.source {
        StepSource::Joker { card, .. } if card.debuffed => "debuffed".to_string(),
        StepSource::Joker { .. } => "did not fire".to_string(),
        _ => "nothing".to_string(),
    }
}

fn card_label(card: &BuffoonCard) -> String {
    let suit = match card.suit.index {
        'S' => '♠',
        'H' => '♥',
        'D' => '♦',
        'C' => '♣',
        other => other,
    };
    format!("{}{suit}", card.rank.index)
}

/// The page's hand names (`HAND_TYPES` in `web/src/model.js`).
fn hand_label(hand_type: HandType) -> &'static str {
    match hand_type {
        HandType::HighCard => "High Card",
        HandType::Pair => "Pair",
        HandType::TwoPair => "Two Pair",
        HandType::ThreeOfAKind => "Three of a Kind",
        HandType::Straight => "Straight",
        HandType::Flush => "Flush",
        HandType::FullHouse => "Full House",
        HandType::FourOfAKind => "Four of a Kind",
        HandType::StraightFlush => "Straight Flush",
        HandType::RoyalFlush => "Royal Flush",
        HandType::FiveOfAKind => "Five of a Kind",
        HandType::FlushHouse => "Flush House",
        HandType::FlushFive => "Flush Five",
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::path::Path;

    fn trace(fixture: &str) -> ScoreTrace {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../fixtures/golden")
            .join(fixture);
        let spec = BoardSpec::from_yaml(&std::fs::read_to_string(path).unwrap()).unwrap();
        BuffoonBoard::try_from(&spec).unwrap().score_traced()
    }

    #[test]
    fn rows_hides_noop_card_steps() {
        let trace = trace("02_pair_additive_jokers.yaml");
        let shown = rows(&trace, false);
        let hidden = trace
            .steps
            .iter()
            .filter(|step| {
                !step.fired && (step.phase != Phase::Joker || step.part != StepPart::Effect)
            })
            .count();
        assert!(hidden > 0, "fixture 02 has no-op card steps");
        assert_eq!(shown.len(), trace.steps.len() - hidden);
        assert!(shown.iter().all(|row| row.effect != "nothing"));
    }

    #[test]
    fn rows_keeps_unfired_joker_effects() {
        let trace = trace("10_failed_conditions.yaml");
        let unfired: Vec<Row> = rows(&trace, false)
            .into_iter()
            .filter(|row| row.effect == "did not fire")
            .collect();
        assert_ne!(unfired.len(), 0, "fixture 10 has jokers that do not fire");
        assert!(unfired.iter().all(|row| row.source.contains("· slot ")));
    }

    #[test]
    fn rows_all_keeps_every_step() {
        let trace = trace("02_pair_additive_jokers.yaml");
        let all = rows(&trace, true);
        assert_eq!(all.len(), trace.steps.len());
        for (row, step) in all.iter().zip(&trace.steps) {
            assert_eq!(row.chips, step.after.chips);
            assert_eq!(row.mult.to_bits(), step.after.mult.to_bits());
            assert_eq!(row.score.to_bits(), step.after.score().to_bits());
        }
    }

    #[test]
    fn rows_label_hand_card_and_joker() {
        let shown = rows(&trace("02_pair_additive_jokers.yaml"), false);
        assert_eq!(shown[0].source, "Pair (level 1)");
        assert_eq!(shown[0].n, 1);
        assert!(
            shown.iter().any(|row| row.source == "A♠ · played 1"),
            "{shown:?}"
        );
        assert!(
            shown
                .iter()
                .any(|row| row.source.starts_with("Jolly Joker · slot ")),
            "{shown:?}"
        );
    }

    #[test]
    fn rows_mark_retriggers_and_parts() {
        let all = rows(&trace("08_hack_retrigger_played.yaml"), true);
        assert!(
            all.iter().any(|row| row.detail.contains("retrigger 1")),
            "{all:?}"
        );
        assert!(
            all.iter().any(|row| row.detail.contains("edition")),
            "{all:?}"
        );
    }

    #[test]
    fn render_total_line_matches_score() {
        let trace = trace("02_pair_additive_jokers.yaml");
        let text = render(&rows(&trace, false), &trace);
        let last = text.lines().last().unwrap();
        assert_eq!(
            last,
            format!(
                "Pair  {} × {} = {}",
                trace.total.chips,
                trace.total.mult,
                trace.total.score()
            )
        );
    }

    #[test]
    fn render_has_a_header_and_one_line_per_row() {
        let trace = trace("02_pair_additive_jokers.yaml");
        let shown = rows(&trace, false);
        let text = render(&shown, &trace);
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[0].trim_start().starts_with("#  Source"), "{text}");
        assert_eq!(lines.len(), 1 + shown.len() + 1, "{text}");
    }
}
