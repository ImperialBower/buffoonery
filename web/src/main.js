// The Joker Calculator page (EPIC-05 Stories 6–7). It draws what model.js
// derives from the engine's trace; it scores nothing itself.
import './style.css';
import { calculator } from './pkg/buffoonery_component.js';
import {
  HAND_TYPES,
  MAX_HELD,
  MAX_PLAYED,
  OVERRIDE_KINDS,
  RANKS,
  SLOT_COUNT,
  SUITS,
  buildSpec,
  fmtNum,
  formatMPip,
  groupRows,
  handScore,
  handSummary,
  initialState,
  opKind,
  rows,
  slotView,
  specToState,
} from './model.js';

const catalog = JSON.parse(calculator.catalog());
const byIdent = new Map(catalog.map((joker) => [joker.ident, joker]));
const pickerOrder = [...catalog].sort((a, b) => a.name.localeCompare(b.name));

const $ = (id) => document.getElementById(id);

// A board can come from the link: `#board=` and a BoardSpec as JSON or YAML,
// URL-encoded. The page keeps the link up to date, so the address bar always
// holds the board on screen.
let loadError = null;

function loadFromHash() {
  const match = /^#board=(.*)$/s.exec(window.location.hash);
  if (!match) return null;
  try {
    return specToState(JSON.parse(calculator.parse(decodeURIComponent(match[1]))));
  } catch (error) {
    loadError = `The board in the link could not be opened: ${error.payload ?? error.message}`;
    return null;
  }
}

let state = loadFromHash() ?? initialState();
let lastSteps = [];
let lastHand = null;

window.addEventListener('hashchange', () => {
  const loaded = loadFromHash();
  if (loaded) state = loaded;
  render();
});

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
  loadError = null;
  change(state);
  render();
}

// ---- Cards: rebuilt only when a card is added or removed --------------------

const PILES = {
  played: { list: $('played'), count: $('played-count'), max: MAX_PLAYED, min: 1, name: 'played' },
  held: { list: $('held'), count: $('held-count'), max: MAX_HELD, min: 0, name: 'held' },
};

function cardEditor(pile, i) {
  const n = i + 1;
  const rank = el('select', { 'aria-label': `Rank, ${pile.name} card ${n}` },
    RANKS.map((r) => el('option', { value: r.rank, text: r.label })));
  const suit = el('select', { 'aria-label': `Suit, ${pile.name} card ${n}` },
    SUITS.map((s) => el('option', { value: s.suit, text: s.label, title: s.name })));
  const remove = el('button', { type: 'button', class: 'remove', 'aria-label': `Remove ${pile.name} card ${n}`, text: '×' });
  // Enhancement and edition have no editor yet (Milestone D); a board from a
  // link can carry them, so they show as a label.
  const tag = el('span', { class: 'tag' });
  const key = pile.name;
  rank.addEventListener('change', () => update((s) => { s[key][i].rank = rank.value; }));
  suit.addEventListener('change', () => update((s) => { s[key][i].suit = suit.value; }));
  remove.addEventListener('click', () => update((s) => { s[key].splice(i, 1); }));
  return Object.assign(el('div', { class: 'card-edit' }, [rank, suit, remove, tag]), { rank, suit, remove, tag });
}

function renderPile(key) {
  const pile = PILES[key];
  const pileCards = state[key];
  if (pile.editors?.length !== pileCards.length) {
    pile.editors = pileCards.map((_, i) => cardEditor(pile, i));
    const add = el('button', { type: 'button', class: 'add', text: '+ Add card' });
    add.addEventListener('click', () => update((s) => { s[key].push({ rank: 'A', suit: 'S' }); }));
    pile.add = add;
    pile.list.replaceChildren(...pile.editors, add);
  }
  pileCards.forEach((card, i) => {
    const editor = pile.editors[i];
    editor.rank.value = card.rank;
    editor.suit.value = card.suit;
    editor.dataset.suit = card.suit;
    const tags = [card.enhancement && formatMPip(card.enhancement), card.edition !== 'None' && card.edition].filter(Boolean);
    editor.tag.textContent = tags.join(' · ');
    editor.tag.hidden = tags.length === 0;
    editor.remove.disabled = pileCards.length <= pile.min;
  });
  pile.add.disabled = pileCards.length >= pile.max;
  pile.count.textContent = `${pileCards.length} of ${pile.max}`;
}

