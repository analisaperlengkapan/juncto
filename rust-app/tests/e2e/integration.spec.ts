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










test.beforeEach(async ({ page }) => {
  await page.goto('/');
});

test('Meeting Flow: Home -> Prejoin -> Room', async ({ page }) => {
  const roomName = 'TestRoom-' + Math.random().toString(36).substring(7);

  // Home Page
  await page.waitForSelector('input[type="text"]');
  await page.fill('input[type="text"]', roomName);
  await page.click('button:has-text("Start Meeting")');

  // Prejoin Screen
  await expect(page).toHaveURL(new RegExp('/room/' + roomName));
  await page.fill('input[placeholder="Enter your name"]', 'Test User');
  const joinBtn = page.locator('button.join-btn');
  await expect(joinBtn).toBeEnabled({ timeout: 15000 });
  await joinBtn.click();

  // Room
  await expect(page.locator('.video-grid')).toBeVisible({ timeout: 15000 });
  await expect(page.locator('.local-video .name-tag', { hasText: 'Me' })).toBeVisible();
});

test('Polls: Create and Vote', async ({ page }) => {
  const roomName = 'PollTest-' + Math.random().toString(36).substring(7);
  await page.goto('/room/' + roomName);
  await page.fill('input[placeholder="Enter your name"]', 'Host');
  const joinBtn = page.locator('button.join-btn');
  await expect(joinBtn).toBeEnabled({ timeout: 15000 });
  await joinBtn.click();

  // Open Polls Dialog
  await page.click('button:has-text("Polls")');
  await expect(page.locator('h3:has-text("Polls")')).toBeVisible();

  // Create Poll
  await page.click('button:has-text("Create Poll")');
  await page.fill('input[placeholder="e.g. What is your favorite color?"]', 'Favorite Color?');
  await page.fill('label:has-text("Option 1") + input', 'Red');
  await page.fill('label:has-text("Option 2") + input', 'Blue');
  await page.click('div.tab-content button:has-text("Create Poll")');

  // Vote
  await expect(page.locator('text=Favorite Color?')).toBeVisible();
  await page.click('button:has-text("Vote") >> nth=0');
  await expect(page.locator('text=1 votes')).toBeVisible();
});

test('Settings: E2EE Toggle', async ({ page }) => {
  const roomName = 'E2EETest-' + Math.random().toString(36).substring(7);
  await page.goto('/room/' + roomName);
  await page.fill('input[placeholder="Enter your name"]', 'Host');
  const joinBtn = page.locator('button.join-btn');
  await expect(joinBtn).toBeEnabled({ timeout: 15000 });
  await joinBtn.click();

  // Open Settings
  await page.click('button:has-text("Settings")');

  // Switch to Moderator tab (since we are host)
  await page.click('button:has-text("Moderator")');

  // Toggle E2EE
  const e2eeToggle = page.locator('text=Enable End-to-End Encryption').locator('..').locator('input[type="checkbox"]');
  await e2eeToggle.check();

  // Verify lock icon appears or indicator - based on room.rs it shows "🔒 E2EE (indicator)"
  await expect(page.locator('text=🔒 E2EE (indicator)')).toBeVisible();
});
