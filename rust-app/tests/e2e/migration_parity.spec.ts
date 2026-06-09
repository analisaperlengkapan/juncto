import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    await page.click('button[title="Login"]');
    await page.fill('input[placeholder="Username"]', 'admin');
    await page.fill('input[placeholder="Password"]', 'admin123');
    await page.click('button:has-text("Login")');
}

test.describe('Migration Parity Features', () => {
    test('Host promotes Visitor and synchronizes E2EE', async ({ context }) => {
        const hostPage = await context.newPage();
        await hostPage.goto('http://localhost:3000/room/ParityRoom');
        await hostPage.fill('#display-name', 'Host');
        await hostPage.click('.join-btn');
        await loginAsAdmin(hostPage);

        const visitorPage = await context.newPage();
        await visitorPage.goto('http://localhost:3000/room/ParityRoom');
        await visitorPage.fill('#display-name', 'Visitor');
        await visitorPage.check('#visitor-mode');
        await visitorPage.click('.join-btn');

        await hostPage.click('#toggle-participants-btn');
        await hostPage.locator('.participant-item:has-text("Visitor")').locator('#promote-btn').click();

        await expect(visitorPage.locator('#toggle-camera-btn')).toBeVisible({ timeout: 10000 });

        // Synchronize E2EE
        await hostPage.click('#settings-btn');
        await hostPage.click('button:has-text("Moderator")');
        await hostPage.locator('#e2ee-toggle').check();
        await hostPage.click('#close-settings-btn');

        await expect(hostPage.locator('.e2ee-lock')).toHaveCount(4, { timeout: 10000 });
        await expect(visitorPage.locator('.e2ee-lock')).toHaveCount(4, { timeout: 10000 });
    });
});
