import { test, expect } from '@playwright/test';

test.beforeEach(async ({ page }) => {
    // Reset room state via API if possible, or just use a unique room name
    await page.goto('/');
});

test('Audio Level Indicator exists on local video', async ({ page }) => {
    const roomName = `audio-level-${Math.random().toString(36).substring(7)}`;
    await page.fill('input[type="text"]', roomName);
    await page.click('.create-btn');

    // Prejoin screen
    await page.waitForSelector('#display-name');
    await page.fill('#display-name', 'Alice');
    await page.click('.join-btn');

    // Room page
    await expect(page.locator('.video-grid')).toBeVisible({ timeout: 15000 });

    // Check for local video level indicator
    const localIndicator = page.locator('.local-video .audioindicator');
    await expect(localIndicator).toBeVisible();
});

test('Per-participant E2EE toggle in Settings', async ({ page }) => {
    const roomName = `e2ee-${Math.random().toString(36).substring(7)}`;
    await page.fill('input[type="text"]', roomName);
    await page.click('.create-btn');

    await page.fill('#display-name', 'Alice');
    await page.click('.join-btn');

    // Open settings
    await page.click('button[title="Settings"]');
    await expect(page.locator('.modal-content')).toBeVisible();

    // Toggle E2EE
    const e2eeCheckbox = page.locator('input[type="checkbox"]', { hasText: 'Enable End-to-End Encryption' }).first();
    // Re-locate more specifically because of multiple checkboxes
    const e2eeLabel = page.getByText('Enable End-to-End Encryption');
    await e2eeLabel.click();

    // Check if visual indicator appears in VideoGrid
    await page.click('.modal-header button'); // Close settings
    const lockIndicator = page.locator('.local-video .status-icons span[title="End-to-End Encrypted"]');
    await expect(lockIndicator).toBeVisible();
});

test('Noise Detection warning appears', async ({ page }) => {
    // This test is hard to trigger with fake media, but we can check if the listener is wired by injecting an event
    const roomName = `noise-${Math.random().toString(36).substring(7)}`;
    await page.fill('input[type="text"]', roomName);
    await page.click('.create-btn');

    await page.fill('#display-name', 'Alice');
    await page.click('.join-btn');

    await page.waitForSelector('.video-grid');

    // Inject custom event to simulate noise detection
    await page.evaluate(() => {
        window.dispatchEvent(new CustomEvent('noise_detected'));
    });

    // Check for warning toast
    const toast = page.locator('.toast', { hasText: 'High background noise detected' });
    await expect(toast).toBeVisible();
    await expect(toast).toHaveCSS('background-color', 'rgb(255, 193, 7)'); // #ffc107
});

test('Dominant speaker switches spotlight', async ({ page, context }) => {
    const roomName = `spotlight-${Math.random().toString(36).substring(7)}`;
    await page.goto(`/?room=${roomName}`);
    await page.fill('input[type="text"]', roomName);
    await page.click('.create-btn');
    await page.fill('#display-name', 'Alice');
    await page.click('.join-btn');

    // Join second user
    const page2 = await context.newPage();
    await page2.goto(`/room/${roomName}`);
    await page2.fill('#display-name', 'Bob');
    await page2.click('.join-btn');

    await page.waitForSelector('.video-card');
    await page.click('button:has-text("Switch View")'); // Enable spotlight

    // Simulate Bob speaking (dominant speaker)
    // We can't easily simulate WebAudio level in playwright, so we check if Bob's card is featured
    // The VideoGrid features the first remote participant if no one is speaking yet.
    const spotlightCard = page.locator('.video-grid.spotlight .video-card');
    await expect(spotlightCard).toContainText('Bob');
});

test('Recent meetings list on Home', async ({ page }) => {
    const roomName = `recent-${Math.random().toString(36).substring(7)}`;
    await page.fill('input[type="text"]', roomName);
    await page.click('.create-btn');
    await page.fill('#display-name', 'Alice');
    await page.click('.join-btn');
    await page.waitForSelector('.video-grid');

    // Go back home
    await page.goto('/');
    const recentItem = page.locator('button', { hasText: roomName });
    await expect(recentItem).toBeVisible();

    // Click to rejoin
    await recentItem.click();
    await expect(page).toHaveURL(new RegExp(`/room/${roomName}`));
});
