// The calculator's model (EPIC-05 Story 6): page state → BoardSpec, and
// ScoreTrace → what the page shows. No scoring happens here. Every chip and
// mult value comes from the engine's trace; the one sum this file does is
// `floor(chips × mult)`, the same expression as `Score::score()` in Rust, to
// show a running score.

/** The joker slots a board has. */
export const SLOT_COUNT = 5;

/**
 * A sample played hand per hand type, until Story 7's hand editor. These are
 * cards only: the engine finds the hand type and its base chips and mult.
 * No Royal Flush: the engine scores one as a Straight Flush, as Balatro does.
 */
export const SAMPLE_HANDS = [
  { type: 'HighCard', label: 'High Card', played: ['AS'] },
  { type: 'Pair', label: 'Pair', played: ['AS', 'AH'] },
  { type: 'TwoPair', label: 'Two Pair', played: ['AS', 'AH', 'KS', 'KH'] },
  { type: 'ThreeOfAKind', label: 'Three of a Kind', played: ['AS', 'AH', 'AD'] },
  { type: 'Straight', label: 'Straight', played: ['9S', 'TH', 'JD', 'QC', 'KS'] },
  { type: 'Flush', label: 'Flush', played: ['AH', 'JH', '9H', '6H', '3H'] },
  { type: 'FullHouse', label: 'Full House', played: ['AS', 'AH', 'AD', 'KS', 'KH'] },
  { type: 'FourOfAKind', label: 'Four of a Kind', played: ['AS', 'AH', 'AD', 'AC'] },
  { type: 'StraightFlush', label: 'Straight Flush', played: ['9H', 'TH', 'JH', 'QH', 'KH'] },
  { type: 'FiveOfAKind', label: 'Five of a Kind', played: ['AS', 'AH', 'AD', 'AC', 'AS'] },
  { type: 'FlushHouse', label: 'Flush House', played: ['AH', 'AH', 'AH', 'KH', 'KH'] },
  { type: 'FlushFive', label: 'Flush Five', played: ['AS', 'AS', 'AS', 'AS', 'AS'] },
];

/** The override kinds the effect editor offers. `null` means "as printed". */
export const OVERRIDE_KINDS = [
  { kind: null, label: 'As printed' },
  { kind: 'chips', label: '+ Chips' },
  { kind: 'mult', label: '+ Mult' },
  { kind: 'xmult', label: '× Mult' },
];

/** The board the page opens with: Story 1's acceptance board. */
export function initialState() {
  const slot = (ident) => ({ ident, override: null, debuffed: false });
  return {
    hand: 'Flush',
    slots: [slot('JOKER'), slot('DROLL_JOKER'), slot('CRAFTY_JOKER'), slot('CAVENDISH'), slot(null)],
    showAll: false,
  };
}

/**
 * The `MPip` an override writes onto a joker, and the value it really holds.
 * Chips and mult are whole numbers. A whole ×mult is `MultTimes(n)`; any other
 * ×mult is `MultTimes1Dot`, in tenths, so it is rounded to 0.1.
 */
export function overrideEnhancement(override) {
  const raw = Number(override.value);
  const value = Number.isFinite(raw) ? Math.max(0, raw) : 0;
  switch (override.kind) {
    case 'chips': {
      const n = Math.round(value);
      return { enhancement: { Chips: n }, value: n };
    }
    case 'mult': {
      const n = Math.round(value);
      return { enhancement: { MultPlus: n }, value: n };
    }
    case 'xmult': {
      if (Number.isInteger(value)) return { enhancement: { MultTimes: value }, value };
      const tenths = Math.round(value * 10);
      return { enhancement: { MultTimes1Dot: tenths }, value: tenths / 10 };
    }
    default:
      return null;
  }
}

/**
 * The `BoardSpec` for the page state, and which trace joker index each slot
 * became (`null` for an empty slot). Empty slots are left out of the spec.
 */
export function buildSpec(state) {
  const hand = SAMPLE_HANDS.find((h) => h.type === state.hand) ?? SAMPLE_HANDS[0];
  const jokers = [];
  const jokerIndex = state.slots.map((slot) => {
    if (!slot.ident) return null;
    const joker = { ident: slot.ident };
    const override = slot.override && overrideEnhancement(slot.override);
    if (override) joker.enhancement = override.enhancement;
    if (slot.debuffed) joker.debuffed = true;
    jokers.push(joker);
    return jokers.length - 1;
  });
  return { spec: { played: [...hand.played], jokers }, jokerIndex };
}

/** `floor(chips × mult)`, as `Score::score()` computes it. */
export function handScore(score) {
  return Math.floor(score.chips * score.mult);
}

/** A number as the page shows it: at most 2 decimals, with separators. */
export function fmtNum(n) {
  return (Math.round(n * 100) / 100).toLocaleString('en-US', { maximumFractionDigits: 2 });
}

/** The row text for a `ScoreOp`, as Rust's `Display` writes it. */
export function formatOp(op) {
  if (op === 'Nothing') return 'nothing';
  if ('AddChips' in op) return `+${op.AddChips} Chips`;
  if ('AddMult' in op) return `+${op.AddMult} Mult`;
  if ('Add' in op) {
    const parts = [];
    if (op.Add.chips > 0) parts.push(`+${op.Add.chips} Chips`);
    if (Math.abs(op.Add.mult) > 0) parts.push(`+${op.Add.mult} Mult`);
    return parts.length ? parts.join(' ') : 'nothing';
  }
  if ('TimesMult' in op) return `×${op.TimesMult} Mult`;
  if ('Seq' in op) return op.Seq.map(formatOp).join(', ');
  return JSON.stringify(op);
}

