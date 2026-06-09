import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    await page.click('button[title="Login"]');
    await page.fill('input[placeholder="Username"]', 'admin');
    await page.fill('input[placeholder="Password"]', 'admin123');
    await page.click('button:has-text("Login")');
}

test.describe('Transcription and Subtitles', () => {
    test('should display subtitles when enabled', async ({ page }) => {
        await page.goto('http://localhost:3000/room/TranscriptionRoom');
        await page.fill('#display-name', 'Speaker');
        await page.click('.join-btn');
        await loginAsAdmin(page);

        await page.click('#toggle-subtitles-btn');
        await expect(page.locator('.subtitles-overlay')).toBeVisible();
        await expect(page.locator('.subtitles-overlay')).toContainText('Subtitles are currently enabled');
    });
});
