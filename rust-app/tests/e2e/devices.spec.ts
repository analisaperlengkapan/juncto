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










test.describe('Device Selection', () => {
    test('User can open settings and select devices', async ({ browser }) => {
        const context = await browser.newContext();
        // Grant permissions to allow enumerateDevices to return labels
        await context.grantPermissions(['camera', 'microphone']);

        const page = await context.newPage();
        await page.goto('/');

        // Join room
        await page.fill('input[type="text"]', 'DeviceTester');
        await page.click('button:has-text("Start Meeting")');
        // Join immediately (Prejoin screen)
        await page.locator('.prejoin-container input[type="text"]').fill('DeviceUser');
        await page.click('button:has-text("Join Meeting")');

        // Open Settings
        await page.click('button:has-text("Settings")');

        // Verify Modal
        await expect(page.locator('h3:has-text("Settings")')).toBeVisible();

        // Switch to Devices tab
        await page.click('button:has-text("Devices")');

        // Verify Device Selects exist
        await expect(page.locator('label:has-text("Camera")')).toBeVisible();
        await expect(page.locator('label:has-text("Microphone")')).toBeVisible();

        // Check if "Apply Devices" button exists
        await expect(page.locator('button:has-text("Apply Devices")')).toBeVisible();

        // Click Apply (mocking actual device selection is hard without virtual devices, but clicking Apply ensures callback is wired)
        await page.click('button:has-text("Apply Devices")');

        // Modal should close
        await expect(page.locator('h3:has-text("Settings")')).not.toBeVisible();
    });
});
