import { test, expect } from '@playwright/test';

test.beforeEach(async ({ request }) => {
  // Reset backend room state before each test
  await request.post('/api/rooms', {
    data: {
      room_name: "Default Room",
      is_locked: false,
      is_recording: false,
      is_lobby_enabled: false,
      max_participants: 100,
      host_id: null,
      e2ee_enabled: false,
      is_subtitles_enabled: false,
      etherpad_url: null
    }
  });
});

test('Authentication flow', async ({ page }) => {
  const roomName = `Auth-${Math.random().toString(36).substring(7)}`;
  await page.goto('/');

  // Wait for the WASM to load and the welcome container to appear
  await page.waitForSelector('.welcome-container', { timeout: 120000 });
  await page.fill('.welcome-container input[type="text"]', roomName);
  await page.click('button.create-btn');

  // Now in Prejoin
  await page.waitForSelector('.prejoin-container', { timeout: 30000 });
  const nameInput = page.getByPlaceholder('Enter your name');
  await expect(nameInput).toBeVisible();
  await nameInput.fill('Tester');

  const btn = page.getByRole('button', { name: 'Join Meeting' }).or(page.locator('button.join-btn'));

  await expect(btn).toBeEnabled();
  await btn.click();

  // Now in Room
  await page.waitForSelector('.room-container', { timeout: 30000 });

  // Find Login button in Toolbox
  // Using a more specific selector to avoid strict mode violations if multiple buttons exist
  const loginBtn = page.locator('.toolbox button').filter({ hasText: 'Login' }).first();
  await expect(loginBtn).toBeVisible();
  await loginBtn.click();

  // Authentication Dialog
  await page.waitForSelector('.modal-content', { timeout: 10000 });
  await page.getByPlaceholder('Username').fill('admin');
  await page.getByPlaceholder('Password').fill('wrong');
  await page.locator('.modal-content button').filter({ hasText: 'Login' }).click();
  await expect(page.locator('text=Invalid username or password')).toBeVisible();

  await page.getByPlaceholder('Password').fill('admin123');
  await page.locator('.modal-content button').filter({ hasText: 'Login' }).click();
  await expect(page.locator('text=Authenticated successfully')).toBeVisible();
});

test('Integrations UI', async ({ page }) => {
  const roomName = `Int-${Math.random().toString(36).substring(7)}`;
  await page.goto('/');
  await page.waitForSelector('.welcome-container', { timeout: 120000 });
  await page.fill('.welcome-container input[type="text"]', roomName);
  await page.click('button.create-btn');

  await page.waitForSelector('.prejoin-container', { timeout: 30000 });
  await page.getByPlaceholder('Enter your name').fill('Tester');
  const btn = page.getByRole('button', { name: 'Join Meeting' }).or(page.locator('button.join-btn'));
  await btn.click();

  await page.waitForSelector('.room-container', { timeout: 30000 });

  const settingsBtn = page.locator('button').filter({ hasText: 'Settings' }).first();
  await expect(settingsBtn).toBeVisible();
  await settingsBtn.click();

  // Open Integrations tab
  const integrationsTab = page.locator('button').filter({ hasText: 'Integrations' }).first();
  await expect(integrationsTab).toBeVisible({ timeout: 10000 });
  await integrationsTab.click();

  await expect(page.locator('.integration-item').filter({ hasText: 'Dropbox' })).toBeVisible();
  await expect(page.locator('.integration-item').filter({ hasText: 'Salesforce' })).toBeVisible();
  await expect(page.locator('.integration-item').filter({ hasText: 'Google Calendar' })).toBeVisible();
});
