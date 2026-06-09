import { test, expect, BrowserContext } from '@playwright/test';

async function loginAsAdmin(page) {
    await page.click('button[title="Login"]');
    await page.fill('input[placeholder="Username"]', 'admin');
    await page.fill('input[placeholder="Password"]', 'admin123');
    await page.click('button:has-text("Login")');
}

async function setupGiphyMock(context: BrowserContext) {
    await context.route(/.*api.giphy.com.*/, async route => {
        const json = {
            data: [
                {
                    id: 'mock1',
                    title: 'Mock GIF',
                    images: {
                        fixed_height: {
                            url: 'https://media.giphy.com/media/v1.Y2lkPTc5MGI3NjExNHJicm9ueGZ4eGZ4eGZ4eGZ4eGZ4eGZ4eGZ4eGZ4eGZ4JmVwPXYxX2ludGVybmFsX2dpZl9ieV9pZCZjdD1n/3o7TKMGpxx66mSstq0/giphy.gif',
                            width: '200',
                            height: '200'
                        }
                    }
                }
            ]
        };
        await route.fulfill({
            status: 200,
            contentType: 'application/json',
            body: JSON.stringify(json)
        });
    });
}

test.describe('Migration Full Integration', () => {
    test('Storage, Giphy, Etherpad, and E2EE', async ({ page, browser }) => {
        await setupGiphyMock(page.context());

        await page.goto('http://localhost:3000/room/FullIntegration');
        await page.fill('#display-name', 'Host');
        await page.click('.join-btn');
        await loginAsAdmin(page);

        // 2. Giphy Test
        await page.click('#toggle-chat-btn');
        await page.getByRole('button', { name: 'GIF' }).click();
        await expect(page.locator('.giphy-search')).toBeVisible();

        await page.fill('.giphy-search input', 'hello');
        await expect(page.locator('.giphy-grid img').first()).toBeVisible({ timeout: 20000 });
        await page.locator('.giphy-grid img').first().click();
        await expect(page.locator('.messages img').first()).toBeVisible();

        // 3. E2EE Test
        await page.click('#settings-btn');
        await page.click('button:has-text("Moderator")');
        await page.locator('#e2ee-toggle').check();
        await page.click('#close-settings-btn');
        await expect(page.locator('#e2ee-indicator')).toBeVisible();

        // 4. Etherpad Test
        await page.click('#toggle-etherpad-btn');
        await expect(page.locator('.etherpad-container')).toBeVisible();
    });
});
