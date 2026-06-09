import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    await page.click('button[title="Login"]');
    await page.fill('input[placeholder="Username"]', 'admin');
    await page.fill('input[placeholder="Password"]', 'admin123');
    await page.click('button:has-text("Login")');
}

test.describe('Subtitles and Presence Status', () => {
    test('Subtitles toggle and Presence Display', async ({ page }) => {
        await page.goto('http://localhost:3000/room/SubsPresence');
        await page.fill('#display-name', 'Tester');
        await page.click('.join-btn');
        await loginAsAdmin(page);

        // Toggle Subtitles
        await page.click('#toggle-subtitles-btn');
        await expect(page.locator('.subtitles-overlay')).toBeVisible();

        // Change Presence
        await page.selectOption('#presence-select', 'Busy');
        await page.click('#toggle-participants-btn');
        await expect(page.locator('.participant-item:has-text("Tester")')).toContainText('Busy');
    });
});
