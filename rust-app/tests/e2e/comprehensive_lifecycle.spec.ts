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




test.describe('Comprehensive Lifecycle', () => {
    test('Host and Guest interactions including login and breakout', async ({ context }) => {
        const roomName = `comp-${Math.random().toString(36).substring(7)}`;

        const hostPage = await context.newPage();
        await hostPage.goto('http://localhost:3000/');
        await hostPage.fill('#meeting-name', roomName);
        await hostPage.click('.create-btn');

        await hostPage.fill('#display-name', 'Host');
        await hostPage.click('.join-btn');
        await loginAsAdmin(hostPage);

        const guestPage = await context.newPage();
        await guestPage.goto(`http://localhost:3000/room/${roomName}`);
        await guestPage.fill('#display-name', 'Guest');
        await guestPage.click('.join-btn');

        // Chat
        await guestPage.fill('#chat-input', 'Hi Host');
        await guestPage.press('#chat-input', 'Enter');
        await expect(hostPage.locator('.chat-message:has-text("Hi Host")').first()).toBeVisible();

        // Breakout
        await hostPage.fill('input[placeholder="New Room Name"]', 'SideRoom');
        await hostPage.click('button:has-text("Create")');
        await hostPage.click('button:has-text("Auto Assign")');
        await expect(guestPage.locator('h4:has-text("(In Breakout Room)")')).toBeVisible({ timeout: 10000 });

        // End
        await hostPage.click('button[title="End Meeting for Everyone"]');
        await expect(hostPage).toHaveURL('http://localhost:3000/', { timeout: 10000 });
    });
});
