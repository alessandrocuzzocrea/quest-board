import { test, expect } from '@playwright/test';

const TEST_EMAIL = `e2e-kanban-${Date.now()}@test.com`;
const TEST_PW = 'test-password-123';

test.describe('Kanban Board', () => {
  let boardId = '';

  test.beforeAll(async ({ browser }) => {
    const page = await browser.newPage();
    // Register
    await page.goto('/');
    await page.getByText('Register').click();
    await page.fill('#reg-name', 'Kanban Tester');
    await page.fill('#reg-email', TEST_EMAIL);
    await page.fill('#reg-password', TEST_PW);
    await page.click('button:has-text("Create Account")');
    await page.waitForURL('/boards.html');

    // Create a board
    await page.click('button:has-text("New Board")');
    await page.fill('#new-board-name', 'Kanban Test Board');
    await page.click('.modal button:has-text("Create")');
    await expect(page.locator('.board-card')).toHaveCount(1);

    // Navigate to board
    await page.locator('.board-card').click();
    await page.waitForURL(/\/board\.html\?id=/);

    // Grab board ID from URL
    const url = page.url();
    boardId = new URL(url).searchParams.get('id')!;
    await page.close();
  });

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.fill('#login-email', TEST_EMAIL);
    await page.fill('#login-password', TEST_PW);
    await page.click('button:has-text("Sign In")');
    await page.waitForURL('/boards.html');
    await page.goto(`/board.html?id=${boardId}`);
    await page.waitForSelector('#board-title');
  });

  test('shows add list prompt with no columns', async ({ page }) => {
    // A new board has no lists, should show "Add List" option
    await expect(page.getByText('+ Add List')).toBeVisible();
  });

  test('add a list (column)', async ({ page }) => {
    await page.getByText('+ Add List').click();
    await page.fill('#new-list-name', 'To Do');
    await page.click('.modal button:has-text("Add")');

    // Should see the column
    await expect(page.locator('.column-header')).toHaveText(/To Do/);
  });

  test('add a second list', async ({ page }) => {
    // First list
    await page.getByText('+ Add List').click();
    await page.fill('#new-list-name', 'To Do');
    await page.click('.modal button:has-text("Add")');
    await expect(page.locator('.column-header')).toHaveCount(1);

    // Second list
    await page.getByText('+ Add List').click();
    await page.fill('#new-list-name', 'Done');
    await page.click('.modal button:has-text("Add")');
    await expect(page.locator('.column-header')).toHaveCount(2);
  });

  test('add a card to a list', async ({ page }) => {
    // Add a list first
    await page.getByText('+ Add List').click();
    await page.fill('#new-list-name', 'Backlog');
    await page.click('.modal button:has-text("Add")');

    // Add a card
    await page.locator('.column-footer button:has-text("Add Card")').click();
    await page.fill('#new-card-title', 'First task');
    await page.click('.modal button:has-text("Add")');

    // Should see the card
    await expect(page.locator('.kanban-card h4')).toHaveText('First task');
    await expect(page.locator('.column-header')).toContainText('(1)');
  });

  test('open card detail panel', async ({ page }) => {
    // Add list + card
    await page.getByText('+ Add List').click();
    await page.fill('#new-list-name', 'Sprint');
    await page.click('.modal button:has-text("Add")');

    await page.locator('.column-footer button:has-text("Add Card")').click();
    await page.fill('#new-card-title', 'Detail test card');
    await page.fill('#new-card-desc', 'This is a test description');
    await page.click('.modal button:has-text("Add")');

    // Open the card
    await page.locator('.kanban-card').click();
    await expect(page.locator('#card-panel')).toHaveClass(/open/);

    // Check panel content
    await expect(page.locator('#panel-title')).toHaveText('Detail test card');
    await expect(page.locator('#panel-body')).toContainText('This is a test description');
  });

  test('add a comment to a card', async ({ page }) => {
    // Add list + card
    await page.getByText('+ Add List').click();
    await page.fill('#new-list-name', 'Review');
    await page.click('.modal button:has-text("Add")');

    await page.locator('.column-footer button:has-text("Add Card")').click();
    await page.fill('#new-card-title', 'Comment test');
    await page.click('.modal button:has-text("Add")');

    // Open card
    await page.locator('.kanban-card').click();
    await expect(page.locator('#card-panel')).toHaveClass(/open/);

    // Write comment
    await page.fill('#comment-text', 'This is a great card!');
    await page.click('button:has-text("Comment")');

    // Should see the comment
    await expect(page.locator('.comment .text')).toHaveText('This is a great card!');
  });

  test('delete a card from panel', async ({ page }) => {
    // Add list + card
    await page.getByText('+ Add List').click();
    await page.fill('#new-list-name', 'Cleanup');
    await page.click('.modal button:has-text("Add")');

    await page.locator('.column-footer button:has-text("Add Card")').click();
    await page.fill('#new-card-title', 'Delete me');
    await page.click('.modal button:has-text("Add")');

    // Open and delete
    await page.locator('.kanban-card').click();
    await expect(page.locator('#card-panel')).toHaveClass(/open/);

    page.on('dialog', (dialog) => dialog.accept());
    await page.click('button:has-text("Delete Card")');

    // Wait for panel to close and card to disappear
    await page.waitForTimeout(500);
    await expect(page.locator('.kanban-card')).toHaveCount(0);
  });

  test('back button returns to boards', async ({ page }) => {
    // Need a list so board.html renders the back button
    await page.getByText('+ Add List').click();
    await page.fill('#new-list-name', 'Temp');
    await page.click('.modal button:has-text("Add")');
    await expect(page.locator('.column-header')).toHaveCount(1);

    // Click "Boards" link
    await page.getByText('← Boards').click();
    await page.waitForURL('/boards.html');
    await expect(page.locator('h1')).toHaveText('Boards');
  });
});
