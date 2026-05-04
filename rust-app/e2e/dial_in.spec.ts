import { test, expect } from '@playwright/test';

test.describe('Dial-in Information', () => {
    test('user can view dial-in information from toolbox', async ({ page }) => {
        await page.goto('/');
        await page.fill('#meeting-name', 'DialRoom');
        await page.click('.create-btn');

        await page.waitForSelector('#display-name', { timeout: 30000 });
        await page.fill('#display-name', 'Bob');

        const joinBtn = page.locator('.join-btn');
        await expect(joinBtn).toBeEnabled({ timeout: 30000 });
        await joinBtn.click();

        await page.waitForSelector('.room-container', { timeout: 30000 });

        const dialBtn = page.locator('button[title="Dial-in Info"]');
        await expect(dialBtn).toBeVisible({ timeout: 15000 });
        await dialBtn.click();

        await expect(page.locator('h3:has-text("Dial-in Information")')).toBeVisible({ timeout: 10000 });
        await expect(page.locator('text=+1 555 012 3456')).toBeVisible();
        await expect(page.locator('text=123 456 789')).toBeVisible();

        await page.click('button:has-text("Close")');
        await expect(page.locator('h3:has-text("Dial-in Information")')).not.toBeVisible();
    });
});
