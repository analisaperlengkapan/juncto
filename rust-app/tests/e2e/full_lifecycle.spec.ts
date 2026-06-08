import { test, expect } from '@playwright/test';

test.describe('Juncto Full Lifecycle Integration', () => {
    test('Complete flow from Home to Room, including multi-user interactions and moderation', async ({ context }) => {
        const roomName = `lifecycle-${Math.random().toString(36).substring(7)}`;

        // 1. Host Session Start
        const hostPage = await context.newPage();
        await hostPage.goto('http://localhost:3000/');
        await hostPage.fill('#meeting-name', roomName);
        await hostPage.click('.create-btn');

        // Prejoin Screen
        await expect(hostPage).toHaveURL(new RegExp(`/room/${encodeURIComponent(roomName)}`));
        await hostPage.fill('#display-name', 'Host Admin');
        await hostPage.click('.join-btn');

        // Verify Room Entry
        await expect(hostPage.locator('.room-container')).toBeVisible({ timeout: 60000 });
        await expect(hostPage.locator('.participant-item')).toContainText('Host Admin (Host)');

        // 2. Visitor Session Start
        const visitorPage = await context.newPage();
        await visitorPage.goto(`http://localhost:3000/room/${encodeURIComponent(roomName)}`);
        await visitorPage.fill('#display-name', 'Visitor Guest');
        await visitorPage.check('#visitor-mode');
        await visitorPage.click('.join-btn');

        // Verify Visitor Entry and UI Restrictions
        await expect(visitorPage.locator('.room-container')).toBeVisible({ timeout: 60000 });
        await expect(visitorPage.locator('#toggle-camera-btn')).not.toBeVisible();
        await expect(visitorPage.locator('#chat-input')).toHaveAttribute('placeholder', /Visitor Mode/);

        // 3. Host Moderation: Promote Visitor
        if (await hostPage.locator('#participants-panel').getAttribute('class').then(c => c?.includes('panel-hidden'))) {
            await hostPage.click('#toggle-participants-btn');
        }
        const visitorItem = hostPage.locator('.participant-item:has-text("Visitor Guest")');
        await visitorItem.locator('#promote-btn').click();

        // Verify Visitor is now a full participant
        await expect(visitorPage.locator('#toggle-camera-btn')).toBeVisible();
        await expect(visitorPage.locator('#chat-input')).not.toHaveAttribute('placeholder', /Visitor Mode/);

        // 4. Multi-User Chat
        // Host sends message
        await hostPage.fill('#chat-input', 'Hello everyone!');
        await hostPage.press('#chat-input', 'Enter');
        await expect(visitorPage.locator('.chat-message:has-text("Hello everyone!")').first()).toBeVisible();

        // 5. Polls Lifecycle (Promoted visitor can now vote)
        await hostPage.click('#toggle-polls-btn');
        await hostPage.click('button:has-text("Create Poll")');
        await hostPage.fill('#poll-question', 'Is Rust the best?');
        await hostPage.fill('#poll-option-1', 'Yes');
        await hostPage.fill('#poll-option-2', 'Absolutely');
        await hostPage.click('#create-poll-submit-btn');

        // Visitor votes
        await visitorPage.click('#toggle-polls-btn');
        const pollItem = visitorPage.locator('.poll-item:has-text("Is Rust the best?")').first();
        await expect(pollItem).toBeVisible({ timeout: 10000 });
        // Use dispatchEvent to ensure click works even if element is overlapping
        await pollItem.getByRole('button', { name: 'Vote' }).first().dispatchEvent('click');

        // Host verifies results (checking for at least 1 vote)
        await expect(hostPage.locator('.poll-item:has-text("Is Rust the best?")').first()).toContainText(/[1-9]\d* votes/, { timeout: 10000 });
        await hostPage.click('#close-polls-btn');

        // 6. AV Moderation: Mute All
        await hostPage.click('#mute-all-btn');
        // Toast notification on visitor side
        await expect(visitorPage.locator('.toast-container')).toContainText('You have been muted by the host');

        // 7. Breakout Rooms
        await hostPage.fill('input[placeholder="New Room Name"]', 'Side Discussion');
        await hostPage.click('button:has-text("Create")');

        await hostPage.click('button:has-text("Auto Assign")');
        // Verify visitor moved (should see "(In Breakout Room)")
        await expect(visitorPage.locator('h4:has-text("(In Breakout Room)")')).toBeVisible({ timeout: 10000 });

        // 8. E2EE Toggle
        await hostPage.click('#settings-btn');
        await hostPage.click('button:has-text("Moderator")');
        const e2eeToggle = hostPage.locator('#e2ee-toggle');
        await e2eeToggle.check();
        await hostPage.click('#close-settings-btn');

        // Verify lock icons (2 participants * 2 icons each = 4)
        // Parity test uses haveCount(4), so we follow that pattern
        await expect(hostPage.locator('.e2ee-lock')).toHaveCount(4, { timeout: 15000 });
        await expect(visitorPage.locator('.e2ee-lock')).toHaveCount(4, { timeout: 15000 });

        // 9. Cleanup: End Meeting
        await hostPage.click('button[title="End Meeting for Everyone"]');

        // Both redirected to Home
        await expect(hostPage).toHaveURL('http://localhost:3000/', { timeout: 15000 });
        await expect(visitorPage).toHaveURL('http://localhost:3000/', { timeout: 15000 });
    });
});
