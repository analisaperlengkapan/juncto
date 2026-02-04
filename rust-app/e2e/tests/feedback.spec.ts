import { test, expect } from '@playwright/test';

test.describe('Feedback Feature', () => {
  test('should allow user to submit feedback', async ({ page }) => {
    // 1. Join Room
    await page.goto('/');
    
    // Handle Prejoin if it exists (assuming input[type="text"] is for name)
    await page.fill('input[type="text"]', 'Test User');
    await page.click('button:has-text("Join Meeting")');

    // Wait for room to load (check for Toolbox)
    await expect(page.locator('.toolbox')).toBeVisible();

    // 2. Open Feedback Dialog
    // We added title="Feedback" to the button
    await page.click('button[title="Feedback"]');
    
    // Verify Modal
    await expect(page.getByText('Rate Your Experience')).toBeVisible();

    // 3. Rate 5 Stars
    // Stars are spans with "★". We can click the last one.
    const stars = await page.locator('.rating-stars span').all();
    await stars[4].click(); // 5th star

    // 4. Fill Comment
    await page.fill('textarea', 'Excellent call quality!');

    // 5. Submit
    await page.click('button:has-text("Submit")');

    // 6. Verify Success
    await expect(page.getByText('Thank You!')).toBeVisible();
    
    // 7. Verify Dialog Closes (after 2s)
    await expect(page.getByText('Rate Your Experience')).not.toBeVisible({ timeout: 5000 });
  });
});
