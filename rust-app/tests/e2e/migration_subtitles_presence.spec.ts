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


test.describe('Subtitles and Presence Status Features', () => {

  test('Subtitles toggle and Presence Display', async ({ browser, request }) => {
    const roomName = `SubRoom_${Date.now()}`;

    // Reset room config via API
    await request.post('/api/rooms', {
        data: {
            room_name: roomName,
            is_locked: false,
            is_recording: false,
            is_lobby_enabled: false,
            max_participants: 100
        }
    });

    const context = await browser.newContext();
    const page = await context.newPage();

    // 1. Join room
    await page.goto(`/room/${roomName}`);

    // Prejoin screen
    await page.waitForSelector('#display-name');
    await page.fill('#display-name', 'SubtitleTestUser');
    await page.click('.join-btn');

    // Wait for the room to load
    await page.waitForSelector('.video-grid', { timeout: 15000 });

    // 2. Verify Presence Status "Connected" is displayed next to name
    // Open participants panel if hidden
    if (await page.locator('#participant-search').isHidden()) {
        await page.click('#toggle-participants-btn');
    }
    const participantLocator = page.locator('.participant-item', { hasText: 'SubtitleTestUser' });
    await expect(participantLocator).toContainText('[Connected]');

    // 3. Toggle Subtitles
    // Standardized ID is #toggle-subtitles-btn
    const ccBtn = page.locator('#toggle-subtitles-btn');

    const overlay = page.locator('.subtitles-overlay');
    const isInitiallyEnabled = await overlay.isVisible();

    await ccBtn.click();

    if (isInitiallyEnabled) {
        await expect(overlay).toBeHidden();
    } else {
        await expect(overlay).toBeVisible();
    }

    // Toggle back
    await ccBtn.click();
    if (isInitiallyEnabled) {
        await expect(overlay).toBeVisible();
    } else {
        await expect(overlay).toBeHidden();
    }

    await context.close();
  });
});
