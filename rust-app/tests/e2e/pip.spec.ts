import { test, expect } from '@playwright/test';

test.describe('Picture-in-Picture feature', () => {
    test('local video has PiP button', async ({ page }) => {
        await page.goto('/room/e2e-pip-test-room');

        // Wait for prejoin screen
        await page.waitForSelector('.prejoin-container', { timeout: 10000 });

        // Ensure camera is on in prejoin
        const camBtn = page.getByRole('button', { name: '🚫' });
        if (await camBtn.isVisible()) {
            await camBtn.click(); // Turn it on (changes from 🚫 to 📷)
        }

        // Enter a name and join
        await page.locator('.prejoin-container input[type="text"]').fill('PiP Tester');
        await page.click('button.join-btn');

        // Wait for the room to load and camera to be active
        // Sometimes the name might be the one we entered, let's just look for any video element that isn't hidden
        await page.waitForSelector('video:not([style*="display: none"])', { timeout: 15000 });

        // Ensure the PiP button exists on the local video
        // Instead of strict "Me" matching, just look for the title attribute within the container
        const pipButton = page.locator('button[title="Picture-in-Picture"]').first();
        await expect(pipButton).toBeVisible({ timeout: 15000 });

        // Note: Playwright doesn't easily allow checking actual native PiP state
        // without injecting complex scripts, but verifying the button exists
        // and has the correct click handler logic (via UI structure) is a good start.
    });

    test('PiP button is hidden when camera is off', async ({ page }) => {
        await page.goto('/room/e2e-pip-test-room');

        // Wait for prejoin screen
        await page.waitForSelector('.prejoin-container', { timeout: 10000 });

        // Enter a name
        await page.locator('.prejoin-container input[type="text"]').fill('PiP Tester');

        // Toggle camera off
        const camBtn = page.getByRole('button', { name: '📷' });
        if (await camBtn.isVisible()) {
            await camBtn.click(); // Turn it off (changes from 📷 to 🚫)
        }

        // Join
        await page.click('button.join-btn');

        // Wait for the room to load and camera to be inactive
        await page.waitForSelector('.video-card', { timeout: 15000 });

        // Ensure the PiP button does not exist on the local video (since camera is off and it's inside the Show block)
        // With only one video block (Me without camera), we should not see the button
        const pipButtons = page.locator('button[title="Picture-in-Picture"]');
        await expect(pipButtons).not.toBeVisible({ timeout: 15000 });
    });
});
