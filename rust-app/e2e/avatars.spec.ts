import { test, expect } from '@playwright/test';

test.describe('Participant Avatars', () => {
    test.beforeEach(async ({ page }) => {
        await page.request.post('/api/rooms', {
            data: { room_name: 'AvatarRoom' }
        });
    });

    test('user can join with avatar and see it in participants list', async ({ page }) => {
        const avatarUrl = 'https://www.gravatar.com/avatar/00000000000000000000000000000000?d=mp&f=y';

        await page.goto('/');
        await page.fill('#meeting-name', 'AvatarRoom');
        await page.click('.create-btn');

        await page.waitForSelector('#display-name', { timeout: 30000 });
        await page.fill('#display-name', 'Alice');
        await page.fill('#avatar-url', avatarUrl);

        const joinBtn = page.locator('.join-btn');
        await expect(joinBtn).toBeEnabled({ timeout: 30000 });
        await joinBtn.click();

        await page.waitForSelector('.room-container', { timeout: 30000 });

        await page.click('button:has-text("Participants")');

        const avatarImg = page.locator('.participant-item img[alt="Avatar"]');
        // Wait for image to load and be visible
        await expect(avatarImg).toBeVisible({ timeout: 20000 });
        await expect(avatarImg).toHaveAttribute('src', avatarUrl);

        const gridAvatar = page.locator('.video-card .avatar-container img[alt="Avatar"]');
        await expect(gridAvatar).toBeVisible({ timeout: 10000 });
    });
});
