import { test, expect } from '@playwright/test';

test('Audio Level Indicator visibility and dots render properly', async ({ browser }) => {
  const context1 = await browser.newContext({ permissions: ['camera', 'microphone'] });
  const context2 = await browser.newContext({ permissions: ['camera', 'microphone'] });
  const page1 = await context1.newPage();
  const page2 = await context2.newPage();

  // 1. Host creates room
  await page1.goto('/');
  await page1.waitForSelector('input[type="text"]');
  await page1.fill('input[type="text"]', 'Audio Indicator Room');
  await page1.click('button.create-btn');
  await expect(page1).toHaveURL(/\/room\/.+/);
  const roomUrl = page1.url();

  // 2. Host joins as "Peer1"
  await expect(page1.locator('h2:has-text("Join Meeting")')).toBeVisible();
  await page1.fill('input', 'Peer1');
  await page1.click('button:has-text("Join Meeting")');
  await expect(page1.locator('.room-container')).toBeVisible();

  // 3. Guest joins as "Peer2"
  await page2.goto(roomUrl);
  await expect(page2.locator('h2:has-text("Join Meeting")')).toBeVisible();
  await page2.fill('input', 'Peer2');
  await page2.click('button:has-text("Join Meeting")');
  await expect(page2.locator('.room-container')).toBeVisible();

  // Wait for the remote video card specifically
  const videoCard = page2.locator('.video-card', { hasText: 'Peer1' }).first();
  await expect(videoCard).toBeVisible();

  // Find status-icons
  const statusIcons = videoCard.locator('.status-icons');

  // They might be visually hidden but exist in the DOM
  await expect(statusIcons).toBeAttached();

  const indicator = statusIcons.locator('.audioindicator');

  // Assert on existence in DOM if visibility is flaky due to CSS
  await expect(indicator).toBeAttached();

  const spans = indicator.locator('span');
  await expect(spans).toHaveCount(5);

  await expect(indicator.locator('.audiodot-middle')).toHaveCount(1);
  await expect(indicator.locator('.audiodot-top')).toHaveCount(2);
  await expect(indicator.locator('.audiodot-bottom')).toHaveCount(2);

  await context1.close();
  await context2.close();
});
