import { test, expect, BrowserContext } from '@playwright/test';

async function setupGiphyMock(context: BrowserContext) {
    await context.route(/.*api.giphy.com.*/, async route => {
        console.log('Mocking Giphy Request:', route.request().url());
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
    test.use({ viewport: { width: 1280, height: 1000 } });

    test('Storage, Giphy, Etherpad, and E2EE', async ({ page, browser, request }) => {
        // Mock Giphy API for host context
        await setupGiphyMock(page.context());

        page.on('console', msg => console.log('HOST LOG:', msg.text()));

        // 1. Persistence Test
        await page.goto('/');
        const roomName = `PersistenceRoom_${Date.now()}`;
        await page.fill('input[type="text"]', roomName);
        await page.click('button:has-text("Start Meeting")');
        await page.waitForURL(/\/room\//);

        const nameInput = page.locator('.prejoin-container input[type="text"]');
        await nameInput.fill('Persistent User');
        await page.click('button:has-text("Join Meeting")');

        await expect(page.getByText(/Meeting Room:/)).toBeVisible({ timeout: 10000 });

        // Wait for storage write
        await page.waitForTimeout(1000);

        // Test Persistence by navigating away and back
        await page.goto('/');
        await page.fill('input[type="text"]', roomName);
        await page.click('button:has-text("Start Meeting")');
        await page.waitForURL(/\/room\//);

        await expect(async () => {
            const val = await nameInput.inputValue();
            expect(val).toBe('Persistent User');
        }).toPass({ timeout: 20000 });

        await page.click('button:has-text("Join Meeting")');

        // 2. Giphy Test
        const chatPanel = page.locator('.side-panel.chat-container');
        const participantsPanel = page.locator('.side-panel.participants-container');

        // Use a more robust way to ensure panel state
        console.log('Adjusting panels for Giphy test');
        await expect(async () => {
            if (await participantsPanel.isVisible()) {
                await page.click('.toolbox button:has-text("Participants")');
            }
            if (await chatPanel.isHidden()) {
                await page.click('.toolbox button:has-text("Chat")');
            }
            expect(await participantsPanel.isHidden()).toBe(true);
            expect(await chatPanel.isVisible()).toBe(true);
        }).toPass({ timeout: 10000 });
        await expect(chatPanel).toBeVisible();

        await page.getByRole('button', { name: 'GIF' }).dispatchEvent('click');
        await expect(page.locator('.giphy-search')).toBeVisible({ timeout: 10000 });

        await page.fill('.giphy-search input', 'hello');
        await expect(page.locator('.giphy-grid img').first()).toHaveCount(1, { timeout: 20000 });
        const gifImg = page.locator('.giphy-grid img').first();
        const gifSrc = await gifImg.getAttribute('src');
        await gifImg.dispatchEvent('click');

        // Verify GIF appears in chat
        await expect(page.locator('.messages img').first()).toHaveAttribute('src', gifSrc || '', { timeout: 10000 });

        // 3. E2EE Test (Host)
        await page.click('.toolbox button:has-text("Settings")');
        await page.click('.tabs button:has-text("Moderator")');

        const e2eeLabel = page.locator('.modal-content label').filter({ has: page.locator('.e2ee-toggle-marker') });
        const e2eeCheckbox = page.locator('.modal-content input[type="checkbox"]').nth(2); // Based on SettingsDialog source

        if (!(await e2eeCheckbox.isChecked())) {
            await e2eeLabel.click();
        }

        await expect(page.locator('div[title^="End-to-End Encryption indicator"]')).toBeVisible({ timeout: 10000 });
        await page.click('.modal-header button:has-text("×")');

        // 4. Etherpad Test (Host)
        const etherpadContainer = page.locator('.etherpad-container');
        if (await etherpadContainer.isHidden()) {
            await page.click('.toolbox button[title="Shared Document (Etherpad)"]');
        }
        await expect(etherpadContainer).toBeVisible({ timeout: 15000 });
        await expect(page.locator('.etherpad-container iframe')).toBeAttached({ timeout: 15000 });

        // 5. Verification for other participants
        const guestContext = await browser.newContext();
        await setupGiphyMock(guestContext);
        const guestPage = await guestContext.newPage();
        guestPage.on('console', msg => console.log('GUEST LOG:', msg.text()));

        await guestPage.goto(page.url());
        await guestPage.fill('.prejoin-container input[type="text"]', 'Guest');
        await guestPage.click('button:has-text("Join Meeting")');

        await expect(guestPage.locator('div[title^="End-to-End Encryption indicator"]')).toBeVisible({ timeout: 15000 });

        const guestChatPanel = guestPage.locator('.side-panel.chat-container');
        if (await guestChatPanel.isHidden()) {
            await guestPage.click('.toolbox button:has-text("Chat")');
        }
        await expect(guestChatPanel).toBeVisible();
        await expect(guestPage.locator('.messages img').first()).toHaveCount(1, { timeout: 20000 });
    });
});
