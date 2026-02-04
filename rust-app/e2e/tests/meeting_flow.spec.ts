import { test, expect } from '@playwright/test';

test.describe('Meeting Flow', () => {
  test('should allow user to join a room', async ({ page }) => {
    // 1. Visit home page
    await page.goto('/');
    
    // 2. Check for Prejoin Screen
    await expect(page.getByText('Join Meeting')).toBeVisible();
    
    // 3. Enter name and join
    await page.fill('input[placeholder="Enter your name"]', 'Test User');
    await page.click('button:has-text("Join Meeting")');
    
    // 4. Verify joined state (e.g., check for toolbox or room ID)
    await expect(page.locator('.room-toolbox')).toBeVisible();
    await expect(page.getByText('Validation User')).not.toBeVisible(); // Just a sanity check
  });
});
