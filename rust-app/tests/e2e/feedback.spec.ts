import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    await page.click('button[title="Login"]');
    await page.fill('input[placeholder="Username"]', 'admin');
    await page.fill('input[placeholder="Password"]', 'admin123');
    await page.click('button:has-text("Login")');
}


test.describe('Feedback Feature', () => {
    test.beforeEach(async ({ page, request }) => {
        // Reset backend state
        await request.post('/api/rooms', {
            data: {
                room_name: 'Feedback Test Room',
                is_locked: false,
                is_recording: false,
                is_lobby_enabled: false,
                max_participants: 100,
                e2ee_enabled: false,
            }
        });

        // Go to room
        await page.goto('/room/feedback-test');

        // Wait for prejoin
        await page.waitForSelector('.prejoin-container');

        // Enter name
        await page.fill('input[type="text"]', 'FeedbackTester');

        // Join
        await page.click('button:has-text("Join Meeting")');

        // Wait for room
        await page.waitForSelector('.room-container');
    });

    test('should allow submitting feedback', async ({ page }) => {
        // 1. Open Feedback Dialog (Find button in toolbox)
        await page.click('button:has-text("Feedback")');

        // Wait for modal
        await expect(page.locator('.modal-content h3')).toContainText('Feedback');

        // 2. Select 5 stars
        // The stars are spans with text "★". We select the 5th one (index 4).
        const stars = page.locator('.modal-content span:has-text("★")');
        await expect(stars).toHaveCount(5);
        await stars.nth(4).click();

        // 3. Enter comment
        await page.fill('textarea', 'This is an amazing app!');

        // 4. Submit
        await page.click('button:has-text("Submit")');

        // 5. Verify success toast
        // We look for .toast with the success message
        const toast = page.locator('.toast').filter({ hasText: 'Feedback Submitted!' });
        await expect(toast).toBeVisible({ timeout: 5000 });

        // 6. Dialog should close
        await expect(page.locator('.modal-content h3:has-text("Feedback")')).not.toBeVisible();
    });
});
