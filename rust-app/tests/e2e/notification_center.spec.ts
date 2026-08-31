import { test, expect } from '@playwright/test';

test.describe('Notification Center', () => {
    test('bell shows unread count and lists toast history', async ({ page, context }) => {
        // Host joins
        await page.goto('/room/notif-center');
        await page.fill('#display-name', 'Host');
        await page.click('.join-btn');
        const hostPage = page;

        // Visitor joins
        const visitorPage = await context.newPage();
        await visitorPage.goto('/room/notif-center');
        await visitorPage.fill('#display-name', 'Visitor');
        await visitorPage.click('.join-btn');

        // Both connected
        await expect(hostPage.locator('.video-grid')).toBeVisible();
        await expect(visitorPage.locator('.video-grid')).toBeVisible();

        // Host triggers a toast on visitor via mute-all (participants panel)
        if (await hostPage.locator('#mute-all-btn').isHidden()) {
            await hostPage.click('#toggle-participants-btn');
        }
        await hostPage.click('#mute-all-btn');
        await expect(visitorPage.locator('.toast-container')).toContainText('You have been muted by the host');

        // Visitor accumulates unread notification -> badge visible
        const badge = visitorPage.locator('#notif-bell-btn .notif-badge');
        await expect(badge).toBeVisible();

        // Open the panel: history item listed; unread cleared
        await visitorPage.click('#notif-bell-btn');
        await expect(visitorPage.locator('#notif-panel')).toBeVisible();
        await expect(visitorPage.locator('#notif-panel')).toContainText('You have been muted by the host');
        await expect(badge).not.toBeVisible();

        // Clear history empties the list
        await visitorPage.click('#notif-panel button:has-text("Clear")');
        await expect(visitorPage.locator('#notif-panel')).not.toContainText('You have been muted by the host');

        await visitorPage.close();
    });
});
