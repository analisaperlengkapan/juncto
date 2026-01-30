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
});
