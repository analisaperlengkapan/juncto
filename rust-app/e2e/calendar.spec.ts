import { test, expect } from '@playwright/test';

test.describe('Calendar Integration Test', () => {
    test('should fetch and display calendar events', async ({ page }) => {
        await page.goto('/room/calendar-test');
        await page.fill('#display-name', 'Alice');
        await page.click('.join-btn');
        await expect(page.locator('.video-grid')).toBeVisible();

        // Open calendar dialog
        await page.click('button[title="Calendar"]');

        // Verify calendar dialog is visible
        await expect(page.locator('.calendar-list-dialog')).toBeVisible();
        await expect(page.locator('h3:has-text("Upcoming Meetings")')).toBeVisible();

        // Verify mock events are displayed
        // Our mock returns "Team Standup", "Project Sync", "1:1 with Manager"
        await expect(page.locator('li:has-text("Team Standup")')).toBeVisible();
        await expect(page.locator('li:has-text("Project Sync")')).toBeVisible();

        // Refresh and check again
        await page.click('button:has-text("Refresh")');
        await expect(page.locator('li:has-text("Release Planning")')).toBeVisible();

        // Close dialog
        await page.click('.calendar-list-dialog button:has-text("×")');
        await expect(page.locator('.calendar-list-dialog')).not.toBeVisible();
    });
});
