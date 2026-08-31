import { expect, test } from '@playwright/test';

test.describe('Conference core parity (Step 1)', () => {
    // Right-click opens the video context menu with pin/kick/volume items
    test('video context menu opens on right-click on remote tiles', async ({ browser, request }) => {
        const roomCfg = await request.post('http://localhost:3000/api/rooms', {
            data: {
                room_name: 'CtxRoom',
                is_locked: false,
                is_recording: false,
                is_lobby_enabled: false,
                max_participants: 10,
            },
        });
        const roomName = (await roomCfg.json()).config.room_name;

        const hostCtx = await browser.newContext();
        const guestCtx = await browser.newContext();
        const hostPage = await hostCtx.newPage();
        const guestPage = await guestCtx.newPage();

        try {
            await hostPage.goto(`/room/${encodeURIComponent(roomName)}`);
            await hostPage.fill('.prejoin-container input[type="text"]', 'Host');
            await hostPage.click('button.join-btn');
            await expect(hostPage.locator('.room-container')).toBeVisible();

            await guestPage.goto(`/room/${encodeURIComponent(roomName)}`);
            await guestPage.fill('.prejoin-container input[type="text"]', 'Guest');
            await guestPage.click('button.join-btn');
            await expect(guestPage.locator('.room-container')).toBeVisible();

            // Host right-clicks on Guest's tile to open the context menu
            const remoteCard = hostPage.locator('.video-card:not(.local-video)');
            await expect(remoteCard).toBeVisible();
            await remoteCard.click({ button: 'right' });

            const menu = hostPage.locator('.video-context-menu');
            await expect(menu).toBeVisible();
            await expect(menu.locator('.context-menu-item').first()).toContainText('Pin participant');
            await expect(menu.locator('.context-menu-item', { hasText: 'Kick participant' })).toBeVisible();

            // Close with Escape
            await hostPage.keyboard.press('Escape');
            await expect(menu).not.toBeVisible();
        } finally {
            await hostCtx.close();
            await guestCtx.close();
        }
    });

    // Connection indicator is visible in the header once joined
    test('connection indicator renders in header', async ({ page, request }) => {
        await request.post('http://localhost:3000/api/rooms', {
            data: { room_name: 'ConnRoom', is_locked: false, is_recording: false, is_lobby_enabled: false, max_participants: 10 },
        });
        await page.goto('/room/ConnRoom');
        await page.fill('.prejoin-container input[type="text"]', 'Tester');
        await page.click('button.join-btn');
        await expect(page.locator('.room-container')).toBeVisible();
        await expect(page.locator('.connection-indicator')).toBeVisible();
    });

    // Layout switches via the dropdown menu
    test('layout menu lists Tile view and Speaker view options', async ({ page, request }) => {
        await request.post('http://localhost:3000/api/rooms', {
            data: { room_name: 'LayoutRoom', is_locked: false, is_recording: false, is_lobby_enabled: false, max_participants: 10 },
        });
        await page.goto('/room/LayoutRoom');
        await page.fill('.prejoin-container input[type="text"]', 'Tester');
        await page.click('button.join-btn');
        await expect(page.locator('.room-container')).toBeVisible();

        await page.click('.layout-menu-btn');
        const menu = page.locator('.layout-menu');
        await expect(menu.locator('.layout-option', { hasText: 'Tile view' })).toBeVisible();
        await expect(menu.locator('.layout-option', { hasText: 'Speaker view' })).toBeVisible();
    });
});
