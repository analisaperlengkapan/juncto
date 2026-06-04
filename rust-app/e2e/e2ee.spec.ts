import { test, expect } from '@playwright/test';

test.describe('E2EE Functional Test', () => {
    test('should exchange E2EE keys and show lock icon', async ({ page, context }) => {
        await page.goto('/room/e2ee-test');
        await page.fill('#display-name', 'Alice');
        await page.click('.join-btn');
        await expect(page.locator('.video-grid')).toBeVisible();

        await page.click('#settings-btn');
        await page.waitForSelector('.modal-content');

        await page.locator('#e2ee-participant-toggle').click({ force: true });
        await page.click('#close-settings-btn');

        // Alice sees her own lock
        const aliceLock = page.locator('.e2ee-lock');
        await expect(aliceLock).toBeAttached({ timeout: 20000 });

        const page2 = await context.newPage();
        await page2.goto('/room/e2ee-test');
        await page2.fill('#display-name', 'Bob');
        await page2.click('.join-btn');

        await page2.click('#settings-btn');
        await page2.locator('#e2ee-participant-toggle').click({ force: true });
        await page2.click('#close-settings-btn');

        // Bob sees two locks (himself and Alice)
        await expect(page2.locator('.e2ee-lock')).toHaveCount(2, { timeout: 20000 });

        // Alice now sees Bob's lock too
        await expect(page.locator('.e2ee-lock')).toHaveCount(2, { timeout: 20000 });
    });
});
