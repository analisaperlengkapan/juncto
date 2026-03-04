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
        await page.waitForSelector('.local-video video', { timeout: 10000 });

        // Ensure the PiP button exists on the local video
        const pipButton = page.locator('.local-video button[title="Picture-in-Picture"]');
        await expect(pipButton).toBeVisible();

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
        await page.waitForSelector('.local-video', { timeout: 10000 });

        // Ensure the PiP button does not exist on the local video (since camera is off and it's inside the Show block)
        const pipButton = page.locator('.local-video button[title="Picture-in-Picture"]');
        await expect(pipButton).toBeHidden();
    });
});
