import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    try {
        await page.click('button[title="Login"]', { timeout: 2000 });
        await page.fill('input[placeholder="user@domain.com"]', 'admin');
        await page.fill('input[placeholder="Password"]', 'admin123');
        await page.click('button:has-text("Login")');
    } catch (e) {
        // Fallback or already logged in
    }
}


test.beforeEach(async ({ request }) => {
    // Reset room state, mirroring moderation.spec.ts so this test runs
    // against a known-good room config regardless of prior test state.
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

test('remote control request protocol', async ({ browser }) => {
    const roomName = `RemoteControlRoom_${Date.now()}`;

    // 1. Requester joins via the home page. The home page exposes a single
    // text input for the room name and a "Start Meeting" button — see
    // moderation.spec.ts for the same pattern.
    const context1 = await browser.newContext();
    const page1 = await context1.newPage();
    await page1.goto('/');
    await page1.fill('input[type="text"]', roomName);
    await page1.click('button:has-text("Start Meeting")');
    await page1.locator('.prejoin-container input[type="text"]').fill('Requester');
    await page1.click('button:has-text("Join Meeting")');
    await expect(page1.locator('.video-grid')).toBeVisible();

    // 2. Target joins the same room directly.
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();
    await page2.goto(`/room/${roomName}`);
    await page2.locator('.prejoin-container input[type="text"]').fill('Target');
    await page2.click('button:has-text("Join Meeting")');
    await expect(page2.locator('.video-grid')).toBeVisible();

    // 3. Open the participants panel on the requester's page if it isn't
    // already visible, then click the RC button on Target's row.
    if (await page1.locator('.participants-list').isHidden()) {
        await page1.click('.toolbox button:has-text("Participants")');
    }
    const targetRow = page1.locator('.participants-list li').filter({ hasText: 'Target' });
    await expect(targetRow).toBeVisible();
    await targetRow.locator('button[title="Request Remote Control"]').dispatchEvent('click');

    // 4. The PR implements consent as a non-blocking in-app modal (see
    // `rust-app/frontend/src/remote_control.rs`), not a toast. Verify the
    // modal text appears on the target's page.
    await expect(page2.locator('text=Remote Control Request')).toBeVisible({ timeout: 10000 });
    await expect(page2.locator('text=is requesting remote control of your session')).toBeVisible();

    await context1.close();
    await context2.close();
});
