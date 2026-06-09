import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    await page.click('button[title="Login"]');
    await page.fill('input[placeholder="Username"]', 'admin');
    await page.fill('input[placeholder="Password"]', 'admin123');
    await page.click('button:has-text("Login")');
}


test.describe('Breakout Rooms', () => {
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

  test('should create and join breakout room', async ({ browser }) => {
    const roomName = `BreakoutRoom_${Date.now()}`;
    const contextA = await browser.newContext();
    const pageA = await contextA.newPage();
    await pageA.goto('/');

    // Host
    await pageA.fill('input[type="text"]', roomName);
    await pageA.click('button:has-text("Start Meeting")');
    await pageA.locator('.prejoin-container input[type="text"]').fill('Alice');
    await pageA.click('button:has-text("Join Meeting")');

    // Wait for component to load
    await expect(pageA.locator('h4:has-text("Breakout Rooms")')).toBeVisible({ timeout: 15000 });

    await pageA.fill('input[placeholder="New Room Name"]', 'Room A');
    await pageA.click('button:has-text("Create")'); // Inside breakout component

    // Verify created
    await expect(pageA.locator('.rooms-list')).toContainText('Room A');

    // Join Room
    await pageA.click('.rooms-list button:has-text("Join")');

    // Verify joined (Return to Main button appears)
    await expect(pageA.locator('button:has-text("Return to Main")')).toBeVisible();

    // Return
    await pageA.click('button:has-text("Return to Main")');
    await expect(pageA.locator('button:has-text("Return to Main")')).not.toBeVisible();

    await contextA.close();
  });
});
