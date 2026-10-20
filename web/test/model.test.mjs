// The calculator's model against the real component (EPIC-05 Stories 6–7).
// `npm test` transpiles the component into src/pkg first.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { calculator } from '../src/pkg/buffoonery_component.js';
import {
  EDITIONS,
  boardHash,
  HAND_TYPES,
  MAX_HELD,
  MAX_PLAYED,
  buildSpec,
  cards,
  formatMPip,
  formatOp,
  groupRows,
  handScore,
  handSummary,
  initialState,
  opHeadline,
  overrideEnhancement,
  parseSeed,
  readHash,
  rows,
  slotView,
  specToState,
} from '../src/model.js';

const score = (spec) => JSON.parse(calculator.score(JSON.stringify(spec)));
const catalog = new Map(JSON.parse(calculator.catalog()).map((j) => [j.ident, j]));
const jokerSteps = (trace, part) => trace.steps.filter((s) => s.phase === 'Joker' && s.part === part);
const board = (patch) => ({ ...initialState(), ...patch });

test('cards reads an index list into rank and suit', () => {
  assert.deepEqual(cards('TS 2h'), [{ rank: 'T', suit: 'S' }, { rank: '2', suit: 'H' }]);
  assert.deepEqual(cards(''), []);
});

test('the spec carries the played and held cards and every changed level', () => {
  const { spec } = buildSpec(board({
    played: cards('AS AH'),
    held: cards('KH 3C'),
    levels: { Pair: 3, Flush: 1 },
    slots: [],
  }));
  assert.deepEqual(spec, { played: ['AS', 'AH'], held: ['KH', '3C'], hand_levels: { Pair: 3 }, jokers: [] });
  assert.equal(MAX_PLAYED, 5);
  assert.equal(MAX_HELD, 8);
});

test('the engine finds the hand type and its level', () => {
  const trace = score(buildSpec(board({ levels: { Flush: 2 }, slots: [] })).spec);
  assert.deepEqual(handSummary(trace), { type: 'Flush', label: 'Flush', level: 2, chips: 50, mult: 6 });

  const pair = score(buildSpec(board({ played: cards('5S 5D'), slots: [] })).spec);
  assert.equal(handSummary(pair).label, 'Pair');
});

test('every hand type has a label and can take a level', () => {
  assert.equal(HAND_TYPES.length, 12);
  for (const { type } of HAND_TYPES) {
    assert.doesNotThrow(() => score({ played: ['AS'], hand_levels: { [type]: 2 } }), type);
  }
});

test('the opening board reads like Story 1 acceptance', () => {
  const trace = score(buildSpec(initialState()).spec);
  assert.deepEqual(
    jokerSteps(trace, 'Effect').map((s) => formatOp(s.op)),
    ['+4 Mult', '+10 Mult', '+80 Chips', '×3 Mult'],
  );
});

test('an empty slot is left out of the spec, and the others keep their index', () => {
  const state = board({ slots: [{ ident: null }, { ident: 'JOKER' }, { ident: null }, { ident: 'DROLL_JOKER' }, { ident: null }] });
  const { spec, jokerIndex } = buildSpec(state);
  assert.deepEqual(spec.jokers, [{ ident: 'JOKER' }, { ident: 'DROLL_JOKER' }]);
  assert.deepEqual(jokerIndex, [null, 0, null, 1, null]);
});

