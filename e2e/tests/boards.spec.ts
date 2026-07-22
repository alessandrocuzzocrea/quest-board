import { test, expect } from '@playwright/test';

const TEST_EMAIL = `e2e-boards-${Date.now()}@test.com`;
const TEST_PW = 'test-password-123';

test.describe('Boards', () => {
  test.beforeAll(async ({ browser }) => {
    // Register a user once for all tests in this file
    const page = await browser.newPage();
    await page.goto('/');
    await page.getByText('Register').click();
    await page.fill('#reg-name', 'Board Tester');
    await page.fill('#reg-email', TEST_EMAIL);
    await page.fill('#reg-password', TEST_PW);
    await page.click('button:has-text("Create Account")');
    await page.waitForURL('/boards.html');
    await page.close();
  });

  test.beforeEach(async ({ page }) => {
    // Login before each test
    await page.goto('/');
    await page.fill('#login-email', TEST_EMAIL);
    await page.fill('#login-password', TEST_PW);
    await page.click('button:has-text("Sign In")');
    await page.waitForURL('/boards.html');
  });

  test('shows empty state with no boards', async ({ page }) => {
    await expect(page.locator('.empty-state')).toBeVisible();
  });

  test('create a board and see it in the list', async ({ page }) => {
    await page.click('button:has-text("New Board")');

    await page.fill('#new-board-name', 'My Test Board');
    await page.fill('#new-board-slug', 'my-test-board');
    await page.click('.modal button:has-text("Create")');

    // Wait for modal to close and board to appear
    await expect(page.locator('.board-card')).toHaveCount(1);
    await expect(page.locator('.board-card h3')).toHaveText('My Test Board');
  });

  test('navigate to board view', async ({ page }) => {
    // Create a board first
    await page.click('button:has-text("New Board")');
    await page.fill('#new-board-name', 'Board for Nav Test');
    await page.click('.modal button:has-text("Create")');
    await expect(page.locator('.board-card')).toHaveCount(1);

    // Click on the board card
    await page.locator('.board-card').click();
    await page.waitForURL(/\/board\.html\?id=/);

    // Should see the board title
    await expect(page.locator('#board-title')).toHaveText('Board for Nav Test');
  });

  test('create board with just a name (no slug)', async ({ page }) => {
    await page.click('button:has-text("New Board")');
    await page.fill('#new-board-name', 'Minimal Board');
    await page.click('.modal button:has-text("Create")');

    await expect(page.locator('.board-card h3')).toHaveText('Minimal Board');
  });

  test('search filters boards', async ({ page }) => {
    // Create two boards
    await page.click('button:has-text("New Board")');
    await page.fill('#new-board-name', 'Alpha Board');
    await page.click('.modal button:has-text("Create")');
    await expect(page.locator('.board-card')).toHaveCount(1);

    await page.click('button:has-text("New Board")');
    await page.fill('#new-board-name', 'Beta Board');
    await page.click('.modal button:has-text("Create")');
    await expect(page.locator('.board-card')).toHaveCount(2);

    // Search for "Alpha"
    await page.fill('#search-input', 'Alpha');
    await page.waitForTimeout(400); // debounce
    await expect(page.locator('.board-card')).toHaveCount(1);
    await expect(page.locator('.board-card h3')).toHaveText('Alpha Board');

    // Clear search
    await page.fill('#search-input', '');
    await page.waitForTimeout(400); // debounce
    await expect(page.locator('.board-card')).toHaveCount(2);
  });

  test('delete a board', async ({ page }) => {
    // Create a board to delete
    await page.click('button:has-text("New Board")');
    await page.fill('#new-board-name', 'Board to Delete');
    await page.click('.modal button:has-text("Create")');
    await expect(page.locator('.board-card')).toHaveCount(1);

    // Hover and click delete button
    const card = page.locator('.board-card');
    await card.hover();
    await card.locator('button[title="Delete"]').click();

    // Confirm deletion
    await expect(page.locator('#delete-modal')).toBeVisible();
    await page.click('#delete-confirm-btn');

    // Should be gone
    await expect(page.locator('.empty-state')).toBeVisible();
  });
});
