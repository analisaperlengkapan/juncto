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

        // Ensure the PiP button exists on the local video
        const pipButton = page.locator('.local-video button[title="Picture-in-Picture"]');
        await expect(pipButton).toBeVisible();

        // Note: Playwright doesn't easily allow checking actual native PiP state
        // without injecting complex scripts, but verifying the button exists
        // and has the correct click handler logic (via UI structure) is a good start.
    });
});
