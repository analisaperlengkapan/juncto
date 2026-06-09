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










test.describe('Juncto Simple Lifecycle', () => {
    test('Basic flow from Home to Room and Promotion', async ({ context }) => {
        const roomName = `simple-${Math.random().toString(36).substring(7)}`;

        // 1. Host Session Start
        const hostPage = await context.newPage();
        await hostPage.goto('http://localhost:3000/');
        await hostPage.fill('#meeting-name', roomName);
        await hostPage.click('.create-btn');

        // Prejoin Screen
        await expect(hostPage).toHaveURL(new RegExp(`/room/${encodeURIComponent(roomName)}`));
        await hostPage.fill('#display-name', 'Host Admin');
        await hostPage.click('.join-btn');
        await loginAsAdmin(hostPage);

        // Verify Room Entry
        await expect(hostPage.locator('.room-container')).toBeVisible({ timeout: 60000 });

        // 2. Visitor Session Start
        const visitorPage = await context.newPage();
        await visitorPage.goto(`http://localhost:3000/room/${encodeURIComponent(roomName)}`);
        await visitorPage.fill('#display-name', 'Visitor Guest');
        await visitorPage.check('#visitor-mode');
        await visitorPage.click('.join-btn');

        // Verify Visitor Entry
        await expect(visitorPage.locator('.room-container')).toBeVisible({ timeout: 60000 });

        // 3. Promote Visitor
        if (await hostPage.locator('#participants-panel').getAttribute('class').then(c => c?.includes('panel-hidden'))) {
            await hostPage.click('#toggle-participants-btn');
        }
        const visitorItem = hostPage.locator('.participant-item:has-text("Visitor Guest")');
        await visitorItem.locator('#promote-btn').dispatchEvent('click');

        // Verify Visitor is now a full participant
        await expect(visitorPage.locator('#toggle-camera-btn')).toBeVisible();

        // 4. Mute All
        await hostPage.click('#mute-all-btn');
        await expect(visitorPage.locator('.toast-container')).toContainText('You have been muted by the host');

        // 5. Leave
        await hostPage.click('button[title="Leave Meeting"]');
        await expect(hostPage).toHaveURL('http://localhost:3000/');
    });
});
