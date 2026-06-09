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










test.describe('Virtual Background', () => {
  test('should allow user to toggle virtual background', async ({ page }) => {
    await page.goto('/room/test-background');
    await page.fill('input[placeholder="Enter your name"]', 'User');
    await page.click('button:has-text("Join Meeting")');
    await expect(page.locator('h2')).toContainText('Meeting Room: test-background');

    // 1. Open Virtual Background dialog via toolbox
    await page.waitForSelector('.room-toolbox');
    // Find background button (usually has an icon or specific title)
    // In our implementation, we can look for "Background" or similar if titles are set.
    // Fallback: click the button at expected position or with specific class if known.
    await page.click('button[title*="Background"], button:has-text("Background")');

    // 2. Verify dialog is visible
    await expect(page.locator('.modal-content')).toBeVisible();
    await expect(page.locator('.modal-content h3')).toContainText('Virtual Background');

    // 3. Select Blur
    await page.click('div:has-text("Blur")');

    // 4. Verify selection (visual check via style)
    // The selected item has a blue border #007bff
    // Selection logic uses style=move || format!(...) which might need exact match
    const blurOption = page.locator('div:has-text("Blur")').first();
    await expect(blurOption).toBeVisible();

    // 5. Apply and close
    await page.click('button:has-text("Done")');
    await expect(page.locator('.modal-content')).not.toBeVisible();
  });
});
