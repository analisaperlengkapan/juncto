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


test.beforeEach(async ({ page }) => {
    // Navigate home and ensure we are starting fresh
    await page.goto('/');
});

test('Audio-only mode, participant search, and pinning', async ({ page, context }) => {
    const roomName = `mig-features-${Math.random().toString(36).substring(7)}`;

    // Join as Participant 1 (Alice)
    await page.fill('#meeting-name', roomName);
    await page.click('.create-btn');
    await page.waitForSelector('#display-name', { timeout: 30000 });
    await page.fill('#display-name', 'Alice');
    await page.click('.join-btn');

    // Verify Alice joined
    await expect(page.locator('.room-container')).toBeVisible({ timeout: 30000 });

    // Join as Participant 2 (Bob) in another page
    const page2 = await context.newPage();
    await page2.goto(`/room/${roomName}`);
    await page2.waitForSelector('#display-name', { timeout: 30000 });
    await page2.fill('#display-name', 'Bob');
    await page2.click('.join-btn');
    await expect(page2.locator('.room-container')).toBeVisible({ timeout: 30000 });

    // Test Search in Participants List (on Alice's page)
    const searchInput = page.locator('#participant-search');

    // In our implementation, panels are open by default.
    // If not visible for some reason, click to open.
    if (!await searchInput.isVisible()) {
        await page.click('#toggle-participants-btn');
    }
    await expect(searchInput).toBeVisible({ timeout: 10000 });

    await searchInput.fill('Bob');
    await expect(page.locator('.participant-item')).toHaveCount(1);
    await expect(page.locator('.participant-item')).toContainText('Bob');

    await searchInput.fill('NonExistentUserXYZ');
    await expect(page.locator('.participant-item')).toHaveCount(0);

    await searchInput.fill('');
    // Should see Alice and Bob
    await expect(page.locator('.participant-item')).toHaveCount(2);

    // Test Audio-Only mode toggle
    // Open settings
    await page.click('#settings-btn');
    await page.click('button:has-text("More")');
    const audioOnlyToggle = page.locator('#audio-only-toggle');
    await audioOnlyToggle.click();
    await page.click('#close-settings-btn');

    // In Audio-Only mode, remote videos should be replaced by avatars
    // Check Bob's video element is hidden (the video card itself stays but video tag is gone/hidden)
    await expect(page.locator('.video-card', { hasText: 'Bob' }).locator('video')).not.toBeVisible();

    // Test Pinning
    // Ensure participants panel is open
    if (!await searchInput.isVisible()) {
        await page.click('#toggle-participants-btn');
    }
    const pinBtn = page.locator('.participant-item', { hasText: 'Bob' }).locator('button[title="Pin participant"]');
    // Use dispatchEvent to avoid overlay issues if any
    await pinBtn.dispatchEvent('click');

    // Switch to spotlight to see pinning effect
    await page.click('button:has-text("Switch to Spotlight")');

    // Bob should be the spotlighted card
    const spotlightCard = page.locator('.video-grid.spotlight .video-card.spotlighted');
    await expect(spotlightCard).toContainText('Bob');

    // Verify pinned indicator (📍)
    await expect(spotlightCard.locator('span[title="Pinned"]')).toBeVisible();
});
