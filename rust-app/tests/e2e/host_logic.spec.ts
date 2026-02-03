import { test, expect } from '@playwright/test';

test.describe('Host Logic', () => {
  // We don't use beforeEach to navigate because we create fresh contexts in each test.

  test('Host Reassignment: When host leaves, next participant becomes host', async ({ browser }) => {
    const roomName = `HostLogicRoom1_${Date.now()}`;
    // 1. User A creates room (Host)
    const contextA = await browser.newContext();
    const pageA = await contextA.newPage();
    await pageA.goto('/');

    // Create Room
    await pageA.fill('input[type="text"]', roomName);
    await pageA.click('button:has-text("Start Meeting")');

    // Prejoin
    await pageA.locator('.prejoin-container input[type="text"]').fill('Host User');
    await pageA.click('button:has-text("Join Meeting")');

    await expect(pageA.locator('.video-grid')).toBeVisible();

    // Verify A is host (sees Host controls, e.g., Lock Room or End Meeting)
    // "End Meeting" is only visible to host
    await expect(pageA.locator('button:has-text("End Meeting")')).toBeVisible();

    // 2. User B joins
    const contextB = await browser.newContext();
    const pageB = await contextB.newPage();
    // Join same room
    await pageB.goto(`/room/${roomName}`);

    // Prejoin
    await pageB.locator('.prejoin-container input[type="text"]').fill('User B');
    await pageB.click('button:has-text("Join Meeting")');

    await expect(pageB.locator('.video-grid')).toBeVisible();

    // Verify B is NOT host initially
    await expect(pageB.locator('button:has-text("End Meeting")')).not.toBeVisible();

    // 3. User A leaves
    // Use the Leave button
    await pageA.click('button:has-text("Leave")');
    // Page A should go back to home
    await expect(pageA.locator('button:has-text("Start Meeting")')).toBeVisible();

    // 4. User B should become host
    // Wait for B to receive update
    await expect(pageB.locator('button:has-text("End Meeting")')).toBeVisible({ timeout: 10000 });

    await contextA.close();
    await contextB.close();
  });

  test('End Meeting: Host can end meeting for everyone', async ({ browser }) => {
    const roomName = `HostLogicRoom2_${Date.now()}`;
    // 1. User A creates room (Host)
    const contextA = await browser.newContext();
    const pageA = await contextA.newPage();
    await pageA.goto('/');

    // Create Room
    await pageA.fill('input[type="text"]', roomName);
    await pageA.click('button:has-text("Start Meeting")');

    // Prejoin
    await pageA.locator('.prejoin-container input[type="text"]').fill('Host User');
    await pageA.click('button:has-text("Join Meeting")');

    await expect(pageA.locator('.video-grid')).toBeVisible();

    // 2. User B joins
    const contextB = await browser.newContext();
    const pageB = await contextB.newPage();
    await pageB.goto(`/room/${roomName}`);

    // Prejoin
    await pageB.locator('.prejoin-container input[type="text"]').fill('User B');
    await pageB.click('button:has-text("Join Meeting")');

    await expect(pageB.locator('.video-grid')).toBeVisible();

    // 3. User A ends meeting
    // pageA.on('dialog', dialog => dialog.accept()); // In case there's a confirmation (not implemented yet but good practice)
    await pageA.click('button:has-text("End Meeting")');

    // 4. Verify both are returned to Prejoin screen or Home
    // In state.rs: RoomEnded -> set_current_state(Prejoin) -> PrejoinScreen
    // So they should see "Join Meeting" button / Prejoin UI

    // Page A
    await expect(pageA.locator('button:has-text("Join Meeting")')).toBeVisible();
    await expect(pageA.locator('.video-grid')).not.toBeVisible();

    // Page B
    await expect(pageB.locator('button:has-text("Join Meeting")')).toBeVisible();
    await expect(pageB.locator('.video-grid')).not.toBeVisible();

    // Check for Toast on Page B
    await expect(pageB.locator('.toast')).toContainText('The meeting has ended by the host.');

    await contextA.close();
    await contextB.close();
  });
});
