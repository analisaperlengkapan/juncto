import { expect, test } from '@playwright/test';

test.describe('E2EE parity (Step 4)', () => {
    test('participant E2EE toggle broadcasts UpdateE2EE and shows lock indicator', async ({
        browser,
        request,
    }) => {
        const roomResp = await request.post('http://localhost:3000/api/rooms', {
            data: {
                room_name: 'E2EEPart',
                is_locked: false,
                is_recording: false,
                is_lobby_enabled: false,
                max_participants: 10,
            },
        });
        const roomName = (await roomResp.json()).config.room_name;
        const roomUrl = `/room/${encodeURIComponent(roomName)}`;

        const hostCtx = await browser.newContext();
        const guestCtx = await browser.newContext();

        try {
            const hostPage = await hostCtx.newPage();
            await hostPage.goto(roomUrl);
            await hostPage.fill('.prejoin-container input[type="text"]', 'Host');
            await hostPage.click('button.join-btn');
            await expect(hostPage.locator('.room-container')).toBeVisible();

            const guestPage = await guestCtx.newPage();
            await guestPage.goto(roomUrl);
            await guestPage.fill('.prejoin-container input[type="text"]', 'Guest');
            await guestPage.click('button.join-btn');
            await expect(guestPage.locator('.room-container')).toBeVisible();

            // Check participant e2ee toggle (profile tab is shown by default)
            await guestPage.click('#settings-btn');
            await guestPage.locator('#e2ee-participant-toggle').click();

            // Host should see lock icon on guest's tile
            await expect(hostPage.locator('.e2ee-lock').first()).toBeVisible({
                timeout: 10000,
            });

            // Untoggle → lock icon disappears
            await guestPage.locator('#e2ee-participant-toggle').click();
            await expect(hostPage.locator('.e2ee-lock')).toHaveCount(0, {
                timeout: 10000,
            });
            await guestPage.click('#close-settings-btn');
        } finally {
            await hostCtx.close();
            await guestCtx.close();
        }
    });

    test('host toggles room E2EE from moderator tab', async ({
        browser,
        request,
    }) => {
        const roomResp = await request.post('http://localhost:3000/api/rooms', {
            data: {
                room_name: 'E2EERoom',
                is_locked: false,
                is_recording: false,
                is_lobby_enabled: false,
                max_participants: 10,
            },
        });
        const roomName = (await roomResp.json()).config.room_name;
        const roomUrl = `/room/${encodeURIComponent(roomName)}`;

        const hostCtx = await browser.newContext();
        const guestCtx = await browser.newContext();

        try {
            const hostPage = await hostCtx.newPage();
            await hostPage.goto(roomUrl);
            await hostPage.fill('.prejoin-container input[type="text"]', 'Host');
            await hostPage.click('button.join-btn');
            await expect(hostPage.locator('.room-container')).toBeVisible();

            const guestPage = await guestCtx.newPage();
            await guestPage.goto(roomUrl);
            await guestPage.fill('.prejoin-container input[type="text"]', 'Guest');
            await guestPage.click('button.join-btn');
            await expect(guestPage.locator('.room-container')).toBeVisible();

            // Host toggles room E2EE via moderator tab
            await hostPage.click('#settings-btn');
            await hostPage.click('button:has-text("Moderator")');
            // room-level e2ee toggle
            await hostPage.locator('#e2ee-toggle').click();

            // Both host and guest see the E2EE room banner
            await expect(hostPage.locator('#e2ee-indicator')).toBeVisible({
                timeout: 10000,
            });
            // Wait a moment for RoomUpdated to reach the guest
            await expect(guestPage.locator('#e2ee-indicator')).toBeVisible({
                timeout: 10000,
            });
        } finally {
            await hostCtx.close();
            await guestCtx.close();
        }
    });
});
