import { test, expect } from '@playwright/test';

test.describe('Polls', () => {
  test.beforeEach(async ({ request }) => {
    // Reset room state
    const response = await request.post('/api/rooms', {
        data: {
            room_name: 'Test Room',
            is_locked: false,
            is_recording: false,
            is_lobby_enabled: false,
            max_participants: 100,
            host_id: null,
            e2ee_enabled: false
        }
    });
    expect(response.ok()).toBeTruthy();
    expect(response.status()).toBe(201);
  });

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

    // Host creates poll BEFORE Guest joins
    await pageA.click('button:has-text("Polls")');
    await pageA.locator('.tabs button', { hasText: 'Create Poll' }).click();
    await pageA.locator('.tab-content input[type="text"]').nth(0).fill('Early Poll?');
    await pageA.locator('.tab-content input[type="text"]').nth(1).fill('Yes');
    await pageA.locator('.tab-content input[type="text"]').nth(2).fill('No');
    await pageA.locator('.tab-content button', { hasText: 'Create Poll' }).click();

    // Guest joins AFTER poll creation
    const contextB = await browser.newContext();
    const pageB = await contextB.newPage();
    await pageB.goto(`/room/${roomName}`);
    await pageB.locator('.prejoin-container input[type="text"]').fill('Bob');
    await pageB.click('button:has-text("Join Meeting")');

    // Verify Guest sees the existing poll
    await pageB.click('button:has-text("Polls")');
    await expect(pageB.locator('.poll-item')).toContainText('Early Poll?');

    // Vote
    await pageB.click('button:has-text("Vote") >> nth=0');

    // Verify result
    await expect(pageB.locator('li', { hasText: 'Yes' })).toContainText('1 votes');
    await expect(pageA.locator('li', { hasText: 'Yes' })).toContainText('1 votes');

    await contextA.close();
    await contextB.close();
  });
});
