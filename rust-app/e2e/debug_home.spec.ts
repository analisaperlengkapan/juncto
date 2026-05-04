import { test, expect } from '@playwright/test';

test('debug home page', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForTimeout(5000);
    const content = await page.content();
    console.log('Page content:', content);
    const meetingInput = page.locator('#meeting-name');
    console.log('Meeting input exists:', await meetingInput.count() > 0);
    await page.screenshot({ path: 'home-debug.png' });
});
