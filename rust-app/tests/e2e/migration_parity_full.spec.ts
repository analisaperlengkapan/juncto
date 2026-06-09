import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    await page.click('button[title="Login"]');
    await page.fill('input[placeholder="Username"]', 'admin');
    await page.fill('input[placeholder="Password"]', 'admin123');
    await page.click('button:has-text("Login")');
}

test.describe('Migration Parity Features Full', () => {
    test('Local Recording and Host Request Unmute', async ({ context }) => {
        const hostPage = await context.newPage();
        await hostPage.goto('http://localhost:3000/room/ParityFull');
        await hostPage.fill('#display-name', 'Host');
        await hostPage.click('.join-btn');
        await loginAsAdmin(hostPage);

        const guestPage = await context.newPage();
        await guestPage.goto('http://localhost:3000/room/ParityFull');
        await guestPage.fill('#display-name', 'Guest');
        await guestPage.click('.join-btn');

        await guestPage.click('#toggle-local-record-btn');
        await expect(hostPage.locator('.toast-container')).toContainText('started recording locally');

        await hostPage.click('#toggle-participants-btn');
        await hostPage.locator('.participant-item:has-text("Guest")').locator('button:has-text("Ask Unmute")').click();

        await expect(guestPage.locator('.toast-container')).toContainText('asked you to unmute');
    });
});