/** What kind of change an op made: `chips`, `mult`, `xmult` or `none`. */
export function opKind(op) {
  if (op === 'Nothing') return 'none';
  if ('AddChips' in op) return op.AddChips > 0 ? 'chips' : 'none';
  if ('AddMult' in op) return op.AddMult > 0 ? 'mult' : 'none';
  if ('Add' in op) {
    if (Math.abs(op.Add.mult) > 0) return 'mult';
    return op.Add.chips > 0 ? 'chips' : 'none';
  }
  if ('TimesMult' in op) return op.TimesMult === 1 ? 'none' : 'xmult';
  if ('Seq' in op) return op.Seq.map(opKind).find((k) => k !== 'none') ?? 'none';
  return 'none';
}

/** The big number on a joker card: `+80`, `×3`, or `—` when nothing fired. */
export function opHeadline(op) {
  const kind = opKind(op);
  if (kind === 'none') return '—';
  if ('TimesMult' in op) return `×${fmtNum(op.TimesMult)}`;
  if ('AddChips' in op) return `+${fmtNum(op.AddChips)}`;
  if ('AddMult' in op) return `+${fmtNum(op.AddMult)}`;
  if ('Add' in op) return `+${fmtNum(kind === 'mult' ? op.Add.mult : op.Add.chips)}`;
  return formatOp(op);
}

/** An `MPip` as text: `MultPlusOnPair(8)`, `Lucky(5, 15)`, `Blank`. */
export function formatMPip(mpip) {
  if (typeof mpip === 'string') return mpip;
  const [name, value] = Object.entries(mpip)[0];
  const args = Array.isArray(value) ? value : [value];
  return `${name}(${args.map((a) => (Array.isArray(a) ? `[${a.join(', ')}]` : a)).join(', ')})`;
}

const SUIT = { S: '♠', H: '♥', D: '♦', C: '♣' };
const PHASE_LABEL = { Played: 'played', Held: 'held' };

function cardLabel(card) {
  return `${card.rank.index}${SUIT[card.suit.index] ?? card.suit.index}`;
}

/**
 * One table row per trace step. A row is hidden unless it changed the score,
 * is a joker's effect (so "did not fire" stays visible), or `showAll` is on.
 * `jokerNames[i]` names the joker at trace index `i`.
 */
export function rows(trace, jokerNames, showAll = false) {
  return trace.steps
    .map((step, i) => {
      let source;
      if ('Hand' in step.source) {
        const { hand_type: type, level } = step.source.Hand;
        const label = SAMPLE_HANDS.find((h) => h.type === type)?.label ?? type;
        source = `${label} (level ${level})`;
      } else if ('Card' in step.source) {
        const { card, index } = step.source.Card;
        source = `${cardLabel(card)} · ${PHASE_LABEL[step.phase]} ${index + 1}`;
      } else {
        const { index } = step.source.Joker;
        source = `${jokerNames[index] ?? 'Joker'} · slot ${index + 1}`;
      }
      const detail = [step.part === 'Effect' ? '' : step.part.toLowerCase(), step.retrigger ? `retrigger ${step.retrigger}` : '']
        .filter(Boolean)
        .join(', ');
      return {
        n: i + 1,
        source,
        detail,
        effect: step.fired ? formatOp(step.op) : step.source.Joker?.card.debuffed ? 'debuffed' : 'did not fire',
        kind: opKind(step.op),
        fired: step.fired,
        chips: step.after.chips,
        mult: step.after.mult,
        score: handScore(step.after),
        visible: showAll || step.fired || (step.phase === 'Joker' && step.part === 'Effect'),
      };
    });
}

/**
 * What a joker card shows, from its catalog entry and its trace steps:
 * the effect that entered the math (or why none did) and its note.
 */
export function slotView(slot, info, steps) {
  if (!slot.ident || !info) {
    return { empty: true, headline: '—', kindLabel: 'empty', kind: 'none', note: 'No joker in this slot.' };
  }
  const effect = steps.find((s) => s.part === 'Effect');
  const edition = steps.find((s) => s.part === 'Edition' && s.fired);
  const kind = effect ? opKind(effect.op) : 'none';
  const override = slot.override && overrideEnhancement(slot.override);
  const printed = formatMPip(info.enhancement);
  let kindLabel;
  if (slot.debuffed) kindLabel = 'debuffed';
  else if (kind === 'none') kindLabel = 'did not fire';
  else kindLabel = kind === 'chips' ? 'Chips' : 'Mult';
  const notes = [override ? `Overridden: ${formatMPip(override.enhancement)} (printed: ${printed})` : printed];
  if (info.blank_reason) notes.push(`Not wired yet: ${info.blank_reason}`);
  if (edition) notes.push(`Edition: ${formatOp(edition.op)}`);
  return {
    empty: false,
    headline: effect && !slot.debuffed ? opHeadline(effect.op) : '—',
    kind: slot.debuffed ? 'none' : kind,
    kindLabel,
    fired: Boolean(effect?.fired),
    overridden: Boolean(override),
    overrideValue: override?.value,
    note: notes.join(' · '),
  };
}
