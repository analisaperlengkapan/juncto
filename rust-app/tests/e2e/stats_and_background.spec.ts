import { test, expect } from '@playwright/test';

test('Speaker Stats, Virtual Background, and Connection Stats', async ({ page }) => {
  // 1. Join Room
  await page.goto('/');
  await page.fill('#meeting-name', 'StatsRoom');
  await page.click('.create-btn');

  // Wait for Prejoin
  await page.waitForSelector('#display-name');
  await page.fill('#display-name', 'Tester');
  await page.click('.join-btn');

  // Wait for join
  await page.waitForSelector('.room-container');

  // 2. Check Connection Stats (always visible)
  await expect(page.locator('.connection-stats')).toBeVisible();
  await expect(page.locator('.connection-stats')).toContainText('ms');

  // 3. Open Speaker Stats
  await page.click('button:has-text("Stats")');
  await expect(page.locator('h3:has-text("Speaker Stats")')).toBeVisible();
  // Check if my name is in the table
  await expect(page.locator('table')).toContainText('Tester');
  // Close it using more specific locator to avoid multi-match or interception
  await page.locator('#close-speaker-stats-btn').dispatchEvent('click');
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