test('an override is an MPip the engine scores', () => {
  assert.deepEqual(overrideEnhancement({ kind: 'chips', value: 50 }), { enhancement: { Chips: 50 }, value: 50 });
  assert.deepEqual(overrideEnhancement({ kind: 'mult', value: '7' }), { enhancement: { MultPlus: 7 }, value: 7 });
  assert.deepEqual(overrideEnhancement({ kind: 'xmult', value: 3 }), { enhancement: { MultTimes: 3 }, value: 3 });
  assert.deepEqual(overrideEnhancement({ kind: 'xmult', value: 1.25 }), { enhancement: { MultTimes1Dot: 13 }, value: 1.3 });
  assert.deepEqual(overrideEnhancement({ kind: 'mult', value: -4 }), { enhancement: { MultPlus: 0 }, value: 0 });
  assert.equal(overrideEnhancement({ kind: null, value: 4 }), null);

  for (const [kind, value, text] of [['chips', 50, '+50 Chips'], ['mult', 7, '+7 Mult'], ['xmult', 1.5, '×1.5 Mult']]) {
    const state = board({ played: cards('AS'), slots: [{ ident: 'JOLLY_JOKER', override: { kind, value }, debuffed: false }] });
    const step = jokerSteps(score(buildSpec(state).spec), 'Effect')[0];
    assert.ok(step.fired, kind);
    assert.equal(formatOp(step.op), text);
  }
});

test('a debuffed joker shows as not firing, and scores nothing', () => {
  const state = board({ slots: [{ ident: 'JOKER', override: null, debuffed: true }] });
  const trace = score(buildSpec(state).spec);
  const view = slotView(state.slots[0], catalog.get('JOKER'), jokerSteps(trace, 'Effect'));
  assert.equal(view.kindLabel, 'debuffed');
  assert.equal(view.headline, '—');
  assert.deepEqual(rows(trace, ['Joker']).filter((r) => r.source.includes('slot')).map((r) => r.effect), ['debuffed', 'debuffed']);
  assert.deepEqual(trace.total, score(buildSpec({ ...state, slots: [] }).spec).total);
});

test('a joker card shows its printed effect, or its override and the printed one', () => {
  const trace = score(buildSpec(initialState()).spec);
  const steps = trace.steps.filter((s) => s.phase === 'Joker' && s.source.Joker.index === 1);
  const droll = slotView({ ident: 'DROLL_JOKER' }, catalog.get('DROLL_JOKER'), steps);
  assert.equal(droll.headline, '+10');
  assert.equal(droll.kindLabel, 'Mult');
  assert.equal(droll.note, 'MultPlusOnFlush(10)');

  const view = slotView({ ident: 'DROLL_JOKER', override: { kind: 'chips', value: 5 } }, catalog.get('DROLL_JOKER'), steps);
  assert.ok(view.overridden);
  assert.match(view.note, /^Overridden: Chips\(5\) \(printed: MultPlusOnFlush\(10\)\)/);
});

test('a Blank joker says why', () => {
  const state = board({ slots: [{ ident: 'DNA', override: null, debuffed: false }] });
  const trace = score(buildSpec(state).spec);
  const view = slotView(state.slots[0], catalog.get('DNA'), trace.steps.filter((s) => s.phase === 'Joker'));
  assert.equal(view.kindLabel, 'did not fire');
  assert.match(view.note, /^Blank · Not wired yet: /);
});

test('the table has one row per trace step', () => {
  const trace = score(buildSpec(initialState()).spec);
  const table = rows(trace, ['Joker', 'Droll Joker', 'Crafty Joker', 'Cavendish'], true);
  assert.equal(table.length, trace.steps.length);
  assert.ok(table.every((row) => row.visible));
  assert.equal(table[0].source, 'Flush (level 1)');
  assert.equal(table.at(-1).source, 'Cavendish · slot 4');
  assert.equal(table.at(-1).score, handScore(trace.total));

  const quiet = rows(trace, [], false).filter((row) => row.visible);
  assert.ok(quiet.length < table.length, 'no-op card parts are hidden by default');
  assert.ok(quiet.every((row) => row.fired || row.source.includes('slot')));
});

