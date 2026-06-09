import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    await page.click('button[title="Login"]');
    await page.fill('input[placeholder="Username"]', 'admin');
    await page.fill('input[placeholder="Password"]', 'admin123');
    await page.click('button:has-text("Login")');
}

test.describe('Settings: E2EE Toggle', () => {
    test('Host can toggle E2EE', async ({ page }) => {
        await page.goto('http://localhost:3000/room/E2EETest');
        await page.fill('#display-name', 'Host');
        await page.click('.join-btn');
        await loginAsAdmin(page);

        await page.click('#settings-btn');
        await page.click('button:has-text("Moderator")');
        await page.locator('#e2ee-toggle').check();
        await page.click('#close-settings-btn');

        await expect(page.locator('#e2ee-indicator')).toBeVisible();
    });
});
