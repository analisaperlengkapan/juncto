import { test, expect } from '@playwright/test';

test.describe('Meeting Subject Integration Test', () => {
    test('should display meeting subject on prejoin and in room', async ({ page }) => {
        const roomName = 'subject-test-' + Math.random().toString(36).substring(7);

        // 1. Set subject as a host first
        await page.goto(`/room/${roomName}`);
        await page.fill('#display-name', 'Host');
        await page.click('.join-btn');
        await expect(page.locator('.video-grid')).toBeVisible();

        await page.click('#settings-btn');
        await page.click('text=Moderator');
        await page.fill('#settings-subject', 'Advanced Rust Meeting');
        await page.click('#update-subject-btn');
        await page.click('#close-settings-btn');

        // Verify subject in room header
        await expect(page.locator('#meeting-subject')).toHaveText('Advanced Rust Meeting');

        // 2. New participant joins and sees subject on Prejoin
        const page2 = await page.context().newPage();
        await page2.goto(`/room/${roomName}`);

        // Verify subject on prejoin screen
        await expect(page2.locator('#prejoin-subject')).toHaveText('Advanced Rust Meeting');

        await page2.fill('#display-name', 'Guest');
        await page2.click('.join-btn');

        // Verify subject in room for guest
        await expect(page2.locator('#meeting-subject')).toHaveText('Advanced Rust Meeting');
    });
});
