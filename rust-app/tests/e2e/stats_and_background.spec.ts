import { test, expect } from '@playwright/test';

test('Speaker Stats, Virtual Background, and Connection Stats', async ({ page }) => {
  // 1. Join Room
  await page.goto('/');
  await page.click('button:has-text("Start Meeting")');

  // Wait for Prejoin
  await expect(page.locator('h2:has-text("Ready to join?")')).toBeVisible();

  await page.fill('input', 'Tester');
  await page.click('button:has-text("Join Meeting")');

  // Wait for join
  await expect(page.locator('.room-container')).toBeVisible();

  // 2. Check Connection Stats (always visible)
  // It contains "ms" text
  await expect(page.locator('.connection-stats')).toBeVisible();
  await expect(page.locator('.connection-stats')).toContainText('ms');

  // 3. Open Speaker Stats
  await page.click('button:has-text("Stats")');
  await expect(page.locator('h3:has-text("Speaker Stats")')).toBeVisible();
  // Check if my name is in the table
  await expect(page.locator('table')).toContainText('Tester');
  // Close it
  await page.click('button:has-text("×")');
  await expect(page.locator('h3:has-text("Speaker Stats")')).not.toBeVisible();

  // 4. Open Virtual Background
  await page.click('button:has-text("Background")');
  await expect(page.locator('h3:has-text("Virtual Background")')).toBeVisible();
  // Click Blur
  await page.click('div:has-text("Blur")');
  // Click Done
  await page.click('button:has-text("Done")');
  await expect(page.locator('h3:has-text("Virtual Background")')).not.toBeVisible();

  // Take screenshot
  await page.screenshot({ path: 'verification.png' });
});
