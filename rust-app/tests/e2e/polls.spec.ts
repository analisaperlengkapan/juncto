import { test, expect } from '@playwright/test';

test.describe('Polls', () => {
  test('should create a poll and allow voting', async ({ browser }) => {
    const roomName = `PollRoom_${Date.now()}`;
    const contextA = await browser.newContext();
    const pageA = await contextA.newPage();
    await pageA.goto('/');

    // Host
    await pageA.fill('input[type="text"]', roomName);
    await pageA.click('button:has-text("Start Meeting")');
    await pageA.locator('.prejoin-container input[type="text"]').fill('Alice');
    await pageA.click('button:has-text("Join Meeting")');

    // Guest
    const contextB = await browser.newContext();
    const pageB = await contextB.newPage();
    await pageB.goto(`/room/${roomName}`);
    await pageB.locator('.prejoin-container input[type="text"]').fill('Bob');
    await pageB.click('button:has-text("Join Meeting")');

    // Host opens polls
    await pageA.click('button:has-text("Polls")');
    await pageA.locator('.tabs button', { hasText: 'Create Poll' }).click();

    // Fill form - using nth index as inputs don't have unique IDs/Names in the component
    // 0: Question, 1: Option 1, 2: Option 2
    await pageA.locator('.tab-content input[type="text"]').nth(0).fill('Favorite Color?');
    await pageA.locator('.tab-content input[type="text"]').nth(1).fill('Red');
    await pageA.locator('.tab-content input[type="text"]').nth(2).fill('Blue');

    // Create
    await pageA.locator('.tab-content button', { hasText: 'Create Poll' }).click();

    // Verify on Host
    await expect(pageA.locator('.poll-item')).toContainText('Favorite Color?');

    // Verify on Guest
    await pageB.click('button:has-text("Polls")');
    await expect(pageB.locator('.poll-item')).toContainText('Favorite Color?');

    // Vote
    await pageB.click('button:has-text("Vote") >> nth=0');

    // Verify result
    await expect(pageB.locator('li', { hasText: 'Red' })).toContainText('1 votes');
    await expect(pageA.locator('li', { hasText: 'Red' })).toContainText('1 votes');

    await contextA.close();
    await contextB.close();
  });
});
