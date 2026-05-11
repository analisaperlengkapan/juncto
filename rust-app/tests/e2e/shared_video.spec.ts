import { test, expect } from '@playwright/test';

test.describe('Shared Video', () => {
    test.beforeEach(async ({ request }) => {
        // Reset room state
        const response = await request.post('/api/rooms', {
            data: {
                room_name: 'Test Room',
                is_locked: false,
                is_recording: false,
                is_lobby_enabled: false,
                max_participants: 100,
                host_id: null,
                e2ee_enabled: false
            }
        });
        expect(response.ok()).toBeTruthy();
        expect(response.status()).toBe(201);
    });

    test('Host can share a video', async ({ browser }) => {
        const roomName = `VideoRoom_${Date.now()}`;
        const contextA = await browser.newContext();
        const pageA = await contextA.newPage();
        await pageA.goto('/');

        // User A (Host)
        await pageA.fill('#meeting-name', roomName);
        await pageA.click('.create-btn');
        await pageA.waitForSelector('#display-name');
        await pageA.fill('#display-name', 'Alice');
        await pageA.click('.join-btn');
        await pageA.waitForSelector('.room-container');

        // Click Share Video
        await pageA.click('button:has-text("Video")');

        // Verify Modal appears and submit
        await expect(pageA.locator('h3:has-text("Share Video")')).toBeVisible();
        await pageA.locator('input[placeholder*="youtube.com"]').fill('https://www.youtube.com/watch?v=dQw4w9WgXcQ');
        // Click Share inside the modal
        await pageA.locator('#submit-shared-video-btn').click();

        // Verify Video Card appears
        await expect(pageA.locator('.shared-video')).toBeVisible();
        await expect(pageA.locator('iframe[src*="dQw4w9WgXcQ"]')).toBeVisible();

        // User B joins
        const contextB = await browser.newContext();
        const pageB = await contextB.newPage();
        await pageB.goto(`/room/${roomName}`);
        await pageB.waitForSelector('#display-name');
        await pageB.fill('#display-name', 'Bob');
        await pageB.click('.join-btn');
        await pageB.waitForSelector('.room-container');

        // Verify Video Card appears for Bob
        await expect(pageB.locator('.shared-video')).toBeVisible();
        await expect(pageB.locator('iframe[src*="dQw4w9WgXcQ"]')).toBeVisible();

        // Host Stops Video
        // When sharing video, button text should change to "Stop Video" or similar
        // Let's check what the button says or use ID if available
        await pageA.click('button:has-text("Stop Video")');

        // Verify removed
        await expect(pageA.locator('.shared-video')).not.toBeVisible();
        await expect(pageB.locator('.shared-video')).not.toBeVisible();

        await contextA.close();
        await contextB.close();
    });
});
