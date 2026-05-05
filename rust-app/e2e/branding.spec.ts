import { test, expect } from '@playwright/test';

test.beforeEach(async ({ page }) => {
  // Reset backend state if possible, or just use a unique room
  await page.goto('/');
});

test('Dynamic branding applies colors', async ({ page }) => {
  const roomName = `branding-test-${Math.random().toString(36).substring(7)}`;

  await page.fill('#meeting-name', roomName);
  await page.click('.create-btn');

  await page.fill('#display-name', 'Host');
  await page.click('.join-btn');

  await expect(page.locator('h2')).toContainText(roomName);

  // Open settings
  await page.click('button[title="Settings"]');
  await page.click('button:has-text("Branding")');

  // Change colors
  await page.fill('#branding-primary-color', '#ff0000');
  await page.fill('#branding-bg-color', '#00ff00');
  await page.click('#save-branding-btn');

  // Check if CSS variables are applied
  const primaryColor = await page.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue('--primary-color').trim());
  const bgColor = await page.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue('--background-color').trim());

  expect(primaryColor).toBe('#ff0000');
  expect(bgColor).toBe('#00ff00');
});
