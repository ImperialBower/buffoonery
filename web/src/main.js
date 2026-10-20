// The Joker Calculator page (EPIC-05 Story 6). It draws what model.js derives
// from the engine's trace; it scores nothing itself.
import './style.css';
import { calculator } from './pkg/buffoonery_component.js';
import {
  OVERRIDE_KINDS,
  SAMPLE_HANDS,
  SLOT_COUNT,
  buildSpec,
  fmtNum,
  handScore,
  initialState,
  opKind,
  rows,
  slotView,
} from './model.js';

const catalog = JSON.parse(calculator.catalog());
const byIdent = new Map(catalog.map((joker) => [joker.ident, joker]));
const pickerOrder = [...catalog].sort((a, b) => a.name.localeCompare(b.name));

let state = initialState();
let lastSteps = [];

const $ = (id) => document.getElementById(id);

function el(tag, attrs = {}, children = []) {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(attrs)) {
    if (key === 'class') node.className = value;
    else if (key === 'text') node.textContent = value;
    else node.setAttribute(key, value);
  }
  node.append(...children);
  return node;
}

function update(change) {
  change(state);
  render();
}

// ---- Hand -----------------------------------------------------------------

const handSelect = $('hand-type');
for (const hand of SAMPLE_HANDS) handSelect.append(el('option', { value: hand.type, text: hand.label }));
handSelect.addEventListener('change', () => update((s) => { s.hand = handSelect.value; }));

$('show-all').addEventListener('change', (e) => update((s) => { s.showAll = e.target.checked; }));

// ---- Joker slots, built once so typing keeps focus --------------------------

/** The number an op carries for an override kind, to seed the value box. */
function seedValue(steps, kind) {
  const effect = steps.find((s) => s.part === 'Effect');
  const defaults = { chips: 0, mult: 0, xmult: 1 };
  if (!effect || opKind(effect.op) !== kind) return defaults[kind];
  const op = effect.op;
  if ('AddChips' in op) return op.AddChips;
  if ('AddMult' in op) return op.AddMult;
  if ('TimesMult' in op) return op.TimesMult;
  if ('Add' in op) return kind === 'chips' ? op.Add.chips : op.Add.mult;
  return defaults[kind];
}

const slotNodes = Array.from({ length: SLOT_COUNT }, (_, i) => {
  const picker = el('select', { 'aria-label': `Joker in slot ${i + 1}` }, [
    el('option', { value: '', text: 'Empty slot' }),
    ...pickerOrder.map((joker) => el('option', { value: joker.ident, text: joker.name })),
  ]);
  const debuff = el('input', { type: 'checkbox', 'aria-label': `Debuff slot ${i + 1}` });
  const kind = el('select', { 'aria-label': `Effect override, slot ${i + 1}` },
    OVERRIDE_KINDS.map((k) => el('option', { value: k.kind ?? '', text: k.label })));
  const value = el('input', { type: 'number', step: 'any', min: '0', 'aria-label': `Override value, slot ${i + 1}` });
  const headline = el('span', { class: 'headline' });
  const kindLabel = el('span', { class: 'kind-label' });
  const status = el('span', { class: 'status' });
  const badge = el('span', { class: 'badge', text: 'overridden', hidden: '' });
  const note = el('p', { class: 'note' });

  const card = el('article', { class: 'slot' }, [
    el('div', { class: 'band' }, [
      el('span', { class: 'slot-n', text: `SLOT ${i + 1}` }),
      status,
    ]),
    el('div', { class: 'body' }, [
      picker,
      el('div', { class: 'effect' }, [headline, kindLabel, badge]),
      note,
      el('div', { class: 'editors' }, [kind, value]),
      el('label', { class: 'check small' }, [debuff, el('span', { text: 'debuffed' })]),
    ]),
  ]);

  picker.addEventListener('change', () => update((s) => {
    s.slots[i] = { ident: picker.value || null, override: null, debuffed: false };
  }));
  debuff.addEventListener('change', () => update((s) => { s.slots[i].debuffed = debuff.checked; }));
  kind.addEventListener('change', () => update((s) => {
    const chosen = kind.value || null;
    s.slots[i].override = chosen ? { kind: chosen, value: seedValue(lastSteps[i] ?? [], chosen) } : null;
  }));
  value.addEventListener('input', () => update((s) => {
    if (s.slots[i].override) s.slots[i].override.value = value.value;
  }));

  $('slots').append(card);
  return { card, picker, debuff, kind, value, headline, kindLabel, status, badge, note };
});

