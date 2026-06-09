import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    await page.click('button[title="Login"]');
    await page.fill('input[placeholder="Username"]', 'admin');
    await page.fill('input[placeholder="Password"]', 'admin123');
    await page.click('button:has-text("Login")');
}

test.describe('AV Moderation Flow', () => {
    test('Host can enable moderation and grant permissions', async ({ context }) => {
        const hostPage = await context.newPage();
        await hostPage.goto('http://localhost:3000/room/AVMod');
        await hostPage.fill('#display-name', 'Host');
        await hostPage.click('.join-btn');
        await loginAsAdmin(hostPage);

        // Enable AV Moderation
        await hostPage.click('#settings-btn');
        await hostPage.click('button:has-text("Moderator")');
        await hostPage.locator('#audio-mod-toggle').check();
        await hostPage.click('#close-settings-btn');

        const userPage = await context.newPage();
        await userPage.goto('http://localhost:3000/room/AVMod');
        await userPage.fill('#display-name', 'User');
        await userPage.click('.join-btn');

        await expect(userPage.locator('#request-unmute-btn')).toBeVisible();
        await userPage.click('#request-unmute-btn');

        await hostPage.click('#toggle-participants-btn');
        await hostPage.locator('.participant-item:has-text("User")').locator('button:has-text("Grant Mic")').click();

        await expect(userPage.locator('.toast-container')).toContainText('permission to unmute');
    });
});
