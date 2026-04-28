import { test, expect } from '@playwright/test';

test.beforeEach(async ({ page }) => {
    await page.goto('/');
});

test('face expression detection toggle', async ({ page }) => {
    await page.fill('#meeting-name', 'test-face');
    await page.click('.create-btn');
    await page.fill('#display-name', 'Alice');
    await page.click('button:has-text("Join Meeting")');
    await page.click('button[title="Settings"]');
    await page.click('button:has-text("More")');
    const checkbox = page.locator('input[type="checkbox"]');
    await checkbox.check();
    await page.click('button:has-text("×")');
    await page.click('button[title="Settings"]');
    await page.click('button:has-text("More")');
    expect(await checkbox.isChecked()).toBeTruthy();
});
