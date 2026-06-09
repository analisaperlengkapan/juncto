import { test, expect } from '@playwright/test';

test('comprehensive meeting lifecycle', async ({ page, context }) => {
  test.setTimeout(120000);

  // Join from home page
  await page.goto('/');
  await page.fill('#meeting-name', 'ComprehensiveTest');
  await page.click('.create-btn');
  await expect(page).toHaveURL(/\/room\/ComprehensiveTest/);

  // Prejoin screen
  await page.fill('#display-name', 'Host');
  await page.click('.join-btn');

  // Verify in room
  await expect(page.locator('.room-container')).toBeVisible();

  // Toggle Chat
  await page.click('#toggle-chat-btn');
  await expect(page.locator('#chat-panel')).toBeVisible();
  await page.fill('#chat-input', 'Hello world');
  await page.keyboard.press('Enter');
  await expect(page.locator('#chat-messages')).toContainText('Hello world');

  // Open Polls and create one
  await page.click('#toggle-polls-btn');
  await page.click('button:has-text("Create Poll")');
  await page.fill('#poll-question', 'Do you like Rust?');
  await page.fill('#poll-option-1', 'Yes');
  await page.fill('#poll-option-2', 'Absolutely');
  await page.click('#create-poll-submit-btn');
  await page.click('button:has-text("Active Polls")');
  await expect(page.locator('.poll-item').first()).toContainText('Do you like Rust?');
  await page.click('#close-polls-btn');

  // Integration Check (Mocks)
  await page.click('#settings-btn');
  await page.click('button:has-text("Integrations")');
  await expect(page.locator('text=Dropbox')).toBeVisible();
  await page.click('#close-settings-btn');

  // Second participant joins
  const page2 = await context.newPage();
  await page2.goto(page.url());
  await page2.fill('#display-name', 'Guest');
  await page2.click('.join-btn');

  // Verify Guest joined on their own screen
  await expect(page2.locator('.room-container')).toBeVisible();

  // Verify guest in participants list of host
  await page.click('#toggle-participants-btn');
  await expect(page.locator('.participants-container')).toContainText('Guest');

  // Leave room
  await page.click('button[title="Leave Meeting"]');
  await expect(page).toHaveURL('/');
});
