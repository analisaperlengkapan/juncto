import { test, expect } from '@playwright/test';

test.beforeEach(async ({ page }) => {
    // Navigate home and ensure we are starting fresh
    await page.goto('/');
});

test('Audio-only mode, participant search, and pinning', async ({ page, context }) => {
    const roomName = `mig-features-${Math.random().toString(36).substring(7)}`;

    // Join as Participant 1 (Alice)
    await page.fill('input[type="text"]', roomName);
    await page.click('.create-btn');
    await page.waitForSelector('#display-name', { timeout: 30000 });
    await page.fill('#display-name', 'Alice');
    await page.click('.join-btn');

    // Verify Alice joined
    await expect(page.locator('h2')).toContainText(`Meeting Room: ${roomName}`, { timeout: 30000 });

    // Join as Participant 2 (Bob) in another page
    const page2 = await context.newPage();
    await page2.goto(`/room/${roomName}`);
    await page2.waitForSelector('#display-name', { timeout: 30000 });
    await page2.fill('#display-name', 'Bob');
    await page2.click('.join-btn');
    await expect(page2.locator('h2')).toContainText(`Meeting Room: ${roomName}`, { timeout: 30000 });

    // Test Search in Participants List (on Alice's page)
    // First, open the panel
    await page.click('#toggle-participants-btn');
    const searchInput = page.locator('#participant-search');
    await expect(searchInput).toBeVisible();

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
    await page.click('button[title="Settings"]');
    await page.click('button:has-text("More")');
    const audioOnlyToggle = page.locator('#audio-only-toggle');
    await audioOnlyToggle.click();
    await page.click('.modal-header button'); // Close settings

    // In Audio-Only mode, remote videos should be hidden (display: none)
    // Bob's video card should be hidden
    await expect(page.locator('.video-card', { hasText: 'Bob' })).not.toBeVisible();

    // Test Pinning
    // Re-open participants panel if closed (it should be open)
    const pinBtn = page.locator('.participant-item', { hasText: 'Bob' }).locator('button[title="Pin participant"]');
    await pinBtn.dispatchEvent('click');

    // Switch to spotlight to see pinning effect
    await page.click('button:has-text("Switch to Spotlight")');

    // Bob should be the spotlighted card even if Alice is host and no one is speaking
    const spotlightCard = page.locator('.video-grid.spotlight .video-card.spotlighted');
    await expect(spotlightCard).toContainText('Bob');

    // Verify pinned indicator (📍)
    await expect(spotlightCard.locator('span[title="Pinned"]')).toBeVisible();
});
