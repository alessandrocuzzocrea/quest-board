import { test, expect } from '@playwright/test';

const TEST_EMAIL = `e2e-${Date.now()}@test.com`;
const TEST_PW = 'test-password-123';
const TEST_NAME = 'E2E User';

test.describe('Auth', () => {
  test('register a new user and redirect to boards', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('h1')).toHaveText('quest-board');

    // Switch to register tab
    await page.getByText('Register').click();

    // Fill registration form
    await page.fill('#reg-name', TEST_NAME);
    await page.fill('#reg-email', TEST_EMAIL);
    await page.fill('#reg-password', TEST_PW);
    await page.click('button:has-text("Create Account")');

    // Should redirect to boards page
    await page.waitForURL('/boards.html');
    await expect(page.locator('h1')).toHaveText('Boards');
  });

  test('login with existing credentials', async ({ page }) => {
    await page.goto('/');

    await page.fill('#login-email', TEST_EMAIL);
    await page.fill('#login-password', TEST_PW);
    await page.click('button:has-text("Sign In")');

    await page.waitForURL('/boards.html');
    await expect(page.locator('#user-name')).toHaveText(TEST_NAME);
  });

  test('show error on invalid login', async ({ page }) => {
    await page.goto('/');

    await page.fill('#login-email', 'wrong@test.com');
    await page.fill('#login-password', 'wrongpw');
    await page.click('button:has-text("Sign In")');

    await expect(page.locator('.alert-error')).toBeVisible();
  });

  test('logout clears session', async ({ page }) => {
    // Login first
    await page.goto('/');
    await page.fill('#login-email', TEST_EMAIL);
    await page.fill('#login-password', TEST_PW);
    await page.click('button:has-text("Sign In")');
    await page.waitForURL('/boards.html');

    // Logout
    await page.getByText('Logout').click();
    await page.waitForURL('/index.html');
    await expect(page.locator('h1')).toHaveText('quest-board');

    // Try accessing boards — should redirect to login
    await page.goto('/boards.html');
    await page.waitForURL('/index.html');
  });
});
