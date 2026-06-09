import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    await page.click('button[title="Login"]');
    await page.fill('input[placeholder="Username"]', 'admin');
    await page.fill('input[placeholder="Password"]', 'admin123');
    await page.click('button:has-text("Login")');
}


test.describe('Reactions', () => {
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

  test('should display reactions across peers', async ({ browser }) => {
    const roomName = `ReactionsRoom_${Date.now()}`;
    const contextA = await browser.newContext();
    const pageA = await contextA.newPage();
    const contextB = await browser.newContext();
    const pageB = await contextB.newPage();

    // User A joins
    await pageA.goto(`/room/${roomName}`);
    await pageA.locator('.prejoin-container input[type="text"]').fill('Alice');
    await pageA.click('button:has-text("Join Meeting")');

    // User B joins
    await pageB.goto(`/room/${roomName}`);
    await pageB.locator('.prejoin-container input[type="text"]').fill('Bob');
    await pageB.click('button:has-text("Join Meeting")');

    // User A sends reaction
    // Give some time for the components to load
    await pageA.waitForSelector('button:has-text("👍")');
    await pageB.waitForSelector('button:has-text("👍")');

    await pageA.click('button:has-text("👍")');

    // User B should see reaction
    await expect(pageB.locator('.reaction-layer div:has-text("👍")')).toBeVisible();

    await contextA.close();
    await contextB.close();
  });
});
