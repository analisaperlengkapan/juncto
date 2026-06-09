import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    const loginBtn = page.locator('button[title="Login"]');
    if (await loginBtn.isVisible()) {
        await loginBtn.click();
        await page.fill('input[placeholder="user@domain.com"]', 'admin');
        await page.fill('input[placeholder="Password"]', 'admin123');
        await page.click('button:has-text("Login")');
        await page.waitForSelector('.toast-container:has-text("Authenticated")', { timeout: 5000 }).catch(() => {});
    }
}










test.describe('Advanced Moderation', () => {
  test('host should be able to mute all participants', async ({ context }) => {
    // 1. Host joins
    const hostPage = await context.newPage();
    // Use a large viewport so the toolbox and participants list are fully visible
    await hostPage.setViewportSize({ width: 1280, height: 1024 });
    await hostPage.goto('/room/mute-all-test');
    await hostPage.fill('input[placeholder="Enter your name"]', 'Host');
    await hostPage.click('button:has-text("Join Meeting")');

    // 2. Guest joins
    const guestPage = await context.newPage();
    await guestPage.setViewportSize({ width: 1280, height: 1024 });
    await guestPage.goto('/room/mute-all-test');
    await guestPage.fill('input[placeholder="Enter your name"]', 'Guest');
    await guestPage.click('button:has-text("Join Meeting")');

    // 3. Host opens participants list and clicks Mute All
    await hostPage.waitForSelector('.participants-list');
    // Wait for the guest to appear in the host's participant list before muting
    await expect(hostPage.locator('.participants-list li').filter({ hasText: 'Guest' })).toBeVisible({ timeout: 15000 });
    const muteAllBtn = hostPage.getByRole('button', { name: 'Mute All', exact: true });
    await expect(muteAllBtn).toBeVisible({ timeout: 10000 });
    await muteAllBtn.scrollIntoViewIfNeeded();
    await muteAllBtn.dispatchEvent('click');

    // 4. Verify guest is muted
    // Guest should see a toast or their own mute indicator
    // In participants list, guest should show 🔇
    const guestEntry = guestPage.locator('.participants-list li').filter({ hasText: 'Guest' });
    await expect(guestEntry.locator('text=🔇')).toBeVisible();

    // Guest should also see a toast if implemented to notify them
    // await expect(guestPage.locator('.toast')).toContainText('muted by the host');
  });
});
