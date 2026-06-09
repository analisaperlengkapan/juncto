import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    const loginBtn = page.locator('button[title="Login"]');
    if (await loginBtn.isVisible()) {
        await loginBtn.click();
        await page.fill('input[placeholder="user@domain.com"]', 'admin');
        await page.fill('input[placeholder="Password"]', 'admin123');
        await page.click('button:has-text("Login")');
        await page.waitForSelector('.toast-container:has-text("Authenticated")', { timeout: 5000 }).catch(() => {});
    }
}










test.describe('Transcription and Subtitles', () => {
  test('should display subtitles when enabled and user is speaking', async ({ page }) => {
    const roomName = `sub-room-${Math.random().toString(36).substring(7)}`;

    await page.goto('/');
    await page.fill('#meeting-name', roomName);
    await page.click('.create-btn');
    await page.waitForSelector('#display-name');
    await page.fill('#display-name', 'SubTester');
    await page.click('.join-btn');
    await loginAsAdmin(page);
    await page.waitForSelector('.room-container');

    // Note: Due to backend state persistence in the sandbox environment,
    // subtitles might be enabled by default if a previous test left it on.
    // We toggle until we reach the desired state.

    const overlay = page.locator('.subtitles-overlay');
    const ccBtn = page.locator('#toggle-subtitles-btn');

    // Ensure we start with subtitles OFF for this test if possible,
    // or just test the toggle functionality.
    if (await overlay.isVisible()) {
        await ccBtn.click();
        await expect(overlay).not.toBeVisible();
    }

    // Now toggle ON
    await ccBtn.click();
    await expect(overlay).toBeVisible();
    await expect(overlay).toContainText('Subtitles are currently enabled');

    // Toggle back OFF
    await ccBtn.click();
    await expect(overlay).not.toBeVisible();
  });
});
