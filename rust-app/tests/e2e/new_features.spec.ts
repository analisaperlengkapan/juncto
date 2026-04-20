import { test, expect } from '@playwright/test';

test('Authentication flow', async ({ page }) => {
  await page.goto('/');
  // Wait for the WASM to load and the welcome container to appear
  await page.waitForSelector('.welcome-container', { timeout: 30000 });
  await page.fill('.welcome-container input[type="text"]', 'AuthTest');
  await page.click('button.create-btn');

  // Now in Prejoin
  await page.waitForSelector('.prejoin-container', { timeout: 30000 });
  const nameInput = page.locator('input[placeholder="Enter your name"]');
  await expect(nameInput).toBeVisible();
  await nameInput.fill('Tester');

  const joinBtn = page.locator('button.join-btn');
  await expect(joinBtn).toBeEnabled();
  await joinBtn.click();

  // Now in Room
  await page.waitForSelector('.room-container', { timeout: 30000 });

  // Find Login button in Toolbox
  const loginBtn = page.locator('button:has-text("Login")');
  await expect(loginBtn).toBeVisible();
  await loginBtn.click();

  // Authentication Dialog
  await page.waitForSelector('.modal-content', { timeout: 10000 });
  await page.fill('input[placeholder="Username"]', 'admin');
  await page.fill('input[placeholder="Password"]', 'wrong');
  await page.click('.modal-content button:has-text("Login")');
  await expect(page.locator('text=Invalid username or password')).toBeVisible();

  await page.fill('input[placeholder="Password"]', 'admin123');
  await page.click('.modal-content button:has-text("Login")');
  await expect(page.locator('text=Authenticated successfully')).toBeVisible();
});

test('Integrations UI', async ({ page }) => {
  await page.goto('/');
  await page.waitForSelector('.welcome-container', { timeout: 30000 });
  await page.fill('.welcome-container input[type="text"]', 'IntegrationsTest');
  await page.click('button.create-btn');

  await page.waitForSelector('.prejoin-container', { timeout: 30000 });
  await page.fill('input[placeholder="Enter your name"]', 'Tester');
  await page.click('button.join-btn');

  await page.waitForSelector('.room-container', { timeout: 30000 });

  const settingsBtn = page.locator('button:has-text("Settings")');
  await expect(settingsBtn).toBeVisible();
  await settingsBtn.click();

  // Open Integrations tab
  const integrationsTab = page.locator('button:has-text("Integrations")');
  await expect(integrationsTab).toBeVisible();
  await integrationsTab.click();

  await expect(page.locator('.integration-item >> text=Dropbox')).toBeVisible();
  await expect(page.locator('.integration-item >> text=Salesforce')).toBeVisible();
  await expect(page.locator('.integration-item >> text=Google Calendar')).toBeVisible();
});
