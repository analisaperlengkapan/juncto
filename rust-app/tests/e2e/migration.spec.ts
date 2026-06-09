import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    await page.click('button[title="Login"]');
    await page.fill('input[placeholder="Username"]', 'admin');
    await page.fill('input[placeholder="Password"]', 'admin123');
    await page.click('button:has-text("Login")');
}

test.describe('Juncto Migration E2E (WASM)', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:3000/room/MigrationRoom');
    await page.fill('#display-name', 'Migration Tester');
    await page.click('.join-btn');
    await loginAsAdmin(page);
  });

  test('Recording and Toolbox presence', async ({ page }) => {
    await page.click('button:has-text("Start Recording")');
    await expect(page.getByText('REC', { exact: true })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Stop Recording' })).toBeVisible();
    await page.click('button:has-text("Stop Recording")');
    await expect(page.getByText('REC', { exact: true })).toBeHidden();
  });

  test('Lobby flow', async ({ context, page }) => {
    // Enable lobby
    await page.click('#settings-btn');
    await page.click('button:has-text("Moderator")');
    await page.locator('#lobby-toggle').check();
    await page.click('#close-settings-btn');

    const guestPage = await context.newPage();
    await guestPage.goto('http://localhost:3000/room/MigrationRoom');
    await guestPage.fill('#display-name', 'Guest User');
    await guestPage.click('.join-btn');

    await expect(guestPage.getByText('Waiting for host...')).toBeVisible();

    await page.click('#toggle-participants-btn');
    await expect(page.locator('.knocking-list li:has-text("Guest User")')).toBeVisible();
    await page.click('button:has-text("Allow")');

    await expect(guestPage.locator('.room-container')).toBeVisible();
  });
});

test.describe('Feature Toasts E2E', () => {
    test('Recording Toast', async ({ page }) => {
        await page.goto('http://localhost:3000/room/ToastRoom');
        await page.fill('#display-name', 'Toaster');
        await page.click('.join-btn');
        await loginAsAdmin(page);

        await page.click('button:has-text("Start Recording")');
        await expect(page.locator('.toast-container')).toContainText('Recording Started');
    });
});
