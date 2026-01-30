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

  // 3. Interact
  const input = page.locator('input[type="text"]');
  await expect(input).toBeVisible();
  await input.fill('Rust Meeting');

  await page.click('button');

  // 4. Verify Navigation to Room
  await expect(page).toHaveURL(/\/room\/123/);
  await expect(page.getByText('Meeting Room: 123')).toBeVisible();
});
