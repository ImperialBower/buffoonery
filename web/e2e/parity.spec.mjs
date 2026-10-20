// The browser parity test (EPIC-05 Story 8): every golden fixture, opened in
// the real page from a link, shows exactly the fixture's trace, one table row
// per step. The expected rows come from the fixture's JSON, never from a fresh
// score, so the page must reach the engine's numbers on its own. Tests of an
// editor score their expected board with the component, as model.test.mjs does.
import { test, expect } from '@playwright/test';
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { calculator } from '../src/pkg/buffoonery_component.js';
import { buildSpec, fmtNum, handScore, initialState, rows } from '../src/model.js';

const golden = join(import.meta.dirname, '../../fixtures/golden');
const fixtures = readdirSync(golden).filter((name) => name.endsWith('.yaml')).sort();

/** The cells the page must show: #, effect, chips, mult, running score. */
function expectedCells(trace) {
  return rows(trace, [], true).map((row) => [String(row.n), row.effect, fmtNum(row.chips), fmtNum(row.mult), fmtNum(row.score)]);
}

async function shownCells(page) {
  return page.locator('#breakdown tbody.group tr').evaluateAll((trs) => trs.map((tr) => {
    const cells = [...tr.cells].map((cell) => cell.innerText.trim());
    return [cells[0], cells[2], cells[3], cells[4], cells[5]];
  }));
}

test('the page loads the component and scores its opening board', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('#final-score')).toHaveText('8,316');
  await expect(page.locator('#error')).toBeHidden();
  expect(page.url()).toContain('#board=');
});

for (const name of fixtures) {
  test(`parity: ${name}`, async ({ page }) => {
    const yaml = readFileSync(join(golden, name), 'utf8');
    const trace = JSON.parse(readFileSync(join(golden, name.replace(/\.yaml$/, '.trace.json')), 'utf8'));

    await page.goto(`/#board=${encodeURIComponent(yaml)}`);
    await page.check('#show-all');
    await expect(page.locator('#error')).toBeHidden();
    await expect(page.locator('#breakdown tbody.group tr')).toHaveCount(trace.steps.length);

    expect(await shownCells(page)).toEqual(expectedCells(trace));
    await expect(page.locator('#foot-score')).toHaveText(fmtNum(handScore(trace.total)));
  });
}

test('a link that is not a board says so, and the page opens its own board', async ({ page }) => {
  await page.goto(`/#board=${encodeURIComponent('colour: red')}`);
  await expect(page.locator('#error')).toContainText('The board in the link could not be opened: parse: ');
  await expect(page.locator('#final-score')).toHaveText('8,316');

  // A fresh load, not a hash change: on a hash change the page puts its own
  // board back in the address bar, so a reload would not see the bad link.
  await page.goto('about:blank');
  await page.goto(`/#board=${encodeURIComponent('played: [AS, ZZ]')}`);
  await expect(page.locator('#error')).toHaveText('The board in the link could not be opened: played[1]: unknown card "ZZ"');
  await expect(page.locator('#final-score')).toHaveText('8,316');
});

test('a pasted board opens like a link does', async ({ page }) => {
  const name = '06_editions.yaml';
  const yaml = readFileSync(join(golden, name), 'utf8');
  const trace = JSON.parse(readFileSync(join(golden, name.replace(/\.yaml$/, '.trace.json')), 'utf8'));

  await page.goto('/');
  await page.click('summary:has-text("Paste a board")');
  await page.fill('#paste-text', yaml);
  await page.click('#paste-open');
  await page.check('#show-all');
  await expect(page.locator('#paste-status')).toHaveText('Opened.');
  expect(await shownCells(page)).toEqual(expectedCells(trace));

  await page.fill('#paste-text', 'played: [AS]\njokers: [{ident: NOPE}]');
  await page.click('#paste-open');
  await expect(page.locator('#paste-status')).toHaveText('Could not open it: jokers[0]: unknown joker ident "NOPE"');
  expect(await shownCells(page)).toEqual(expectedCells(trace));
});

test('copy board and copy link put the board on the clipboard', async ({ page, context }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  const yaml = readFileSync(join(golden, '13_board_readers.yaml'), 'utf8');
  await page.goto(`/#board=${encodeURIComponent(yaml)}`);

  await page.click('#copy-board');
  await expect(page.locator('#copy-status')).toHaveText('Board copied.');
  const board = JSON.parse(await page.evaluate(() => navigator.clipboard.readText()));
  expect(board).toMatchObject({ money: 20, discards_left: 2, played: ['AS', 'AD', 'AH'] });

  await page.click('#copy-link');
  await expect(page.locator('#copy-status')).toHaveText('Link copied.');
  const link = await page.evaluate(() => navigator.clipboard.readText());
  expect(link).toBe(page.url());
  expect(JSON.parse(decodeURIComponent(new URL(link).hash.slice('#board='.length)))).toEqual(board);
});

test('the edition picker puts an edition on a joker, and the engine scores it', async ({ page }) => {
  const state = initialState();
  state.slots[0].edition = 'Polychrome';
  const trace = JSON.parse(calculator.score(JSON.stringify(buildSpec(state).spec)));

  await page.goto('/');
  await page.selectOption('[aria-label="Edition, slot 1"]', 'Polychrome');
  await page.check('#show-all');
  await expect(page.locator('#final-score')).toHaveText(fmtNum(handScore(trace.total)));
  expect(await shownCells(page)).toEqual(expectedCells(trace));
  await expect(page.locator('.slot').first().locator('.note')).toContainText('Edition: ×1.5 Mult');
  expect(JSON.parse(decodeURIComponent(new URL(page.url()).hash.slice('#board='.length))).jokers[0])
    .toEqual({ ident: 'JOKER', edition: 'Polychrome' });

  await page.selectOption('[aria-label="Edition, slot 1"]', '');
  await expect(page.locator('#final-score')).toHaveText('8,316');
  await expect(page.locator('[aria-label="Edition, slot 5"]')).toBeDisabled();
});
