import { test, expect } from '@playwright/test';

test.describe('Prejoin Screen', () => {
  test.beforeEach(async ({ page, request }) => {
    // Reset backend state
    const response = await request.post('/api/rooms', {
      data: {
        room_name: "Test Room",
        is_locked: false,
        is_recording: false,
        is_lobby_enabled: false,
        max_participants: 100,
        host_id: null,
        e2ee_enabled: false
      }
    });
    expect(response.status()).toBe(201);
    // Go directly to a room to trigger Prejoin
    await page.goto('/room/e2e-test-room');
  });

  test('should show prejoin screen by default', async ({ page }) => {
    await expect(page.locator('h2')).toHaveText('Join Meeting');
    await expect(page.locator('input[type="text"]')).toHaveValue('Guest');
    // Check for video preview element (even if camera is mocked)
    await expect(page.locator('video')).toBeVisible();
  });

  test('should join with default settings (Camera ON)', async ({ page }) => {
    await page.fill('input[type="text"]', 'Alice');
    await page.click('button:has-text("Join Meeting")');

    // Should navigate to room
    await expect(page.locator('.room-container')).toBeVisible();

    // Check local video is showing (not "Camera Off")
    // The VideoGrid shows <video> if stream exists
    await expect(page.locator('.local-video video')).toBeVisible();
    await expect(page.locator('.local-video')).not.toContainText('Camera Off');
  });

  test('should join with Camera OFF', async ({ page }) => {
    await page.fill('input[type="text"]', 'Bob');

    // Toggle Camera OFF in Prejoin
    // Button has title "Toggle Camera"
    await page.click('button[title="Toggle Camera"]');

    // Verify preview shows "Camera is Off" text or similar fallback
    // In PrejoinScreen fallback: <div style="color: white;">Camera is Off</div>
    await expect(page.locator('.prejoin-container')).toContainText('Camera is Off');

    await page.click('button:has-text("Join Meeting")');

    // Should navigate to room
    await expect(page.locator('.room-container')).toBeVisible();

    // Check local video shows "Camera Off" fallback
    await expect(page.locator('.local-video')).toContainText('Camera Off');
    await expect(page.locator('.local-video video')).not.toBeVisible();
  });
});
