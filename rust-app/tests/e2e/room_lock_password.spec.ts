import { expect, test } from '@playwright/test';

test.describe('Room lock password parity (Step 4)', () => {
    test('password-protected lock rejects, then admits, joins with the password', async ({
        browser,
        request,
    }) => {
        const roomResp = await request.post('http://localhost:3000/api/rooms', {
            data: {
                room_name: 'LockPwd',
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

            // Host: open settings → moderator tab, set password, lock the room
            await hostPage.click('#settings-btn');
            await hostPage.click('button:has-text("Moderator")');
            await hostPage.fill('#lock-password-input', 's3cret');
            await hostPage.click('#lock-room-toggle');
            await hostPage.click('#close-settings-btn');

            // Guest joins without password → rejected with prompt
            const guestPage = await guestCtx.newPage();
            await guestPage.goto(roomUrl);
            await guestPage.fill('.prejoin-container input[type="text"]', 'Guest');
            await guestPage.click('button.join-btn');
            await expect(guestPage.locator('.toast-container .toast-error'))
                .toHaveText('Password required', { timeout: 10000 });
            await expect(guestPage.locator('#room-password')).toBeVisible();

            // Wrong password → rejected again
            await guestPage.fill('#room-password', 'wrong');
            await guestPage.click('button.join-btn');
            await expect(
                guestPage.locator('.toast-container .toast-error').filter({
                    hasText: 'Invalid room password',
                }),
            ).toBeVisible({ timeout: 10000 });

            // Correct password → admitted
            await guestPage.fill('#room-password', 's3cret');
            await guestPage.click('button.join-btn');
            await expect(guestPage.locator('.room-container')).toBeVisible({ timeout: 10000 });
            await expect(hostPage.locator('.participant-item')).toHaveCount(2, {
                timeout: 10000,
            });
        } finally {
            await hostCtx.close();
            await guestCtx.close();
        }
    });

    test('hard lock without password rejects joins outright', async ({
        browser,
        request,
    }) => {
        const roomResp = await request.post('http://localhost:3000/api/rooms', {
            data: {
                room_name: 'HardLock',
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

            // Host locks with no password
            await hostPage.click('#settings-btn');
            await hostPage.click('button:has-text("Moderator")');
            await hostPage.click('#lock-room-toggle');
            await hostPage.click('#close-settings-btn');

            const guestPage = await guestCtx.newPage();
            await guestPage.goto(roomUrl);
            await guestPage.fill('.prejoin-container input[type="text"]', 'Guest');
            await guestPage.click('button.join-btn');
            await expect(guestPage.locator('.toast-container .toast-error'))
                .toHaveText('Room is locked', { timeout: 10000 });
            // No password prompt for a hard lock
            await expect(guestPage.locator('#room-password')).toBeHidden();
        } finally {
            await hostCtx.close();
            await guestCtx.close();
        }
    });
});
