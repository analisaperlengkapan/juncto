import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    await page.click('button[title="Login"]');
    await page.fill('input[placeholder="Username"]', 'admin');
    await page.fill('input[placeholder="Password"]', 'admin123');
    await page.click('button:has-text("Login")');
}


test.describe('Internationalization & Settings', () => {
  test('should display translated settings and quality options', async ({ page, browser }) => {
    // 1. Join a room
    await page.goto('/');
    const roomName = `i18n-test-${Math.floor(Math.random() * 1000)}`;
    // Use generic text input selector as placeholder might differ or be absent
    await page.locator('input[type="text"]').fill(roomName);
    // Button might be "Start Meeting" or "Join Room" depending on state/text
    // Home page usually has "Start Meeting"
    await page.click('button:has-text("Start Meeting")');

    // Wait for redirect
    await page.waitForURL(/\/room\//);

    // Prejoin screen - Join as 'User'
    // Ensure we target the prejoin input specifically to avoid ambiguity
    await page.locator('.prejoin-container input[type="text"]').fill('User');
    await page.click('button:has-text("Join Meeting")');

    // 2. Open Settings
    await page.click('button:has-text("Settings")');

    // 3. Verify Profile Tab Translations (English Default)
    // Keys: "display_name", "save_profile"
    // Since our mock I18n returns values like "Display Name" for En, we check for that.
    await expect(page.locator('label:has-text("Display Name")')).toBeVisible();
    await expect(page.locator('button:has-text("Save Profile")')).toBeVisible();

    // 4. Switch to Devices Tab
    await page.click('button:has-text("Devices")');

    // 5. Verify Video Quality Option
    await expect(page.locator('label:has-text("Video Quality")')).toBeVisible();
    const qualitySelect = page.locator('select').nth(1); // Assuming 2nd select is quality (Camera, Quality, Mic) - Wait, let's check order in settings.rs
    // Order: Camera, Quality, Mic. So it is the 2nd one (index 1).

    // Or select by label proximity
    const qualityLabel = page.locator('label:has-text("Video Quality")');
    // We can just verify the select exists near it, or check the options.

    // Check if options exist
    const options = await page.locator('select option').allTextContents();
    expect(options).toEqual(expect.arrayContaining(['HD (720p)', 'SD (360p)']));

    // 6. Verify "Camera" and "Microphone" labels
    await expect(page.locator('label:has-text("Camera")')).toBeVisible();
    await expect(page.locator('label:has-text("Microphone")')).toBeVisible();
  });
});
