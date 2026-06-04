import { test, expect } from '@playwright/test';

test.describe('AV Moderation', () => {
    test('should restrict guest AV when moderation is enabled', async ({ page, context }) => {
        // Host (Alice) joins
        await page.goto('/room/mod-test');
        await page.fill('#display-name', 'Alice');
        await page.click('.join-btn');
        await expect(page.locator('.video-grid')).toBeVisible();

        // Guest (Bob) joins
        const page2 = await context.newPage();
        await page2.goto('/room/mod-test');
        await page2.fill('#display-name', 'Bob');
        await page2.click('.join-btn');
        await expect(page2.locator('.video-grid')).toBeVisible();

        // Bob mutes himself
        await page2.click('#toggle-mic-btn');

        // Host enables audio moderation
        await page.click('#settings-btn');
        await page.click('text=Moderator');
        await page.locator('#audio-moderation-toggle').click({ force: true });
        await page.click('#close-settings-btn');

        // Bob should see the request button now
        await expect(page2.locator('#request-unmute-btn')).toBeVisible({ timeout: 15000 });

        // Bob requests permission
        await page2.click('#request-unmute-btn');

        // Alice sees request and grants it
        await page.click('#toggle-participants-btn');
        const grantBtn = page.locator('.grant-mic-btn');
        await expect(grantBtn).toBeAttached({ timeout: 15000 });
        // Use dispatchEvent to bypass viewport/intersection issues
        await grantBtn.dispatchEvent('click');

        // Bob should now have the request button hidden (reactive update)
        await expect(page2.locator('#request-unmute-btn')).toBeHidden({ timeout: 15000 });

        // Bob should now be able to unmute without getting an error toast
        await page2.click('#toggle-mic-btn');
        await expect(page2.locator('.toast-error')).not.toBeVisible();
    });
});
