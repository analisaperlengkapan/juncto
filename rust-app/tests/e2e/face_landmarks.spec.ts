import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    await page.click('button[title="Login"]');
    await page.fill('input[placeholder="Username"]', 'admin');
    await page.fill('input[placeholder="Password"]', 'admin123');
    await page.click('button:has-text("Login")');
}


test.beforeEach(async ({ request }) => {
    // Reset room state, mirroring moderation.spec.ts.
    const response = await request.post('http://localhost:3000/api/rooms', {
        data: {
            room_name: 'Test Room',
            is_locked: false,
            is_recording: false,
            is_lobby_enabled: false,
            max_participants: 100,
            host_id: null,
            e2ee_enabled: false,
        },
    });
    expect(response.ok()).toBeTruthy();
});

test('face expression detection toggle', async ({ page }) => {
    const roomName = `FaceLandmarksRoom_${Date.now()}`;

    // Join via the home page using the same selectors as moderation.spec.ts.
    await page.goto('/');
    await page.fill('input[type="text"]', roomName);
    await page.click('button:has-text("Start Meeting")');
    await page.locator('.prejoin-container input[type="text"]').fill('Alice');
    await page.click('button:has-text("Join Meeting")');
    await expect(page.locator('.video-grid')).toBeVisible();

    // Open Settings → More tab and toggle face landmarks on.
    await page.click('.toolbox button:has-text("Settings")');
    await expect(page.locator('.modal-content')).toBeVisible();
    await page.locator('.modal-content .tabs button:has-text("More")').click();
    const checkbox = page.locator('.modal-content input[type="checkbox"]').first();
    await checkbox.check();
    expect(await checkbox.isChecked()).toBeTruthy();

    // Close and reopen Settings → More to verify the toggle persists across
    // settings dialog open/close cycles.
    await page.click('.modal-content button:has-text("×")');
    await page.click('.toolbox button:has-text("Settings")');
    await page.locator('.modal-content .tabs button:has-text("More")').click();
    expect(await checkbox.isChecked()).toBeTruthy();
});
