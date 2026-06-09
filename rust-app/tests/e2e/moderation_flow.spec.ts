import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    try {
        await page.click('button[title="Login"]', { timeout: 2000 });
        await page.fill('input[placeholder="user@domain.com"]', 'admin');
        await page.fill('input[placeholder="Password"]', 'admin123');
        await page.click('button:has-text("Login")');
    } catch (e) {
        // Fallback or already logged in
    }
}


test.describe('AV Moderation Flow', () => {
    test('Host can enable moderation and grant permissions to participants', async ({ context }) => {
        const roomName = `moderation-test-${Math.random().toString(36).substring(7)}`;

        // 1. Host joins
        const hostPage = await context.newPage();
        await hostPage.goto('/');
        await hostPage.fill('#meeting-name', roomName);
        await hostPage.click('.create-btn');
        await hostPage.fill('#display-name', 'Host');
        await hostPage.click('.join-btn');
        await expect(hostPage.locator('.room-container')).toBeVisible();

        // 2. Participant joins
        const participantPage = await context.newPage();
        await participantPage.goto(`/room/${encodeURIComponent(roomName)}`);
        await participantPage.fill('#display-name', 'Participant');
        await participantPage.click('.join-btn');
        await expect(participantPage.locator('.room-container')).toBeVisible();

        // 3. Host enables audio moderation
        await hostPage.click('#settings-btn');
        await hostPage.click('button:has-text("Moderator")');
        const audioToggle = hostPage.locator('#audio-moderation-toggle');
        await audioToggle.check();
        await hostPage.click('#close-settings-btn');

        // 4. Participant tries to speak (mock) and sees error or is restricted
        // Since we can't easily trigger the "Speaking" WS message from Playwright without
        // internal hooks, we verify the "Req Mic" button is visible in the toolbox.
        await expect(participantPage.locator('button:has-text("Req Mic")')).toBeVisible();
        await participantPage.click('button:has-text("Req Mic")');

        // 5. Host sees "Grant Mic" button in participants list
        if (await hostPage.locator('.participants-list').isHidden()) {
            await hostPage.click('#toggle-participants-btn');
        }
        await hostPage.waitForSelector(".participants-list", { state: "visible" });
        const grantBtn = hostPage.locator('button:has-text("Grant Mic")');
        await hostPage.waitForSelector(".grant-mic-btn", { state: "visible" });
        await expect(grantBtn).toBeVisible({ timeout: 15000 });
        await grantBtn.click();

        // 6. Participant sees success toast
        await expect(participantPage.locator('.toast-success')).toContainText('granted permission to unmute');
    });

    test('Integrations tab connection flow', async ({ page }) => {
        await page.goto('/');
        await page.fill('#meeting-name', 'integration-test');
        await page.click('.create-btn');
        await page.fill('#display-name', 'User');
        await page.click('.join-btn');

        await page.click('#settings-btn');
        await page.click('button:has-text("Integrations")');

        const dropboxBtn = page.locator('.integration-item:has-text("Dropbox") button');
        await expect(dropboxBtn).toHaveText('Connect');
        await dropboxBtn.click();

        // Check for mock connecting state
        await expect(page.locator('.toast-info')).toContainText('Connecting to Dropbox');

        // Wait for connection
        await expect(dropboxBtn).toHaveText('Disconnect', { timeout: 5000 });
        await expect(page.locator('.toast-success')).toContainText('Connected to Dropbox');
    });

    test('Visitor mode enforces read-only chat', async ({ page }) => {
        await page.goto('/');
        await page.fill('#meeting-name', 'visitor-test');
        await page.click('.create-btn');
        await page.fill('#display-name', 'Visitor');
        await page.check('#visitor-mode');
        await page.click('.join-btn');

        await expect(page.locator('.room-container')).toBeVisible();

        // Check chat input is disabled
        if (await page.locator('.chat-container').isHidden()) {
            await page.click('#toggle-chat-btn');
        }
        await page.waitForSelector("#chat-panel", { state: "visible" });
        const chatInput = page.locator('#chat-input');
        await page.waitForSelector("#chat-input", { state: "visible" });

        await expect(chatInput).toBeDisabled({ timeout: 15000 });
        await expect(chatInput).toHaveAttribute('placeholder', /Visitor Mode/);
    });
});
