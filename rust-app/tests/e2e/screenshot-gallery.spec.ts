import { test, expect } from '@playwright/test';

const OUT = process.env.SCREENSHOT_DIR || '../screenshots';

test.describe('UI screenshot gallery', () => {
    test('capture all main views', async ({ page, context, request }) => {
        const room = `GalleryRoom_${Date.now()}`;

        // Reset global room state (backend is a single global room)
        await request.post('http://localhost:3000/api/rooms', {
            data: {
                room_name: room,
                is_locked: false,
                is_recording: false,
                is_lobby_enabled: false,
                max_participants: 100,
            },
        });

        // 1. Home page
        await page.goto('/');
        await page.screenshot({ path: `${OUT}/01-home.png`, fullPage: true });

        // 2. Prejoin screen
        await page.goto(`/room/${room}`);
        await page.waitForSelector('#display-name');
        await page.screenshot({ path: `${OUT}/02-prejoin.png`, fullPage: true });

        // 3. Room view (Alice)
        await page.fill('#display-name', 'Alice');
        await page.click('.join-btn');
        await expect(page.locator('.room-container')).toBeVisible({ timeout: 15000 });
        await page.waitForTimeout(1000);

        // Join Bob on a second page so we have 2 tiles
        const page2 = await context.newPage();
        await page2.goto(`/room/${room}`);
        await page2.fill('#display-name', 'Bob');
        await page2.click('.join-btn');
        await expect(page2.locator('.room-container')).toBeVisible();
        await page.waitForTimeout(1500);
        await page.screenshot({ path: `${OUT}/03-room.png`, fullPage: true });

        // 4. Participants panel (already open by default, else toggle)
        if (!await page.locator('#participant-search').isVisible().catch(() => false)) {
            await page.click('#toggle-participants-btn');
        }
        await page.waitForTimeout(500);
        await page.screenshot({ path: `${OUT}/04-participants.png`, fullPage: true });

        // 5. Chat panel
        if (!await page.locator('.chat-container').isVisible().catch(() => false)) {
            await page.click('#toggle-chat-btn').catch(() => {});
        }
        // Send a message from Bob to make chat non-empty
        if (await page2.locator('#chat-input').isVisible().catch(() => false)) {
            await page2.fill('#chat-input', 'Halo, kita sudah migrasi ke Rust!');
            await page2.keyboard.press('Enter');
        }
        await page.waitForTimeout(800);
        await page.screenshot({ path: `${OUT}/05-chat.png`, fullPage: true });

        // 6. Settings dialog
        await page.click('#settings-btn');
        await page.waitForTimeout(500);
        await page.screenshot({ path: `${OUT}/06-settings.png`, fullPage: true });

        // 7. Polls dialog
        await page.click('#close-settings-btn');
        await page.click('#toggle-polls-btn');
        await page.waitForTimeout(500);
        await page.screenshot({ path: `${OUT}/07-polls.png`, fullPage: true });
        await page.click('#close-polls-btn').catch(() => {});

        // 8. Notification bell panel
        await page.click('#notif-bell-btn');
        await page.waitForTimeout(400);
        await page.screenshot({ path: `${OUT}/08-notifications.png`, fullPage: true });
        await page.click('#notif-bell-btn'); // close

        // 9. Mobile viewport home (480px)
        await page.setViewportSize({ width: 480, height: 800 });
        await page.goto('/');
        await page.waitForTimeout(500);
        await page.screenshot({ path: `${OUT}/09-mobile-home.png`, fullPage: true });

        // 10. Mobile viewport room
        await page.goto('/room/MobileRoom');
        await page.waitForSelector('#display-name');
        await page.fill('#display-name', 'Mobile');
        await page.click('.join-btn');
        await expect(page.locator('.room-container')).toBeVisible({ timeout: 15000 });
        await page.waitForTimeout(1000);
        await page.screenshot({ path: `${OUT}/10-mobile-room.png`, fullPage: true });

        await page2.close();
    });
});
