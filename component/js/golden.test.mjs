// The golden fixtures against the built WIT component, through jco: the same
// ES module the calculator loads (EPIC-05 Story 4). `npm test` transpiles
// target/wasm32-wasip2/component/buffoonery_component.wasm into pkg/ first.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { calculator } from './pkg/buffoonery_component.js';

const golden = join(import.meta.dirname, '../../fixtures/golden');
const specs = readdirSync(golden).filter((name) => name.endsWith('.yaml')).sort();

test('the fixture set is not empty', () => {
  assert.ok(specs.length >= 10, `found ${specs.length} fixtures`);
});

for (const name of specs) {
  test(`score ${name}`, () => {
    const yaml = readFileSync(join(golden, name), 'utf8');
    const expected = readFileSync(join(golden, name.replace(/\.yaml$/, '.trace.json')), 'utf8');
    assert.deepEqual(JSON.parse(calculator.score(yaml)), JSON.parse(expected));
  });
}

test('a JSON spec scores like its YAML', () => {
  const trace = JSON.parse(calculator.score('{"played": ["AS", "AH"], "jokers": [{"ident": "JOLLY_JOKER"}]}'));
  assert.equal(trace.hand_type, 'Pair');
  assert.deepEqual(trace.total, { chips: 32, mult: 10 });
});

test('parse turns a YAML board into JSON that scores the same', () => {
  for (const name of specs) {
    const yaml = readFileSync(join(golden, name), 'utf8');
    const json = calculator.parse(yaml);
    assert.deepEqual(JSON.parse(calculator.score(json)), JSON.parse(calculator.score(yaml)), name);
  }
});

test('a bad spec throws its error message', () => {
  assert.throws(
    () => calculator.score('jokers: [{ident: NOPE}]'),
    (error) => error.payload === 'jokers[0]: unknown joker ident "NOPE"',
  );
});

test('score-with rolls the chances from a seed, and a seed repeats', () => {
  const yaml = readFileSync(join(golden, '11_inert_jokers.yaml'), 'utf8');
  const rolled = (trace) => JSON.parse(trace).steps.filter((step) => step.rolled).length;
  // WIT u64 is a BigInt in JavaScript; an absent option is `undefined`.
  const seeded = (seed) => ({ seed, effects: [] });
  assert.equal(rolled(calculator.scoreWith(yaml, seeded(undefined))), 0);
  assert.equal(calculator.scoreWith(yaml, seeded(undefined)), calculator.score(yaml));
  assert.equal(rolled(calculator.scoreWith(yaml, seeded(7n))), 2);
  assert.equal(calculator.scoreWith(yaml, seeded(7n)), calculator.scoreWith(yaml, seeded(7n)));
  assert.ok(calculator.scoreWith(yaml, seeded(2n ** 64n - 1n)));
  assert.throws(
    () => calculator.scoreWith('played: [AS, ZZ]', seeded(1n)),
    (error) => error.payload === 'played[1]: unknown card "ZZ"',
  );
});

test('score-with scores a custom effect as its op', () => {
  const board = 'played: [AS]\njokers: [{ident: JOKER, enhancement: !Custom 3}]';
  const op = { Seq: [{ AddChips: 20 }, { TimesMult: 2 }] };
  const trace = JSON.parse(calculator.scoreWith(board, { effects: [{ id: 3, op: JSON.stringify(op) }] }));
  assert.deepEqual(trace.total, { chips: 36, mult: 2 });
  assert.throws(
    () => calculator.scoreWith(board, { effects: [{ id: 3, op: '{"AddChips": -1}' }] }),
    (error) => error.payload.startsWith('effects[0].op: '),
  );
});

test('catalog lists every joker', () => {
  const catalog = JSON.parse(calculator.catalog());
  assert.equal(catalog.length, 112);
  assert.equal(catalog[0].ident, 'JOKER');
});

test('version is a semver string', () => {
  assert.match(calculator.version(), /^\d+\.\d+\.\d+/);
});

// The size budget in component/README.md: 500 KiB after `jco opt -Oz`.
test('the optimized component fits the size budget', () => {
  const wasm = join(import.meta.dirname, '../../target/wasm32-wasip2/component/buffoonery_component.opt.wasm');
  const kib = statSync(wasm).size / 1024;
  assert.ok(kib <= 500, `${kib.toFixed(0)} KiB is over the 500 KiB budget`);
});
