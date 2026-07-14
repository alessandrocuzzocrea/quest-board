import { test, expect } from '@playwright/test';

const TEST_EMAIL = `e2e-settings-${Date.now()}@test.com`;
const TEST_PW = 'test-password-123';

test.describe('Settings', () => {
  test.beforeAll(async ({ browser }) => {
    const page = await browser.newPage();
    await page.goto('/');
    await page.getByText('Register').click();
    await page.fill('#reg-name', 'Settings Tester');
    await page.fill('#reg-email', TEST_EMAIL);
    await page.fill('#reg-password', TEST_PW);
    await page.click('button:has-text("Create Account")');
    await page.waitForURL('/boards.html');
    await page.close();
  });

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.fill('#login-email', TEST_EMAIL);
    await page.fill('#login-password', TEST_PW);
    await page.click('button:has-text("Sign In")');
    await page.waitForURL('/boards.html');
    await page.goto('/settings.html');
    await page.waitForSelector('#settings-name');
  });

  test('shows user profile data', async ({ page }) => {
    await expect(page.locator('#settings-name')).toHaveValue('Settings Tester');
    await expect(page.locator('#settings-email')).toHaveValue(TEST_EMAIL);
  });

  test('update user name', async ({ page }) => {
    await page.fill('#settings-name', 'Updated Name');
    await page.click('button:has-text("Save Changes")');
    await expect(page.locator('.alert-success')).toBeVisible();

    // Verify name updated in nav bar
    await expect(page.locator('#user-name')).toHaveText('Updated Name');
  });

  test('change password then login with new password', async ({ page, browser }) => {
    const NEW_PW = 'new-password-456';

    // Change password
    await page.fill('#pw-current', TEST_PW);
    await page.fill('#pw-new', NEW_PW);
    await page.click('button:has-text("Update Password")');
    await expect(page.locator('.alert-success')).toBeVisible();

    // Logout
    await page.getByText('Logout').click();
    await page.waitForURL('/index.html');

    // Login with new password
    await page.fill('#login-email', TEST_EMAIL);
    await page.fill('#login-password', NEW_PW);
    await page.click('button:has-text("Sign In")');
    await page.waitForURL('/boards.html');
    await expect(page.locator('#user-name')).toBeVisible();
  });

  test('create and revoke API key', async ({ page }) => {
    // Create API key
    await page.fill('#new-key-name', 'ci-token');
    await page.click('button:has-text("Generate")');
    await expect(page.locator('.alert-success')).toBeVisible();

    // Should see the key in the list
    await expect(page.locator('.api-key-row')).toHaveCount(1);
    await expect(page.locator('.api-key-row strong')).toHaveText('ci-token');

    // Revoke the key
    page.on('dialog', (dialog) => dialog.accept());
    await page.click('.api-key-row button:has-text("Revoke")');
    await page.waitForTimeout(300);
    await expect(page.locator('.api-key-row')).toHaveCount(0);
  });
});
