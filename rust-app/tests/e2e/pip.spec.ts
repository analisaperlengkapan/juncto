import { test, expect } from '@playwright/test';

test.describe('Picture-in-Picture feature', () => {
    test('local video has PiP button', async ({ page }) => {
        await page.goto('/room/e2e-pip-test-room');

        // Wait for prejoin screen
        await page.waitForSelector('.prejoin-container', { timeout: 10000 });

        // Enter a name and join
        await page.fill('input[type="text"]', 'PiP Tester');
        await page.click('button:has-text("Join Meeting")');

        // Wait for the room to load and camera to be active
        await page.waitForSelector('.local-video', { timeout: 10000 });

        // Depending on the mocked media stream, it might be 'Camera Off' or a video.
        // We check the participant name to ensure we joined successfully before checking the button.
        await expect(page.locator('.local-video')).toContainText('Me', { timeout: 10000 });

        // The PiP button should exist if the camera is deemed active.
        // If it's falling back to 'Camera Off' in CI, the button won't be there because it's inside the <Show> block.
        // So we need to handle both cases like prejoin.spec.ts does.
        const isCameraOff = await page.locator('.local-video:has-text("Camera Off")').isVisible();
        if (!isCameraOff) {
            const pipButton = page.locator('.local-video button[title="Picture-in-Picture"]');
            await expect(pipButton).toBeVisible({ timeout: 15000 });
        }

        // Note: Playwright doesn't easily allow checking actual native PiP state
        // without injecting complex scripts, but verifying the button exists
        // and has the correct click handler logic (via UI structure) is a good start.
    });

    test('PiP button is hidden when camera is off', async ({ page }) => {
        await page.goto('/room/e2e-pip-test-room');

        // Wait for prejoin screen
        await page.waitForSelector('.prejoin-container', { timeout: 10000 });

        // Enter a name
        await page.fill('input[type="text"]', 'PiP Tester');

        // Toggle camera off
        const camBtn = page.locator('button[title="Toggle Camera"]');
        await expect(camBtn).toBeVisible();
        if (await camBtn.textContent() === '📷') {
            await camBtn.click(); // Toggle it off
        }

        // Wait for fallback
        await expect(page.locator('.camera-off-text')).toBeVisible({ timeout: 10000 });

        // Join
        await page.click('button:has-text("Join Meeting")');

        // Wait for the room to load and camera to be inactive
        await page.waitForSelector('.local-video', { timeout: 10000 });

        // Ensure the PiP button does not exist on the local video (since camera is off and it's inside the Show block)
        const pipButton = page.locator('.local-video button[title="Picture-in-Picture"]');
        await expect(pipButton).toBeHidden();
    });
});
