import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    await page.click('button[title="Login"]');
    await page.fill('input[placeholder="Username"]', 'admin');
    await page.fill('input[placeholder="Password"]', 'admin123');
    await page.click('button:has-text("Login")');
}


test.describe('Picture-in-Picture feature', () => {
    test('local video has PiP button', async ({ page, request }) => {
        // Reset room state to ensure clean config (no stale lock/lobby/max from prior tests)
        await request.post('http://localhost:3000/api/rooms', {
            data: {
                room_name: 'e2e-pip-test-room',
                is_locked: false,
                is_recording: false,
                is_lobby_enabled: false,
                max_participants: 100
            }
        });

        await page.goto('/room/e2e-pip-test-room');

        // Wait for prejoin screen
        await page.waitForSelector('.prejoin-container', { timeout: 10000 });

        // Ensure camera is on in prejoin.
        // The camera button shows 🚫 when off and 📷 when on.
        // Use title selector for reliability; check text content with trim
        // to handle any invisible characters from emoji rendering.
        const camBtn = page.locator('button[title="Toggle Camera"]');
        await expect(camBtn).toBeVisible();
        const camBtnText = (await camBtn.innerText()).trim();
        if (camBtnText !== '📷') {
            await camBtn.click(); // Turn it on
            // Wait for the button text to change, confirming the camera is now on
            await expect(camBtn).toContainText('📷', { timeout: 5000 });
        }

        // Enter a name
        await page.locator('.prejoin-container input[type="text"]').fill('PiP Tester');

        // Wait for Join button to be enabled (meaning WebSocket is connected)
        const joinBtn = page.locator('button.join-btn');
        await expect(joinBtn).toHaveText('Join Meeting', { timeout: 15000 });
        await joinBtn.click();

        // First confirm we've entered the room
        await expect(page.locator('h2')).toContainText('Meeting Room:', { timeout: 30000 });

        // Wait for the room to load and camera to be active
        await page.waitForSelector('.video-card.local-video video', { timeout: 30000 });

        // Ensure the PiP button exists on the local video
        const pipButton = page.locator('.video-card.local-video button[title="Picture-in-Picture"]');
        await expect(pipButton).toBeVisible({ timeout: 15000 });

        // Note: Playwright doesn't easily allow checking actual native PiP state
        // without injecting complex scripts, but verifying the button exists
        // and has the correct click handler logic (via UI structure) is a good start.
    });

    test('PiP button is hidden when camera is off', async ({ page, request }) => {
        // Reset room state to ensure clean config (no stale lock/lobby/max from prior tests)
        await request.post('http://localhost:3000/api/rooms', {
            data: {
                room_name: 'e2e-pip-test-room',
                is_locked: false,
                is_recording: false,
                is_lobby_enabled: false,
                max_participants: 100
            }
        });

        await page.goto('/room/e2e-pip-test-room');

        // Wait for prejoin screen
        await page.waitForSelector('.prejoin-container', { timeout: 10000 });

        // Enter a name
        await page.locator('.prejoin-container input[type="text"]').fill('PiP Tester');

        // Ensure camera is off in prejoin.
        // The camera button shows 🚫 when off and 📷 when on.
        const camBtn = page.locator('button[title="Toggle Camera"]');
        await expect(camBtn).toBeVisible();
        const camBtnText = (await camBtn.innerText()).trim();
        if (camBtnText !== '🚫') {
            await camBtn.click(); // Turn it off
            await expect(camBtn).toContainText('🚫', { timeout: 5000 });
        }

        // Wait for Join button to be enabled (meaning WebSocket is connected)
        const joinBtn = page.locator('button.join-btn');
        await expect(joinBtn).toHaveText('Join Meeting', { timeout: 15000 });
        await joinBtn.click();

        // First confirm we've entered the room
        await expect(page.locator('h2')).toContainText('Meeting Room:', { timeout: 30000 });

        // Wait for the room to load — the local-video card always renders,
        // but when camera is off the "Camera Off" fallback text appears instead of <video>.
        await page.waitForSelector('.video-card.local-video', { timeout: 30000 });

        // Ensure the PiP button does not exist on the local video (since camera is off and it's inside the Show block)
        const pipButton = page.locator('.video-card.local-video button[title="Picture-in-Picture"]');
        await expect(pipButton).toHaveCount(0, { timeout: 10000 });
    });
});
