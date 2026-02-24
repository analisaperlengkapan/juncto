import { test, expect } from '@playwright/test';

test.describe('Invite and Chat Features', () => {
  test.beforeEach(async ({ request }) => {
    // Reset room state
    const response = await request.post('/api/rooms', {
        data: {
            room_name: 'InviteTest',
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

  test('Invite Dialog and Chat Toggle', async ({ page }) => {
    const roomName = `InviteRoom_${Date.now()}`;

    // 1. Join Meeting
    await page.goto('/');
    await page.fill('input[type="text"]', roomName);
    await page.click('button:has-text("Start Meeting")');
    await page.locator('.prejoin-container input[type="text"]').fill('Alice');
    await page.click('button:has-text("Join Meeting")');
    await expect(page.locator('.video-grid')).toBeVisible();

    // 2. Verify Invite Feature
    const inviteButton = page.locator('button:has-text("Invite")');
    await expect(inviteButton).toBeVisible();
    await inviteButton.click();

    // Dialog should open
    const modal = page.locator('.modal-content');
    await expect(modal).toBeVisible();
    await expect(modal.locator('h3')).toHaveText('Invite People');

    // Check URL
    const urlInput = modal.locator('input[readonly]');
    const inputValue = await urlInput.inputValue();
    expect(inputValue).toContain(`/room/${encodeURIComponent(roomName)}`); // URL encoded name

    // Click Copy Link
    // Note: Clipboard write might be blocked in headless unless permitted.
    // Playwright usually grants clipboard-read/write permissions by default or we might need to grant them.
    // However, we are checking for the Toast notification which is triggered by the click success path (mostly).
    // In our implementation, we didn't await the promise result for toast, so toast shows immediately.
    await modal.locator('button:has-text("Copy Link")').click();

    // Verify Toast
    const toast = page.locator('.toast');
    await expect(toast).toContainText('Link Copied!');
    await toast.click(); // Dismiss toast

    // Close Modal
    await modal.locator('button:has-text("×")').click();
    await expect(modal).not.toBeVisible();

    // 3. Verify Chat Toggle
    const chatButton = page.locator('button:has-text("Chat")');
    await expect(chatButton).toBeVisible();

    // Chat should be visible initially
    const chatContainer = page.locator('.chat-container').first();
    // Since we wrapped it in a div with display style, let's target that wrapper or check visibility of chat-container
    // The wrapper has style="display: block;" or "display: none;"
    // Playwright .toBeVisible() checks computed style.
    await expect(chatContainer).toBeVisible();

    // Toggle Off
    await chatButton.click();
    await expect(chatContainer).not.toBeVisible();

    // Toggle On
    await chatButton.click();
    await expect(chatContainer).toBeVisible();
  });
});