test('rows are grouped by the card or joker that made them', () => {
  // Hack retriggers a played 2: 2 passes × 3 parts, all in one group.
  const state = board({ played: cards('2S 2H'), held: cards('KH'), slots: [{ ident: 'HACK', override: null, debuffed: false }] });
  const trace = score(buildSpec(state).spec);
  const groups = groupRows(rows(trace, ['Hack'], true));

  assert.deepEqual(groups.map((g) => g.key), ['Base', 'Played 0', 'Played 1', 'Held 0', 'Joker 0']);
  assert.equal(groups[1].rows.length, 6);
  assert.equal(groups[1].label, '2♠ · played 1');
  assert.deepEqual(groups[1].rows.map((r) => r.detail), ['', 'special', 'edition', 'retrigger 1', 'special, retrigger 1', 'edition, retrigger 1']);
  assert.equal(groups[1].rows[1].effect, 'nothing', 'a card part that changed nothing');
  assert.equal(groups.at(-1).rows[0].effect, 'did not fire', 'Hack adds no score of its own');

  // A group whose rows are all hidden is left out.
  const quiet = groupRows(rows(trace, ['Hack'], false));
  assert.deepEqual(quiet.map((g) => g.key), ['Base', 'Played 0', 'Played 1', 'Joker 0']);
});

const golden = join(import.meta.dirname, '../../fixtures/golden');
const fixtures = readdirSync(golden).filter((name) => name.endsWith('.yaml')).sort();

test('every golden board survives the trip through page state', () => {
  assert.ok(fixtures.length >= 10);
  for (const name of fixtures) {
    const spec = JSON.parse(calculator.parse(readFileSync(join(golden, name), 'utf8')));
    const expected = JSON.parse(readFileSync(join(golden, name.replace(/\.yaml$/, '.trace.json')), 'utf8'));
    assert.deepEqual(score(buildSpec(specToState(spec)).spec), expected, name);
  }
});

test('a card keeps its enhancement and edition', () => {
  const state = specToState({ played: [{ card: 'AH', enhancement: { Chips: 30 }, edition: 'Foil' }, 'KS'] });
  assert.deepEqual(state.played, [{ rank: 'A', suit: 'H', enhancement: { Chips: 30 }, edition: 'Foil' }, { rank: 'K', suit: 'S' }]);
  assert.deepEqual(buildSpec(state).spec.played, [{ card: 'AH', enhancement: { Chips: 30 }, edition: 'Foil' }, 'KS']);
});

test('a joker override the editor knows becomes an override; any other stays as it is', () => {
  const state = specToState({ played: ['AS'], jokers: [
    { ident: 'JOKER', enhancement: { MultPlus: 7 } },
    { ident: 'JOLLY_JOKER', enhancement: { MultPlusOnPair: 9 }, edition: 'Negative', debuffed: true },
  ] });
  assert.deepEqual(state.slots[0], { ident: 'JOKER', override: { kind: 'mult', value: 7 }, debuffed: false });
  assert.deepEqual(state.slots[1], { ident: 'JOLLY_JOKER', override: null, enhancement: { MultPlusOnPair: 9 }, edition: 'Negative', debuffed: true });
  assert.equal(state.slots.length, 5);
  assert.deepEqual(buildSpec(state).spec.jokers[1], { ident: 'JOLLY_JOKER', enhancement: { MultPlusOnPair: 9 }, edition: 'Negative', debuffed: true });
});

test('every edition the picker offers scores as the engine says', () => {
  assert.deepEqual(EDITIONS.map((e) => e.edition), [null, 'Foil', 'Holographic', 'Polychrome', 'Negative']);
  const expected = { Foil: { AddChips: 50 }, Holographic: { AddMult: 10 }, Polychrome: { TimesMult: 1.5 } };
  for (const { edition } of EDITIONS) {
    const state = initialState();
    state.slots[0].edition = edition;
    const { spec } = buildSpec(state);
    if (edition) assert.equal(spec.jokers[0].edition, edition);
    else assert.equal('edition' in spec.jokers[0], false);
    const step = jokerSteps(score(spec), 'Edition').find((s) => s.source.Joker.index === 0);
    if (expected[edition]) {
      assert.equal(step.fired, true, edition);
      assert.deepEqual(step.op, expected[edition]);
    } else {
      assert.equal(step?.fired ?? false, false, String(edition));
    }
  }
});

