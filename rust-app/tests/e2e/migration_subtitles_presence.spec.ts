import { test, expect } from '@playwright/test';

test.describe('Subtitles and Presence Status Features', () => {

  test('Subtitles toggle and Presence Display', async ({ browser }) => {
    const context = await browser.newContext();
    const page = await context.newPage();

    // 1. Join room
    const roomName = `SubRoom_${Date.now()}`;
    await page.goto(`http://localhost:3000/room/${roomName}`);

    // Prejoin screen
    await page.fill('input[type="text"]', 'SubtitleTestUser');
    await page.click('button:has-text("Join Meeting")');

    // Wait for the room to load
    if (await page.locator('.participants-list').isHidden()) { await page.click('.toolbox button:has-text("Participants")'); }
    await expect(page.locator('.participants-list')).toBeVisible({ timeout: 10000 });

    // 2. Verify Presence Status "Connected" is displayed next to name
    const participantLocator = page.locator('.participants-list li:has-text("SubtitleTestUser")');
    await expect(participantLocator).toContainText('[Connected]');

    // 3. Toggle Subtitles
    const subtitlesButton = page.locator('button:has-text("Show Subtitles")');
    await expect(subtitlesButton).toBeVisible();
    await subtitlesButton.click();

    // Verify overlay appears
    const overlay = page.locator('.subtitles-overlay');
    await expect(overlay).toBeVisible();
    await expect(overlay).toContainText('Subtitles are currently enabled');

    // Verify button text changed to "Hide Subtitles"
    await expect(page.locator('button:has-text("Hide Subtitles")')).toBeVisible();

    // 4. Toggle back
    await page.locator('button:has-text("Hide Subtitles")').click();
    await expect(overlay).toBeHidden();
    await expect(page.locator('button:has-text("Show Subtitles")')).toBeVisible();

    await context.close();
  });
});
