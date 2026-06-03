import { test, expect } from '@playwright/test';

test.describe('Moderation and Lobby Features', () => {
    test('Host can lock room and manage lobby', async ({ browser }) => {
        const roomName = `LobbyTest_${Date.now()}`;

        // 1. Host joins and locks the room
        const hostContext = await browser.newContext();
        const hostPage = await hostContext.newPage();
        await hostPage.goto('/');
        await hostPage.fill('#meeting-name', roomName);
        await hostPage.click('.create-btn');
        await hostPage.fill('#display-name', 'Host');
        await hostPage.click('.join-btn');
        await expect(hostPage.locator('h2')).toContainText(roomName);

        // Enable Lobby in Settings
        await hostPage.click('button[title="Settings"]');
        await hostPage.click('button:has-text("Moderator")');
        await hostPage.check('#lobby-toggle');
        await hostPage.click('#close-settings-btn');

        // 2. Guest tries to join and gets put in lobby
        const guestContext = await browser.newContext();
        const guestPage = await guestContext.newPage();
        await guestPage.goto(`/room/${roomName}`);
        await guestPage.fill('#display-name', 'Guest');
        await guestPage.click('.join-btn');

        await expect(guestPage.locator('h2')).toContainText('Waiting for host...');

        // 3. Host sees guest in participants list and allows them
        await hostPage.click('button[title="Toggle Participants"]');
        const knockList = hostPage.locator('.knocking-list');
        await expect(knockList).toContainText('Guest');

        await hostPage.locator('.knocking-list .btn-success:has-text("Allow")').first().dispatchEvent('click');

        // 4. Guest is admitted
        await expect(guestPage.locator('h2')).toContainText(roomName);
        await expect(guestPage.locator('.room-container')).toBeVisible();

        await hostContext.close();
        await guestContext.close();
    });
});
