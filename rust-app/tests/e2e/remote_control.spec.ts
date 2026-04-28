import { test, expect } from '@playwright/test';

test('remote control request protocol', async ({ browser }) => {
    const context1 = await browser.newContext();
    const page1 = await context1.newPage();
    await page1.goto('/');
    await page1.fill('#meeting-name', 'test-remote');
    await page1.click('.create-btn');
    await page1.fill('#display-name', 'Requester');
    await page1.click('button:has-text("Join Meeting")');

    const context2 = await browser.newContext();
    const page2 = await context2.newPage();
    await page2.goto(page1.url());
    await page2.fill('#display-name', 'Target');
    await page2.click('button:has-text("Join Meeting")');

    await page1.waitForSelector('.participants-container');
    await page1.click('button:has-text("Participants")');
    await page1.locator('.participant-item').filter({ hasText: 'Target' }).locator('button[title="Request Remote Control"]').dispatchEvent('click');

    await expect(page2.locator('.toast-container')).toContainText('requested remote control', { timeout: 10000 });
});
