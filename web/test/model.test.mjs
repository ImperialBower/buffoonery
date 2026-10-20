// The calculator's model against the real component (EPIC-05 Story 6).
// `npm test` transpiles the component into src/pkg first.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { calculator } from '../src/pkg/buffoonery_component.js';
import {
  SAMPLE_HANDS,
  buildSpec,
  formatMPip,
  formatOp,
  handScore,
  initialState,
  opHeadline,
  overrideEnhancement,
  rows,
  slotView,
} from '../src/model.js';

const score = (spec) => JSON.parse(calculator.score(JSON.stringify(spec)));
const catalog = new Map(JSON.parse(calculator.catalog()).map((j) => [j.ident, j]));
const jokerSteps = (trace, part) => trace.steps.filter((s) => s.phase === 'Joker' && s.part === part);

test('every sample hand is the hand type it is listed under', () => {
  for (const hand of SAMPLE_HANDS) {
    const { spec } = buildSpec({ ...initialState(), hand: hand.type, slots: [] });
    assert.equal(score(spec).hand_type, hand.type, hand.played.join(' '));
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
  const state = initialState();
  state.slots = [{ ident: null }, { ident: 'JOKER' }, { ident: null }, { ident: 'DROLL_JOKER' }, { ident: null }];
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
    const state = { ...initialState(), hand: 'HighCard' };
    state.slots = [{ ident: 'JOLLY_JOKER', override: { kind, value }, debuffed: false }];
    const step = jokerSteps(score(buildSpec(state).spec), 'Effect')[0];
    assert.ok(step.fired, kind);
    assert.equal(formatOp(step.op), text);
  }
});

test('a debuffed joker shows as not firing, and scores nothing', () => {
  const state = initialState();
  state.slots = [{ ident: 'JOKER', override: null, debuffed: true }];
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

  const view = slotView(
    { ident: 'DROLL_JOKER', override: { kind: 'chips', value: 5 } },
    catalog.get('DROLL_JOKER'),
    steps,
  );
  assert.ok(view.overridden);
  assert.match(view.note, /^Overridden: Chips\(5\) \(printed: MultPlusOnFlush\(10\)\)/);
});

test('a Blank joker says why', () => {
  const state = initialState();
  state.slots = [{ ident: 'DNA', override: null, debuffed: false }];
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

test('text formats match the engine', () => {
  assert.equal(formatOp({ Add: { chips: 5, mult: 3 } }), '+5 Chips +3 Mult');
  assert.equal(formatOp({ Add: { chips: 0, mult: 0 } }), 'nothing');
  assert.equal(formatOp({ Seq: [{ AddMult: 2 }, { TimesMult: 2 }] }), '+2 Mult, ×2 Mult');
  assert.equal(opHeadline({ TimesMult: 1.5 }), '×1.5');
  assert.equal(formatMPip({ Lucky: [5, 15] }), 'Lucky(5, 15)');
  assert.equal(formatMPip({ RetriggerPlayedRanks: [1, ['2', '3', '4', '5']] }), 'RetriggerPlayedRanks(1, [2, 3, 4, 5])');
  assert.equal(handScore({ chips: 26, mult: 10.125 }), 263);
});
