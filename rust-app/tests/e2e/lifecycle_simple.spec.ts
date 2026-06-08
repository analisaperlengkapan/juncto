import { test, expect } from '@playwright/test';

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
