# Technical Debt

> Maintained by the `/backlog` skill. Items tagged 🤖 were proposed by automated
> review — review and edit them; they are suggestions, not facts.
> Created 2026-10-20 at `main` `4b337a5`. The overview of all open work is in
> [BACKLOG.md](../BACKLOG.md).

## Tracked debt
<!-- Human-authored and code-comment-sourced items. -->

The source has no `TODO` / `FIXME` / `HACK` / `XXX` comments. These items come
from "known gap" notes in the code and open tasks in the EPICs.

- [x] **Stale comment (EPIC-05 task 0i)** — said Joker Stencil was not
  applied yet. Fixed 2026-10-20; the comment now names only Loyalty Card.
  (`src/types/board/scoring.rs:361`)
- [x] **`blind_target` is `usize`** — overflows past 4.29 × 10⁹ on wasm32.
  EPIC-05 S1 follow-up. Fixed 2026-10-24 (EPIC-10 Phase 3): it is `f64`,
  the type of `round_score`. (`src/types/board.rs:92`)
- [ ] **Run-level tarots are no-ops** — Death, Judgement, The Hermit, The Wheel
  of Fortune. (`src/types/board.rs:531`)
- [ ] **Hiker bumps once per hand, not per trigger** — needs scoring to
  mutate. (`src/types/board.rs:2195`)
- [ ] **Destruction is data only** — Glass card breakage
  (`src/types/board/scoring.rs:184`); Gros Michel's end-of-round roll
  (`src/types/board/scoring.rs:496`).
- [ ] **8 Blank jokers** — each with its blocker in `Joker::BLANK_WITH_REASON`.
  (`src/decks/joker.rs:306`)
- [ ] **File paths in the `.wasm`** — ~6 KiB of build paths in panic messages,
  including the builder's home folder; `--remap-path-prefix` would shorten
  them. (`component/README.md`, Size)

## 🤖 Automated review findings
<!-- Machine-proposed. Promote good ones up to "Tracked debt", delete the rest. -->

From a read-only review on 2026-10-20 (no code was run by the reviewer).
"Checked" means a maintainer session read the code and confirmed the claim.

- [x] 🤖 **Four of a Kind base is 50 × 7; Balatro's is 60 × 7** — every Four of
  a Kind scores 10 chips low, and no test pins the base-hand table. Suggested:
  set 60, add a test for all 12 base values and a Four of a Kind golden
  fixture. (`src/types/hands.rs:76`) — *checked, fixed 2026-10-20*
- [x] 🤖 **Planet cards carry base values, not level-up values** — e.g. Jupiter
  `ChipsMultPlusOnHand(35, 4, Flush)` where `LEVEL_UP` says +15/+2; same for
  Saturn, Earth, Mars. `use_consumable` and `set_level` give different
  results; the comments contradict the numbers; `FlushHouse` has no planet.
  Suggested: build planet values from `LEVEL_UP`, test every planet against
  it. (`src/decks/planet.rs:147-182`, `src/types/hands.rs:130`) — *checked, fixed 2026-10-20*
- [x] 🤖 **Stone cards can make Two Pair and Full House** — `has_full_house`
  and `has_2pair` skip `detectable()`, unlike `has_x_of_a_kind`; a Stone card
  is an Ace underneath. Suggested: filter with `detectable()`, add Stone tests
  for both. (`src/types/buffoon_pile.rs:542`, `:598`) — *checked, fixed 2026-10-20*
- [x] 🤖 **Joker Stencil counts Negative jokers as filling a slot** — uses
  `jokers.len()`, not `slots_taken()`. Suggested: use `slots_taken`, add a
  Negative + Stencil test. (`src/types/board/scoring.rs:750`,
  `src/types/board.rs:492`) — *checked, fixed 2026-10-20*
- [ ] 🤖 **Unwired jokers score 0 with no note** — Ceremonial Dagger, Loyalty
  Card, Supernova, Ride the Bus hold non-`Blank` effects with no scoring arm;
  the page shows "did not fire" without "Not wired yet". `BoardSpec` also
  accepts any `MPip`, including unwired ones. Suggested: move the test-only
  `scores_hand` classification into an `unwired_reason` helper, surface it in
  `JokerInfo`. (`src/decks/joker.rs:684,744,1033,1048`,
  `src/decks/joker_catalog.rs:198`)
- [ ] 🤖 **`remove_joker` can panic** — public, calls `BuffoonPile::remove`,
  which panics out of range (no `# Panics` doc); a wasm trap under
  `panic = "abort"`. Suggested: return `Option`. (`src/types/board.rs:284`,
  `src/types/buffoon_pile.rs:690`)
- [ ] 🤖 **`play_hand` accepts 0 or more than 5 cards** — `MAX_PLAYED` lives
  only in `BoardSpec`. Suggested: enforce it in the round loop too.
  (`src/types/board.rs:1013`, `src/types/board_spec.rs:42`)
- [ ] 🤖 **Unchecked `usize` arithmetic on user values** — `chips * ups`,
  `n * dollars`, and others wrap silently in the release-based component
  profile; the page's level box has no maximum. Suggested: saturating math,
  a level cap in `BoardSpec`, `overflow-checks = true` in `[profile.component]`.
  (`src/types/hands.rs:163`, `src/types/board/scoring.rs:448,488,759`)
- [ ] 🤖 **The Smeared suit rule has three copies** — a `matches!` table, an
  'R'/'B' closure, and red/black counts; the Wild-card fix would need all
  three. Suggested: one `HandRules::suit_group`. (`src/types/board/scoring.rs:572`,
  `src/types/buffoon_pile.rs:290`, `:367`)
- [ ] 🤖 **The page duplicates engine constants** — slot/played/held limits,
  hand and edition labels; `specToState` rejects boards the engine accepts
  (Negative jokers past 5). Suggested: export limits and edition ops from the
  component and build the page lists from them. (`web/src/model.js:8-45,97`)
- [ ] 🤖 **Specs accept non-playing cards** — `played: [FOOL]` or `"__"` parse
  and score as rank-less cards. Suggested: reject non-`Basic` card types as
  `UnknownCard`. (`src/types/board_spec.rs:226`)
- [ ] 🤖 **Purity and dead-code drift** — crate-wide `#![allow(dead_code)]`;
  ambient-entropy `shuffle`; `PokerHands` is a `HashMap` that serializes in
  unstable order. Suggested: drop the crate-wide allow, gate the entropy
  shuffles, use a `BTreeMap`. (`src/lib.rs:23`, `src/types/buffoon_pile.rs:697`,
  `src/types/hands.rs:51`)
- [ ] 🤖 **Counter jokers live in four parallel tables** — read, write, decay
  and the test oracle must match one-for-one; `builtin_joker_op` and
  `joker_x_mult` are 100+ lines each. Suggested: one descriptor per `MPip`.
  (`src/types/board/scoring.rs:980`, `src/types/board.rs:819,2130`)
- [ ] 🤖 **Few doc tests and `# Errors` / `# Panics` sections** — ~190 `pub fn`,
  14 doc-test blocks; commented-out tests under "region garbage". Suggested:
  start with the board lifecycle and `Score` APIs; delete the dead test
  blocks. (`src/types/buffoon_pile.rs:1366-1474`)
- [ ] 🤖 **Small drift** — "MSRV is 1.85" comment (`src/types/board.rs:752`;
  it is 1.99.1); `catalog()` swallows a serialization error as `[]`
  (`component/src/lib.rs:159`); `BLESS=1` freezes behaviour, it does not
  check Balatro's numbers.
