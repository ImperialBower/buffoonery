# Golden fixtures

Each pair is one board and the trace the engine must return for it:

- `NN_name.yaml` — a `BoardSpec` (comments say what the board pins).
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

No fixture plays a kicker. The engine scores every played card, Balatro does
not (EPIC-05 risk 7), so a kicker would pin a known-wrong total.
