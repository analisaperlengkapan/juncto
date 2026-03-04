import { test, expect } from '@playwright/test';

test.describe('Always On Top Feature', () => {
    test('Always on top toolbar controls should toggle media state and leave meeting', async ({ page, context }) => {
        // Grant permissions just in case
        await context.grantPermissions(['camera', 'microphone']);

        // Create an alias for joining a room
        const roomName = `AlwaysOnTop_${Date.now()}`;

        await page.goto(`/room/${roomName}`);
        await page.locator('.prejoin-container input[type="text"]').fill('AOT_User');
        await page.click('button:has-text("Join Meeting")');

        // Wait until we are in the room
        await expect(page.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

        // Verify the always-on-top container is present
        const aotContainer = page.locator('.always-on-top-container');
        await expect(aotContainer).toBeVisible();

        // Find buttons in the AOT toolbar
        const audioBtn = aotContainer.locator('button').first();
        const videoBtn = aotContainer.locator('button').nth(1);
        const leaveBtn = aotContainer.locator('.hangup-button');

        // Test Audio Toggle
        await expect(audioBtn).toHaveText('🎤');
        await audioBtn.click();
        await expect(audioBtn).toHaveText('🔇');

        // Test Video Toggle
        // In headless testing environments, accessing actual video hardware might fail, so we wait for the reaction
        // The browser might start with video off due to missing fake devices, so we verify the toggle changes state
        const initialVideoState = await videoBtn.innerText();
        await videoBtn.click();
        const expectedVideoState = initialVideoState === '📷' ? '🚫' : '📷';
        await expect(videoBtn).toHaveText(expectedVideoState, { timeout: 10000 });

        // Test Leave Meeting
        await leaveBtn.click();

        // Ensure user is redirected to the home screen
        await expect(page).toHaveURL(/\/$/);
        await expect(page.getByText('Welcome to Juncto')).toBeVisible();
    });
});
