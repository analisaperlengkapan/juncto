import { test, expect } from '@playwright/test';

test.describe('Always On Top Feature', () => {
    test('Always on top toolbar should be hidden in main meeting view', async ({ page, context }) => {
        // Grant permissions just in case
        await context.grantPermissions(['camera', 'microphone']);

        // Create an alias for joining a room
        const roomName = `AlwaysOnTop_${Date.now()}`;

        await page.goto(`/room/${roomName}`);
        await page.locator('.prejoin-container input[type="text"]').fill('AOT_User');
        await page.click('button:has-text("Join Meeting")');

        // Wait until we are in the room
        await expect(page.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

        // The widget was hidden from the active meeting UI in recent PR feedback to prevent duplicating the main toolbox
        // We verify that it does not appear when connected.
        const aotContainer = page.locator('.always-on-top-container');
        await expect(aotContainer).not.toBeVisible();
    });
});
