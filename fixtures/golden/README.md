# Golden fixtures

Each pair is one board and the trace the engine must return for it:

- `NN_name.yaml` — a `BoardSpec` (comments say what the board pins). A joker
  is a map (`ident: BARON`, …) or a shorthand token (`BARON+FOIL`, EPIC-08).
- `NN_name.trace.json` — the `ScoreTrace` from `BuffoonBoard::score_traced()`
  (unseeded: Lucky cards and Misprint do not roll).

The Rust tests (`tests/golden.rs`), the WIT component tests and the browser
parity test all read these files as they are. A change here is a change to
the contract.

## After a deliberate scoring change

```bash
BLESS=1 cargo test --test golden
git diff fixtures/golden
```

Check every changed number by hand before you commit it. A blessed file
records what the engine does, not what Balatro does.

## Coverage

`golden__coverage` fails if the set loses any of: each `ScoreOp` variant but
`Seq`, a failed joker condition, a retriggered played card, a retriggered held
card, an edition that fires on a card and on a joker, a `Blank` joker.

`Seq` has no fixture: only a registered custom effect returns it, and a
`BoardSpec` cannot carry a registry.

Kickers do not score, as in Balatro: only the cards that make the hand do
(plus Stone cards, and every card under Splash). `15_kickers.yaml` pins that,
and `16_ace_low_straight.yaml` pins the Ace-low straight (EPIC-05 risks 7
and 8, fixed 2026-10-20). `17_four_fingers_bonus_jokers.yaml` pins that the
"contains a Flush" jokers see a Four Fingers flush. `18_four_of_a_kind.yaml`
pins the Four of a Kind base (60 × 7), and `19_stone_makes_no_pair.yaml` pins
that a Stone card pairs with nothing (both fixed 2026-10-20).
`20_shorthand_jokers.yaml` writes its jokers as tokens and pins ×3.25 as exact
hundredths (1,170; tenths would give 1,188).
`21_wild_flush.yaml` pins that a Wild card is every suit: four Hearts and a
Wild card are a Flush, and the Wild card pays Lusty and Greedy (2,272).
`22_stone_and_wild_suits.yaml` pins that a Stone card is no suit (it pays no
Wrathful) and that a held Wild card passes Blackboard (1,230). Both fixed
2026-10-24 by EPIC-10.

## Set size

`golden__set_size` accepts 10 to 30 fixtures (raised from 20 by EPIC-08).
