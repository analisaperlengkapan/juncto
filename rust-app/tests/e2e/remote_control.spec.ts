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

    // Open the participants panel on the requester's page. The toolbox button
    // toggles the panel; we click it and wait for the participant list to
    // populate before clicking the RC button.
    await page1.click('button:has-text("Participants")');
    await page1.waitForSelector('.participants-list');
    await page1.locator('.participant-item').filter({ hasText: 'Target' }).locator('button[title="Request Remote Control"]').dispatchEvent('click');

    // The PR implements consent as a non-blocking in-app modal (see
    // `rust-app/frontend/src/remote_control.rs`), not a toast. Verify the
    // modal text appears on the target's page.
    await expect(page2.locator('text=Remote Control Request')).toBeVisible({ timeout: 10000 });
    await expect(page2.locator('text=is requesting remote control of your session')).toBeVisible();
});
