import { test, expect } from '@playwright/test';

test.describe('Branding Integration Test', () => {
    test('should apply color and logo branding', async ({ page }) => {
        await page.goto('/room/branding-test');
        await page.fill('#display-name', 'Host');
        await page.click('.join-btn');
        await expect(page.locator('.video-grid')).toBeVisible();

        // Open settings
        await page.click('#settings-btn');
        await page.waitForSelector('.modal-content');

        // Go to Branding tab - use a more precise selector to avoid ambiguity
        await page.click('.tabs button:has-text("Branding")');

        // Set primary color and logo URL
        await page.fill('#branding-primary-color', '#ff0000'); // Red
        await page.fill('#branding-logo-url', 'https://media.giphy.com/media/v1.Y2lkPTc5MGI3NjExNHJid3R6NmE5bzh6am9ueXp6bzh6am9ueXp6bzh6am9ueXp6Yzh6aiZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9cw/3o7TKVUn7iM8FMEU24/giphy.gif');
        await page.click('#save-branding-btn');

        // Use the close button specifically
        await page.click('#close-settings-btn');

        // Verify primary color is applied (via CSS variable)
        const primaryColor = await page.evaluate(() => {
            const root = document.documentElement;
            return getComputedStyle(root).getPropertyValue('--primary-color').trim();
        });
        // Note: Browsers might normalize #ff0000 to rgb(255, 0, 0)
        expect(['#ff0000', 'rgb(255, 0, 0)']).toContain(primaryColor);

        // Verify logo is visible in header
        await expect(page.locator('#room-logo')).toBeVisible({ timeout: 10000 });
        await expect(page.locator('#room-logo')).toHaveAttribute('src', /giphy\.gif/);
    });
});
