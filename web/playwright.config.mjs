import { defineConfig } from '@playwright/test';

// The browser parity test (EPIC-05 Story 8) runs against the built page
// (`npm run e2e` builds it first), served by `vite preview`.
export default defineConfig({
  testDir: 'e2e',
  forbidOnly: Boolean(process.env.CI),
  reporter: process.env.CI ? 'github' : 'list',
  use: { baseURL: 'http://localhost:4174', locale: 'en-US' },
  projects: [{ name: 'chromium', use: { browserName: 'chromium' } }],
  webServer: {
    command: 'npx vite preview --port 4174 --strictPort',
    url: 'http://localhost:4174',
    reuseExistingServer: false,
  },
});
