import { test, expect } from '@playwright/test';

test.describe('Moderation Controls', () => {
  test.beforeEach(async ({ request }) => {
    // Reset room state
    const response = await request.post('http://localhost:3000/api/rooms', {
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

  test('Host can mute a participant', async ({ browser }) => {
    const roomName = `ModerationRoom1_${Date.now()}`;
    const contextA = await browser.newContext(); // Host
    const pageA = await contextA.newPage();
    const contextB = await browser.newContext(); // Participant
    const pageB = await contextB.newPage();

    // 1. Host joins
    await pageA.goto('/');
    await pageA.fill('input[type="text"]', roomName);
    await pageA.click('button:has-text("Start Meeting")');
    await pageA.locator('.prejoin-container input[type="text"]').fill('Host User');
    await pageA.click('button:has-text("Join Meeting")');
    await expect(pageA.locator('.video-grid')).toBeVisible();

    // 2. Participant joins (Unmuted)
    await pageB.goto(`/room/${roomName}`);
    await pageB.locator('.prejoin-container input[type="text"]').fill('User B');
    // Ensure mic is enabled (default is usually off in some tests if not careful, but let's assume default UI is ON or we click it)
    // Check toggle button state. "Mute" means currently unmuted (to mute). "Unmute" means currently muted.
    // If button says "Unmute", click it to unmute.
    // However, default Prejoin might have it off. Let's check logic.
    // Prejoin usually has toggles.
    // Let's assume default is unmuted for test simplicity, or click toggle if needed.
    // We can check "Mute" button presence in prejoin to confirm state.
    // Or just look at room state after join.
    await pageB.click('button:has-text("Join Meeting")');
    await expect(pageB.locator('.video-grid')).toBeVisible();

    // Ensure User B is unmuted. Host sees "Mute" button for User B.
    // If User B joined muted, Host would see NO "Mute" button (because of `Show when=!is_muted`).
    // So if "Mute" button is missing, B is muted.
    // Let's wait for participant list to populate.
    if (await pageA.locator('.participants-list').isHidden()) { await pageA.click('#toggle-participants-btn'); }
    const userBRow = pageA.locator('.participants-list li').filter({ hasText: 'User B' });
    await expect(userBRow).toBeVisible();

    // Check if "Mute" button exists. If not, maybe B joined muted.
    // If so, have B unmute first.
    // On Page B:
    if (await pageB.locator('button:has-text("Unmute")').isVisible()) {
        await pageB.click('button:has-text("Unmute")');
    }
    // Now B should be unmuted. Host should see "Mute" button.
    const muteBtn = userBRow.getByRole('button', { name: 'Mute', exact: true });
    await expect(muteBtn).toBeVisible();

    // 3. Host mutes User B. Scroll the button into view first so Playwright's
    // actionability checks pass — the new "RC" button added in
    // `rust-app/frontend/src/participants.rs` can push the Mute button
    // outside the viewport in narrow test windows. Using a real `click()`
    // (rather than `dispatchEvent('click')`) keeps the visibility/stability
    // checks active so genuine UI regressions still fail this test.
    await muteBtn.scrollIntoViewIfNeeded();
    await muteBtn.click();

    // 4. Verify User B is muted
    // Host sees Mute button disappear (or change state? Logic removes it if muted)
    await expect(muteBtn).not.toBeVisible();
    // Host sees Muted Icon on User B
    await expect(userBRow.locator('span:has-text("🔇")')).toBeVisible();

    // User B sees Toast
    await expect(pageB.locator('.toast')).toContainText('You have been muted by the host.');
    // User B mic button changes to "Unmute"
    await expect(pageB.locator('button:has-text("Unmute")')).toBeVisible();

    await contextA.close();
    await contextB.close();
  });

  test('Host can transfer host role', async ({ browser }) => {
    const roomName = `ModerationRoom2_${Date.now()}`;
    const contextA = await browser.newContext(); // Host
    const pageA = await contextA.newPage();
    const contextB = await browser.newContext(); // Participant
    const pageB = await contextB.newPage();

    // 1. Host joins
    await pageA.goto('/');
    await pageA.fill('input[type="text"]', roomName);
    await pageA.click('button:has-text("Start Meeting")');
    await pageA.locator('.prejoin-container input[type="text"]').fill('Host User');
    await pageA.click('button:has-text("Join Meeting")');
    await expect(pageA.locator('.video-grid')).toBeVisible();
    // Host has "End Meeting"
    await expect(pageA.locator('button:has-text("End Meeting")')).toBeVisible();

    // 2. Participant joins
    await pageB.goto(`/room/${roomName}`);
    await pageB.locator('.prejoin-container input[type="text"]').fill('User B');
    await pageB.click('button:has-text("Join Meeting")');
    await expect(pageB.locator('.video-grid')).toBeVisible();
    // Participant has "Leave" but NOT "End Meeting"
    await expect(pageB.locator('button:has-text("End Meeting")')).not.toBeVisible();

    // 3. Host transfers role to User B
    if (await pageA.locator('.participants-list').isHidden()) { await pageA.click('#toggle-participants-btn'); }
    const userBRow = pageA.locator('.participants-list li').filter({ hasText: 'User B' });
    await expect(userBRow).toBeVisible();
    const hostBtn = userBRow.getByRole('button', { name: 'Host', exact: true });
    await hostBtn.dispatchEvent('click');

    // 4. Verify Transfer
    // User B gets "End Meeting"
    await expect(pageB.locator('button:has-text("End Meeting")')).toBeVisible();
    // User A loses "End Meeting"
    await expect(pageA.locator('button:has-text("End Meeting")')).not.toBeVisible();
    // User A sees "(Host)" label on User B in list
    const userBRowInA = pageA.locator('li').filter({ hasText: 'User B' });
    await expect(userBRowInA).toContainText('(Host)');

    await contextA.close();
    await contextB.close();
  });


  test('Moderator tab in settings', async ({ browser }) => {
    const roomName = `ModeratorTabRoom_${Date.now()}`;
    const contextA = await browser.newContext(); // Host
    const pageA = await contextA.newPage();
    const contextB = await browser.newContext(); // Participant
    const pageB = await contextB.newPage();

    // 1. Host joins
    await pageA.goto('/');
    await pageA.fill('input[type="text"]', roomName);
    await pageA.click('button:has-text("Start Meeting")');
    await pageA.locator('.prejoin-container input[type="text"]').fill('Host User');
    await pageA.click('button:has-text("Join Meeting")');
    await expect(pageA.locator('.video-grid')).toBeVisible();

    // 2. Participant joins
    await pageB.goto(`/room/${roomName}`);
    await pageB.locator('.prejoin-container input[type="text"]').fill('User B');
    await pageB.click('button:has-text("Join Meeting")');
    await expect(pageB.locator('.video-grid')).toBeVisible();

    // 3. Host opens settings and sees Moderator tab
    await pageA.click('#settings-btn');
    await expect(pageA.locator('.modal-content')).toBeVisible();
    const moderatorTabButtonA = pageA.locator('.modal-content .tabs button:has-text("Moderator")');
    await expect(moderatorTabButtonA).toBeVisible();

    // 4. Host uses Moderator tab
    await moderatorTabButtonA.click();
    const lockRoomCheckbox = pageA.locator('.modal-content input[type="checkbox"]').nth(0);
    const lobbyCheckbox = pageA.locator('.modal-content input[type="checkbox"]').nth(1);

    // Toggle lock
    await lockRoomCheckbox.check();
    // Verification logic can be added here if there's a UI indicator for lock, or just checking the checkbox state
    await expect(lockRoomCheckbox).toBeChecked();

    await pageA.click('.modal-content button:has-text("×")'); // Close settings

    // 5. Participant opens settings and does NOT see Moderator tab
    await pageB.click('#settings-btn');
    await expect(pageB.locator('.modal-content')).toBeVisible();
    const moderatorTabButtonB = pageB.locator('.modal-content .tabs button:has-text("Moderator")');
    await expect(moderatorTabButtonB).not.toBeVisible();

    await contextA.close();
    await contextB.close();
  });
});
