import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  timeout: 30000,
  fullyParallel: false,
  retries: 1,
  use: {
    baseURL: 'http://localhost:3001',
    trace: 'on-first-retry',
  },
  webServer: {
    command: 'DATABASE_URL=postgres://postgres:quest@localhost:5432/quest_test APP_SECRET=test-secret-for-e2e ../target/release/quest-board',
    port: 3001,
    timeout: 30000,
    reuseExistingServer: !process.env.CI,
    cwd: __dirname,
  },
});
