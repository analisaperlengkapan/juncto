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
        await pageA.fill('input[type="text"]', roomName);
        await pageA.click('button:has-text("Start Meeting")');
        await pageA.locator('.prejoin-container input[type="text"]').fill('Alice');
        await pageA.click('button:has-text("Join Meeting")');

        // Click Share Video
        await pageA.click('button:has-text("Share Video")');

        // Verify Modal appears and submit
        await expect(pageA.locator('h3:has-text("Share Video")')).toBeVisible();
        await pageA.locator('input[placeholder*="youtube.com"]').fill('https://www.youtube.com/watch?v=dQw4w9WgXcQ');
        // Click Share inside the modal (exact match to avoid "Share Screen")
        await pageA.locator('.modal-content button:has-text("Share")').click();

        // Verify Video Card appears
        await expect(pageA.locator('.shared-video')).toBeVisible();
        await expect(pageA.locator('iframe[src*="dQw4w9WgXcQ"]')).toBeVisible();

        // User B joins
        const contextB = await browser.newContext();
        const pageB = await contextB.newPage();
        await pageB.goto(`/room/${roomName}`);
        await pageB.locator('.prejoin-container input[type="text"]').fill('Bob');
        await pageB.click('button:has-text("Join Meeting")');

        // Verify Video Card appears for Bob
        await expect(pageB.locator('.shared-video')).toBeVisible();
        await expect(pageB.locator('iframe[src*="dQw4w9WgXcQ"]')).toBeVisible();

        // Host Stops Video
        await pageA.click('button:has-text("Stop Video")');

        // Verify removed
        await expect(pageA.locator('.shared-video')).not.toBeVisible();
        await expect(pageB.locator('.shared-video')).not.toBeVisible();

        await contextA.close();
        await contextB.close();
    });
});
