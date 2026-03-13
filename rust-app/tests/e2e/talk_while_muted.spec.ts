import { test, expect } from '@playwright/test';

test.describe('Talk While Muted Feature', () => {
    test('Should display toast when user talks while muted', async ({ page, context }) => {
        // Since we are using fake audio streams, triggering the exact volume threshold reliably in Playwright can be flaky,
        // but we can at least simulate the event if the system permits, or verify the feature exists and skip the actual volume check
        // like the audio_features.spec.ts does. We'll simulate the custom event on the window to verify the UI response.

        await context.grantPermissions(['camera', 'microphone']);
        const roomName = `TalkMuted_${Date.now()}`;

        await page.goto(`/room/${roomName}`);
        await page.locator('.prejoin-container input[type="text"]').fill('MutedUser');
        // Join with mic explicitly muted
        const micBtn = page.locator('button:has-text("Turn Off Mic")');
        if (await micBtn.isVisible()) {
            await micBtn.click();
        }
        await page.click('button:has-text("Join Meeting")');

        await expect(page.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

        // Simulate the custom event that the AudioMonitor would fire
        await page.evaluate(() => {
            const event = new CustomEvent('talk_while_muted');
            window.dispatchEvent(event);
        });

        // Verify the toast appears
        const toast = page.locator('.toast');
        await expect(toast).toContainText('You are muted. Please unmute to speak.');
    });
});
