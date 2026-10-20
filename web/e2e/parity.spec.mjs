// The browser parity test (EPIC-05 Story 8): every golden fixture, opened in
// the real page from a link, shows exactly the fixture's trace, one table row
// per step. The expected rows come from the fixture's JSON, never from a fresh
// score, so the page must reach the engine's numbers on its own.
import { test, expect } from '@playwright/test';
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fmtNum, handScore, rows } from '../src/model.js';

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
});
