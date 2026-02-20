import { test, expect } from '@playwright/test';

test.describe('Breakout Rooms', () => {
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

    // Create Room
    // Assuming BreakoutRooms component is visible (it is usually part of the sidebar or main view depending on implementation,
    // but based on room.rs it seems embedded).
    // The component has "Breakout Rooms" header.

    // Wait for component to load
    await expect(pageA.locator('h4:has-text("Breakout Rooms")')).toBeVisible();

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
