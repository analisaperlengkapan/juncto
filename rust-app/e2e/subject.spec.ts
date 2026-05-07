import { test, expect } from '@playwright/test';

test.describe('Meeting Subject', () => {
    test.beforeEach(async ({ page }) => {
        await page.request.post('/api/rooms', {
            data: {
                room_name: 'TestRoom',
                is_locked: false,
                is_recording: false,
                is_lobby_enabled: false,
                max_participants: 10,
                subject: null
            }
        });
    });

    test('host can set and update meeting subject', async ({ page }) => {
        page.on('console', msg => console.log('BROWSER LOG:', msg.text()));

        await page.goto('/');
        await page.fill('#meeting-name', 'TestRoom');
        await page.click('.create-btn');

        await page.waitForSelector('#display-name', { timeout: 30000 });
        await page.fill('#display-name', 'Host');

        const joinBtn = page.locator('.join-btn');
        await expect(joinBtn).toBeEnabled({ timeout: 30000 });
        await joinBtn.click();

        await page.waitForSelector('.room-container', { timeout: 30000 });

        // Ensure we are host (Moderator tab should eventually appear)
        await page.click('button[title="Settings"]');
        const moderatorTab = page.locator('button:has-text("Moderator")');
        await expect(moderatorTab).toBeVisible({ timeout: 20000 });
        await moderatorTab.click();

        await page.fill('#settings-subject', 'Project Alpha');
        await page.click('#update-subject-btn');
        await page.click('#close-settings-btn');

        const subject = page.locator('#meeting-subject');
        await expect(subject).toBeVisible({ timeout: 10000 });
        await expect(subject).toHaveText('Project Alpha');
    });
});
