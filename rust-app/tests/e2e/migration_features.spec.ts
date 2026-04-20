import { test, expect } from '@playwright/test';

test.describe('Migration Features', () => {
    test('Authentication and Calendar Dialog', async ({ page }) => {
        await page.goto('/room/migration-test-room');

        // Wait for prejoin screen
        await page.waitForSelector('.prejoin-container', { timeout: 10000 });

        // Enter a name
        await page.locator('.prejoin-container input[type="text"]').fill('Migration Tester');

        // Join
        await page.click('button.join-btn');

        // Wait for the room to load
        await page.waitForSelector('.video-grid', { timeout: 15000 });

        // Ensure the toolbox is visible
        await page.waitForSelector('.toolbox', { timeout: 15000 });

        // Test Authentication Flow
        // Open Authentication dialog by clicking Login button in the toolbox
        await page.click('.toolbox button:has-text("Login")', { timeout: 15000 });
        await expect(page.locator('.login-dialog-overlay')).toBeVisible();

        // Fill in the form
        await page.fill('input[placeholder="user@domain.com"]', 'admin');
        await page.fill('input[placeholder="Password"]', 'admin123');
        await page.click('.login-dialog button:has-text("Login")');

        // Note: The mock backend handles authentication but we might need a small delay or explicit wait for the toast
        // to verify success depending on WS latency. We can simply close the dialog if the state transition didn't hide it instantly due to test speed
        await expect(page.locator('.login-dialog-overlay')).not.toBeVisible({ timeout: 15000 });

        // Test Calendar Flow
        // Open Calendar dialog by clicking Calendar button in the toolbox
        await page.click('.toolbox button:has-text("Calendar")');
        await expect(page.locator('.calendar-list-overlay')).toBeVisible();

        // Wait for mock events to load and render
        // Since `create_effect` fires on mount to trigger the WS send, it might take a frame to render.
        // Also it might say "No upcoming events found." initially. Let's wait for the list item OR click refresh
        await page.click('.calendar-list-dialog button:has-text("Refresh")');
        await expect(page.locator('.calendar-list-dialog li').first()).toContainText('Team Standup - 10:00 AM', { timeout: 15000 });

        // Close the dialog
        await page.click('.calendar-list-dialog button:has-text("×")');
        await expect(page.locator('.calendar-list-overlay')).not.toBeVisible();
    });
});
