import { test, expect } from '@playwright/test';

test('Juncto Migration E2E (WASM)', async ({ page, request }) => {
  page.on('console', msg => console.log('BROWSER LOG:', msg.text()));
  page.on('pageerror', err => console.log('BROWSER ERROR:', err));

  // 1. Check API Health
  const healthResponse = await request.get('http://localhost:3000/health');
  expect(healthResponse.status()).toBe(200);

  // 2. Load Frontend
  await page.goto('/');
  await expect(page).toHaveTitle(/Juncto/);

  // Wait for WASM to hydrate and show content
  await expect(page.getByText('Welcome to Juncto (Rust Edition)')).toBeVisible({ timeout: 10000 });

  // 3. Interact (Start Meeting)
  const input = page.locator('input[type="text"]');
  await expect(input).toBeVisible();
  await input.fill('Rust Meeting');
  await page.click('button.create-btn');

  // 4. Prejoin Screen
  // "Rust Meeting" gets encoded to "Rust%20Meeting"
  await expect(page).toHaveURL(/\/room\/Rust%20Meeting/);
  await expect(page.getByText('Ready to join?')).toBeVisible();

  // Enter Name and Join
  const nameInput = page.locator('.prejoin-container input[type="text"]');
  await nameInput.fill('E2E User');
  await page.click('button.join-btn');

  // 5. Verify Room UI
  // The component likely displays the decoded parameter
  await expect(page.getByText('Meeting Room: Rust Meeting')).toBeVisible();

  // 5. Verify Chat Functionality
  const chatInput = page.locator('.chat-container input[type="text"]');
  const chatSendBtn = page.locator('.chat-container button');

  await expect(chatInput).toBeVisible();

  // Wait for connection
  await expect(chatSendBtn).toBeEnabled();
  await expect(chatSendBtn).toHaveText('Send');

  await chatInput.fill('Hello from E2E');
  await chatSendBtn.click();

  // Verify message appears (User ID "Me" is hardcoded in chat.rs)
  // Check for the content first as it's most unique
  await expect(page.getByText('Hello from E2E')).toBeVisible();

  // 6. Verify Participants List
  const participantsList = page.locator('.participants-list');
  await expect(participantsList).toBeVisible();
  // Should contain at least "User ..." because backend assigns random names starting with "User"
  await expect(participantsList.locator('ul')).toContainText('User');

  // 7. Verify Room Lock
  const lockBtn = page.getByRole('button', { name: 'Lock Room' });
  await expect(lockBtn).toBeVisible();
  await lockBtn.click();

  // Verify button text changes to "Unlock Room"
  await expect(page.getByRole('button', { name: 'Unlock Room' })).toBeVisible();

  // 8. Verify Settings / Profile Update
  // Open Settings
  await page.getByRole('button', { name: 'Settings' }).click();
  await expect(page.getByText('Save Profile')).toBeVisible();

  // Change Name
  const nameSettingInput = page.locator('.modal-content input[type="text"]');
  await nameSettingInput.fill('Updated Name');
  await page.click('button:has-text("Save Profile")');

  // Verify modal closed
  await expect(page.getByText('Save Profile')).not.toBeVisible();

  // Verify name updated in Participants List
  // Ideally we wait for the update
  await expect(participantsList.locator('ul')).toContainText('Updated Name');

  // 9. Verify Reactions
  const likeBtn = page.getByRole('button', { name: '👍' });
  await expect(likeBtn).toBeVisible();
  await likeBtn.click();

  // Verify reaction appears in the overlay
  // Note: Animation lasts 2s, so we must be quick or just check existence
  await expect(page.locator('.reaction-layer')).toContainText('👍');

  // 10. Verify Recording
  const recordBtn = page.getByRole('button', { name: 'Start Recording' });
  await expect(recordBtn).toBeVisible();
  await recordBtn.click();

  // Verify REC indicator
  await expect(page.getByText('REC', { exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Stop Recording' })).toBeVisible();

  // Stop Recording
  await page.getByRole('button', { name: 'Stop Recording' }).click();
  await expect(page.getByText('REC', { exact: true })).not.toBeVisible();

  // 11. Verify Polls
  // Open Polls
  await page.getByRole('button', { name: 'Polls' }).click();
  await expect(page.getByRole('heading', { name: 'Polls' })).toBeVisible();

  // Create Poll
  await page.getByRole('button', { name: 'Create Poll' }).click(); // Click Tab

  const pollForm = page.locator('.modal-content .tab-content');
  await expect(pollForm).toBeVisible();

  await pollForm.locator('input').nth(0).fill('Fav Color?'); // Question
  await pollForm.locator('input').nth(1).fill('Red'); // Option 1
  await pollForm.locator('input').nth(2).fill('Blue'); // Option 2

  // Click the 'Create Poll' button inside the tab content (the submit button)
  await pollForm.locator('button:has-text("Create Poll")').click();

  // Verify Poll Created
  await expect(page.getByText('Fav Color?')).toBeVisible();
  await expect(page.getByText('0 votes')).toHaveCount(2);

  // Vote
  await page.locator('button:has-text("Vote")').first().click();

  // Verify Vote Count Updated
  await expect(page.getByText('1 votes')).toBeVisible();

  // Close Polls Dialog
  await page.locator('.modal-header button').click(); // Close button "×"
  await expect(page.getByRole('heading', { name: 'Polls' })).not.toBeVisible();

  // 12. Verify Raise Hand
  const handBtn = page.getByRole('button', { name: 'Raise Hand' });
  await expect(handBtn).toBeVisible();
  await handBtn.click();

  // Verify hand icon in participants list
  // Ideally, find the list item for "E2E User" or "User ..." and check for hand emoji
  await expect(page.locator('.participants-list li').filter({ hasText: 'Updated Name' })).toContainText('✋');

  // Lower hand
  await handBtn.click();
  // Verify hand icon removed (might need short wait or check lack of text)
  await expect(page.locator('.participants-list li').filter({ hasText: 'Updated Name' })).not.toContainText('✋');

  // 13. Verify Screen Share
  const screenBtn = page.getByRole('button', { name: 'Share Screen' });
  await expect(screenBtn).toBeVisible();
  await screenBtn.click();

  // Verify screen icon in participants list
  await expect(page.locator('.participants-list li').filter({ hasText: 'Updated Name' })).toContainText('🖥️');

  // 14. Verify Whiteboard
  const wbBtn = page.getByRole('button', { name: 'Whiteboard' });
  await expect(wbBtn).toBeVisible();
  await wbBtn.click();

  const canvas = page.locator('canvas');
  await expect(canvas).toBeVisible();

  // Simulate drawing
  const box = await canvas.boundingBox();
  if (box) {
      await page.mouse.move(box.x + 10, box.y + 10);
      await page.mouse.down();
      await page.mouse.move(box.x + 50, box.y + 50);
      await page.mouse.up();
  }

  // Close Whiteboard
  await wbBtn.click();
  await expect(canvas).not.toBeVisible();

  // Unlock the room to reset state for next test
  await page.getByRole('button', { name: 'Unlock Room' }).click();
  await expect(page.getByRole('button', { name: 'Lock Room' })).toBeVisible();
});

test('Lobby Feature E2E', async ({ browser }) => {
  // Scenario:
  // 1. Host creates room, enables Lobby.
  // 2. Guest attempts to join, sees Waiting Screen.
  // 3. Host sees Guest knocking, grants access.
  // 4. Guest enters room.

  // Host context
  const hostContext = await browser.newContext();
  const hostPage = await hostContext.newPage();

  // Guest context
  const guestContext = await browser.newContext();
  const guestPage = await guestContext.newPage();

  const roomName = 'LobbyTestRoom';

  // --- HOST ---
  await hostPage.goto('/');
  await hostPage.locator('input[type="text"]').fill(roomName);
  await hostPage.click('button.create-btn');

  await hostPage.locator('.prejoin-container input[type="text"]').fill('Host');
  await hostPage.click('button.join-btn');
  await expect(hostPage.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

  // Enable Lobby
  await hostPage.getByRole('button', { name: 'Enable Lobby' }).click();
  await expect(hostPage.getByRole('button', { name: 'Disable Lobby' })).toBeVisible();

  // --- GUEST ---
  // Need to get the room URL (encoded)
  const roomUrl = hostPage.url();
  await guestPage.goto(roomUrl);
  await guestPage.locator('.prejoin-container input[type="text"]').fill('Guest');
  await guestPage.click('button.join-btn');

  // Verify Guest sees Lobby/Waiting Screen
  await expect(guestPage.getByText('Waiting for host...')).toBeVisible();

  // --- HOST ---
  // Verify Host sees Guest knocking
  await expect(hostPage.locator('.knocking-list')).toBeVisible();
  await expect(hostPage.locator('.knocking-list')).toContainText('Guest');

  // Host allows Guest
  await hostPage.getByRole('button', { name: 'Allow' }).click();

  // --- GUEST ---
  // Verify Guest enters room
  await expect(guestPage.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

  // Cleanup
  await hostContext.close();
  await guestContext.close();
});
