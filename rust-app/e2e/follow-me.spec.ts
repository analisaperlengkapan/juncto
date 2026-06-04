import { test, expect } from '@playwright/test';

test.describe('Follow Me Layout Sync', () => {
    test('should sync layout from host to guest', async ({ page, context }) => {
        // Host (Alice) joins
        await page.goto('/room/follow-me-test');
        await page.fill('#display-name', 'Alice');
        await page.click('.join-btn');
        await expect(page.locator('.video-grid')).toBeVisible();

        // Guest (Bob) joins
        const page2 = await context.newPage();
        await page2.goto('/room/follow-me-test');
        await page2.fill('#display-name', 'Bob');
        await page2.click('.join-btn');
        await expect(page2.locator('.video-grid')).toBeVisible();

        // Default layout is grid
        await expect(page2.locator('.video-grid')).toHaveClass(/grid/);

        // Host switches to spotlight
        await page.click('text=Switch to Spotlight');

        // Guest should follow
        await expect(page2.locator('.video-grid')).toHaveClass(/spotlight/, { timeout: 10000 });

        // Host switches back to grid
        await page.click('text=Switch to Grid');
        await expect(page2.locator('.video-grid')).toHaveClass(/grid/, { timeout: 10000 });
    });
});