// ---- Hand level -----------------------------------------------------------

const levelHand = $('level-hand');
const levelInput = $('level');
for (const hand of HAND_TYPES) levelHand.append(el('option', { value: hand.type, text: hand.label }));
levelHand.addEventListener('change', () => update((s) => { s.levelHand = levelHand.value; }));
levelInput.addEventListener('input', () => {
  const level = Number.parseInt(levelInput.value, 10);
  if (level >= 1) update((s) => { s.levels[levelHand.value] = level; });
});

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
    // A link's override with no editor kind is dropped once you pick one.
    delete s.slots[i].enhancement;
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

function renderSteps(trace, jokerNames) {
  // One <tbody> per group, so each card's rows sit together.
  const groups = groupRows(rows(trace, jokerNames, state.showAll));
  const table = $('breakdown');
  table.querySelectorAll('tbody.group').forEach((body) => body.remove());
  const foot = table.querySelector('tfoot');
  for (const group of groups) {
    const body = el('tbody', { class: 'group' }, group.rows.map((row, i) => el('tr', { class: row.fired ? '' : 'quiet' }, [
      el('td', { class: 'muted', text: String(row.n) }),
      el('td', {}, [
        ...(i === 0 ? [el('span', { class: 'strong', text: group.label })] : []),
        ...(row.detail ? [el('span', { class: 'muted small detail', text: row.detail })] : []),
      ]),
      el('td', { class: `strong ${row.kind}`, text: row.effect }),
      el('td', { class: 'chips', text: fmtNum(row.chips) }),
      el('td', { class: 'mult', text: fmtNum(row.mult) }),
      el('td', { class: 'strong', text: fmtNum(row.score) }),
    ])));
    table.insertBefore(body, foot);
  }
}

function render() {
  renderPile('played');
  renderPile('held');

  const { spec, jokerIndex } = buildSpec(state);
  let trace;
  try {
    trace = JSON.parse(calculator.score(JSON.stringify(spec)));
    if (loadError) showError(loadError);
    else $('error').hidden = true;
    window.history.replaceState(null, '', `#board=${encodeURIComponent(JSON.stringify(spec))}`);
  } catch (error) {
    showError(error.payload ?? String(error));
    return;
  }

  lastHand = handSummary(trace);
  $('hand-name').textContent = lastHand.label;
  $('hand-level').textContent = `level ${lastHand.level}`;
  $('base-chips').textContent = `${fmtNum(lastHand.chips)} Chips`;
  $('base-mult').textContent = `${fmtNum(lastHand.mult)} Mult`;
  const editing = state.levelHand ?? lastHand.type;
  levelHand.value = editing;
  if (document.activeElement !== levelInput) levelInput.value = String(state.levels[editing] ?? 1);

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

  renderSteps(trace, jokerNames);

  const total = trace.total;
  for (const prefix of ['final', 'foot']) {
    $(`${prefix}-chips`).textContent = fmtNum(total.chips);
    $(`${prefix}-mult`).textContent = fmtNum(total.mult);
    $(`${prefix}-score`).textContent = fmtNum(handScore(total));
  }
  $('footnote').textContent = `Every number comes from the buffoonery engine ${calculator.version()}, through its WebAssembly component. `
    + 'The breakdown groups the steps by the card or joker that made them; steps that changed nothing are hidden unless you ask. '
    + 'Known gaps: every played card scores, so a kicker adds chips that Balatro would not; A-2-3-4-5 is not a straight.';
}

render();
