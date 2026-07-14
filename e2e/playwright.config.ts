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
    command: '../target/release/quest-board',
    timeout: 60000,
    reuseExistingServer: !process.env.CI,
    url: 'http://localhost:3001/api/v1/health',
    env: {
      DATABASE_URL: process.env.DATABASE_URL || 'postgres://postgres:quest@localhost:5432/quest_test',
      APP_SECRET: process.env.APP_SECRET || 'test-secret-for-e2e',
    },
  },
});
