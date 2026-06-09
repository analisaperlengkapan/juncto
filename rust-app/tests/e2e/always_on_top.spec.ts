import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    try {
        await page.click('button[title="Login"]', { timeout: 2000 });
        await page.fill('input[placeholder="user@domain.com"]', 'admin');
        await page.fill('input[placeholder="Password"]', 'admin123');
        await page.click('button:has-text("Login")');
    } catch (e) {
        // Fallback or already logged in
    }
}


test.describe('Always On Top Feature', () => {
    test('Always on top toolbar should be visible in main meeting view when connected', async ({ page, context }) => {
        // Grant permissions just in case
        await context.grantPermissions(['camera', 'microphone']);

        // Create an alias for joining a room
        const roomName = `AlwaysOnTop_${Date.now()}`;

        await page.goto(`/room/${roomName}`);
        await page.locator('.prejoin-container input[type="text"]').fill('AOT_User');
        await page.click('button:has-text("Join Meeting")');

        // Wait until we are in the room
        await expect(page.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

        // The always-on-top controls are rendered once the user is connected so that
        // mute/camera/leave actions are always reachable. Verify it appears.
        const aotContainer = page.locator('.always-on-top-container');
        await expect(aotContainer).toBeVisible();
    });
});