test('an edition of None leaves the spec as it was', () => {
  const state = specToState({ played: ['AS'], jokers: [{ ident: 'JOKER', edition: 'None' }] });
  assert.deepEqual(buildSpec(state).spec.jokers, [{ ident: 'JOKER' }]);
});

test('the board fields the page has no control for are kept', () => {
  const spec = { played: ['AS'], money: 20, discards_left: 0, hands_left: 2, blind: 'Small' };
  assert.deepEqual(buildSpec(specToState(spec)).spec, { played: ['AS'], jokers: [], money: 20, discards_left: 0, hands_left: 2, blind: 'Small' });
});

test('a board the page cannot show is an error', () => {
  const six = Array.from({ length: 6 }, () => ({ ident: 'JOKER' }));
  assert.throws(() => specToState({ played: ['AS'], jokers: six }), /at most 5 jokers/);
  assert.throws(() => specToState({ played: [] }), /at least 1 played card/);
  assert.throws(() => specToState({ played: ['AS'], held: Array(9).fill('2C') }), /at most 8 held/);
});

test('a seed is a whole number that fits a u64, or nothing', () => {
  assert.equal(parseSeed(''), null);
  assert.equal(parseSeed('  '), null);
  assert.equal(parseSeed('7'), 7n);
  assert.equal(parseSeed(' 007 '), 7n);
  assert.equal(parseSeed('18446744073709551615'), 2n ** 64n - 1n);
  for (const bad of ['18446744073709551616', '-1', '1.5', 'abc', '1e3']) {
    assert.throws(() => parseSeed(bad), /whole number from 0 to 18446744073709551615/, bad);
  }
});

test('the link holds the board, and the seed when there is one', () => {
  const spec = { played: ['AS'], jokers: [] };
  assert.equal(boardHash(spec, null), `#board=${encodeURIComponent(JSON.stringify(spec))}`);
  assert.equal(boardHash(spec, 7n), `#seed=7&board=${encodeURIComponent(JSON.stringify(spec))}`);
  assert.deepEqual(readHash(boardHash(spec, 7n)), { board: JSON.stringify(spec), seed: 7n });
  assert.deepEqual(readHash(boardHash(spec, null)), { board: JSON.stringify(spec), seed: null });
  assert.deepEqual(readHash(`#board=${encodeURIComponent('a: b&c')}`), { board: 'a: b&c', seed: null });
  assert.equal(readHash('#other'), null);
  assert.equal(readHash(''), null);
});

test('a seeded score marks its rolled rows', () => {
  const yaml = readFileSync(join(golden, '11_inert_jokers.yaml'), 'utf8');
  const marked = (trace) => rows(trace, [], true).filter((row) => row.rolled).map((row) => row.detail);
  assert.deepEqual(marked(JSON.parse(calculator.score(yaml))), []);
  assert.deepEqual(marked(JSON.parse(calculator.scoreSeeded(yaml, 7n))), ['special, rolled', 'rolled']);
});

test('text formats match the engine', () => {
  assert.equal(formatOp({ Add: { chips: 5, mult: 3 } }), '+5 Chips +3 Mult');
  assert.equal(formatOp({ Add: { chips: 0, mult: 0 } }), 'nothing');
  assert.equal(formatOp({ Seq: [{ AddMult: 2 }, { TimesMult: 2 }] }), '+2 Mult, ×2 Mult');
  assert.equal(opHeadline({ TimesMult: 1.5 }), '×1.5');
  assert.equal(formatMPip({ Lucky: [5, 15] }), 'Lucky(5, 15)');
  assert.equal(formatMPip({ RetriggerPlayedRanks: [1, ['2', '3', '4', '5']] }), 'RetriggerPlayedRanks(1, [2, 3, 4, 5])');
  assert.equal(handScore({ chips: 26, mult: 10.125 }), 263);
});
