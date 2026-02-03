import { test, expect } from '@playwright/test';

test.describe('Keyboard Shortcuts', () => {
    test('Toggle Mute with M key and Open Shortcuts Dialog', async ({ browser }) => {
        const roomName = `ShortcutRoom_${Date.now()}`;
        const context = await browser.newContext();
        const page = await context.newPage();
        page.on('console', msg => console.log(`PAGE LOG: ${msg.text()}`));
        await page.goto('/');

        // Create and Join
        await page.fill('input[type="text"]', roomName);
        await page.click('button:has-text("Start Meeting")');
        await page.locator('.prejoin-container input[type="text"]').fill('Alice');
        await page.click('button:has-text("Join Meeting")');
        await expect(page.locator('.video-grid')).toBeVisible();

        // 1. Check Initial Mute State (Unmuted)
        // Mute button says "Mute" (green background)
        await expect(page.locator('button:has-text("Mute")')).toBeVisible();

        // Verify button click works first
        await page.click('button:has-text("Mute")');
        await expect(page.locator('button:has-text("Unmute")')).toBeVisible();
        await page.click('button:has-text("Unmute")');
        await expect(page.locator('button:has-text("Mute")')).toBeVisible();

        // Ensure focus is on the body
        await page.click('body');

        // 2. Press 'M' to toggle mute
        await page.keyboard.press('m');

        // Check State (Muted)
        // Button should say "Unmute" (red background)
        await expect(page.locator('button:has-text("Unmute")')).toBeVisible();

        // 3. Press 'M' again to unmute
        await page.keyboard.press('m');
        await expect(page.locator('button:has-text("Mute")')).toBeVisible();

        // 4. Open Shortcuts Dialog with '?' button
        await page.click('button[title="Keyboard Shortcuts"]');
        await expect(page.locator('h3:has-text("Keyboard Shortcuts")')).toBeVisible();
        await expect(page.locator('li:has-text("Toggle Microphone")')).toBeVisible();

        // Close dialog
        await page.click('.modal-header button');
        await expect(page.locator('h3:has-text("Keyboard Shortcuts")')).not.toBeVisible();

        await context.close();
    });
});