// ---- Render ---------------------------------------------------------------

function showError(message) {
  $('error').textContent = message;
  $('error').hidden = false;
}

function render() {
  const { spec, jokerIndex } = buildSpec(state);
  let trace;
  try {
    trace = JSON.parse(calculator.score(JSON.stringify(spec)));
    $('error').hidden = true;
  } catch (error) {
    showError(error.payload ?? String(error));
    return;
  }

  handSelect.value = state.hand;
  $('played').textContent = spec.played.join(' ');
  const base = trace.steps[0].op.Add;
  $('base-chips').textContent = `${fmtNum(base.chips)} Chips`;
  $('base-mult').textContent = `${fmtNum(base.mult)} Mult`;

  const jokerNames = spec.jokers.map((j) => byIdent.get(j.ident)?.name ?? j.ident);
  const jokerSteps = (index) => trace.steps.filter((s) => s.phase === 'Joker' && s.source.Joker.index === index);
  lastSteps = jokerIndex.map((index) => (index === null ? [] : jokerSteps(index)));

  state.slots.forEach((slot, i) => {
    const nodes = slotNodes[i];
    const view = slotView(slot, byIdent.get(slot.ident), lastSteps[i]);
    nodes.picker.value = slot.ident ?? '';
    nodes.card.dataset.kind = view.kind;
    nodes.card.classList.toggle('empty', view.empty);
    nodes.card.classList.toggle('quiet', !view.empty && !view.fired);
    nodes.headline.textContent = view.headline;
    nodes.kindLabel.textContent = view.kindLabel;
    nodes.status.textContent = view.empty ? '' : slot.debuffed ? 'debuffed' : view.fired ? 'fired ✓' : 'did not fire';
    nodes.badge.hidden = !view.overridden;
    nodes.note.textContent = view.note;
    nodes.debuff.checked = Boolean(slot.debuffed);
    nodes.debuff.disabled = view.empty;
    nodes.kind.value = slot.override?.kind ?? '';
    nodes.kind.disabled = view.empty;
    nodes.value.disabled = !slot.override;
    if (document.activeElement !== nodes.value) nodes.value.value = slot.override ? String(view.overrideValue) : '';
  });

  const body = $('steps');
  body.replaceChildren(...rows(trace, jokerNames, state.showAll)
    .filter((row) => row.visible)
    .map((row) => el('tr', { class: row.fired ? '' : 'quiet' }, [
      el('td', { class: 'muted', text: String(row.n) }),
      el('td', {}, [
        el('span', { class: 'strong', text: row.source }),
        ...(row.detail ? [el('span', { class: 'muted small detail', text: row.detail })] : []),
      ]),
      el('td', { class: `strong ${row.kind}`, text: row.effect }),
      el('td', { class: 'chips', text: fmtNum(row.chips) }),
      el('td', { class: 'mult', text: fmtNum(row.mult) }),
      el('td', { class: 'strong', text: fmtNum(row.score) }),
    ])));

  const total = trace.total;
  for (const prefix of ['final', 'foot']) {
    $(`${prefix}-chips`).textContent = fmtNum(total.chips);
    $(`${prefix}-mult`).textContent = fmtNum(total.mult);
    $(`${prefix}-score`).textContent = fmtNum(handScore(total));
  }
  $('footnote').textContent = `Every number comes from the buffoonery engine ${calculator.version()}, through its WebAssembly component. `
    + 'The breakdown has one row per step the engine took; rows that changed nothing are hidden unless you ask. '
    + 'Known gap: every played card scores, so a kicker adds chips that Balatro would not.';
}

render();
