import { test, expect } from '@playwright/test';

test.describe('Migration Parity and Complete Lifecycle', () => {
    test('Host promotes Visitor and synchronizes E2EE and Layout', async ({ context }) => {
        const roomName = `parity-test-${Math.random().toString(36).substring(7)}`;

        // 1. Host Joins
        const hostPage = await context.newPage();
        await hostPage.goto(`http://localhost:3000/room/${roomName}`);
        await hostPage.fill('#display-name', 'Host User');
        await hostPage.click('.join-btn');

        // Verify Host is in the room
        await expect(hostPage.locator('.room-container')).toBeVisible({ timeout: 60000 });
        await expect(hostPage.locator('.participant-item')).toContainText('Host User (Host)');

        // 2. Visitor Joins
        const visitorPage = await context.newPage();
        await visitorPage.goto(`http://localhost:3000/room/${roomName}`);
        await visitorPage.fill('#display-name', 'Visitor User');
        await visitorPage.check('#visitor-mode');
        await visitorPage.click('.join-btn');

        // Verify Visitor UI restrictions
        await expect(visitorPage.locator('.room-container')).toBeVisible({ timeout: 60000 });
        await expect(visitorPage.locator('#toggle-camera-btn')).not.toBeVisible();
        await expect(visitorPage.locator('#toggle-mic-btn')).not.toBeVisible();
        await expect(visitorPage.locator('#chat-input')).toHaveAttribute('placeholder', /Visitor Mode/);

        // 3. Host promotes Visitor
        await hostPage.click('#toggle-participants-btn');

        // Ensure participants panel is actually shown (not hidden by class)
        await expect(hostPage.locator('#participants-panel')).not.toHaveClass(/panel-hidden/, { timeout: 10000 });

        // Wait for visitor to appear in list
        const visitorItem = hostPage.locator('.participant-item:has-text("Visitor User")');
        await expect(visitorItem).toBeVisible({ timeout: 20000 });

        const promoteBtn = visitorItem.getByTitle('Promote to Participant');
        // Sometimes elements are "hidden" because they are during transition or have zero size
        // Use dispatchEvent('click') for elements that are outside viewport or hidden
        await expect(promoteBtn).toBeAttached({ timeout: 10000 });
        await promoteBtn.dispatchEvent('click');

        // Verify Visitor is promoted
        await expect(visitorPage.locator('#toggle-camera-btn')).toBeVisible();
        await expect(visitorPage.locator('#toggle-mic-btn')).toBeVisible();
        await expect(visitorPage.locator('#chat-input')).not.toHaveAttribute('placeholder', /Visitor Mode/);

        // 4. Host toggles E2EE
        await hostPage.click('#settings-btn');
        await hostPage.click('button:has-text("Moderator")');
        const e2eeToggle = hostPage.locator('#e2ee-toggle');
        await e2eeToggle.check();
        await hostPage.click('#close-settings-btn');

        // Verify E2EE visual indicator for both (one in participant list, one in video card per participant)
        await expect(hostPage.locator('.e2ee-lock')).toHaveCount(4);
        await expect(visitorPage.locator('.e2ee-lock')).toHaveCount(4);

        // 5. Layout synchronization (Follow Me)
        // Switch to Spotlight
        await hostPage.click('button:has-text("Switch to Spotlight")');
        await expect(hostPage.locator('.video-grid.spotlight')).toBeVisible();

        // Verify Visitor's layout updated automatically
        await expect(visitorPage.locator('.video-grid.spotlight')).toBeVisible();

        // Switch back to Grid
        await hostPage.click('button:has-text("Switch to Grid")');
        await expect(hostPage.locator('.video-grid.grid')).toBeVisible();
        await expect(visitorPage.locator('.video-grid.grid')).toBeVisible();

        // 6. Integrated Features check
        // Chat
        await visitorPage.click('#toggle-chat-btn');
        // Ensure host also has chat open to see the message
        await hostPage.click('#toggle-chat-btn');
        await expect(hostPage.locator('#chat-panel')).not.toHaveClass(/panel-hidden/);

        await visitorPage.fill('#chat-input', 'Hello from promoted visitor');
        await visitorPage.press('#chat-input', 'Enter');
        await expect(hostPage.locator('.chat-message:has-text("Hello from promoted visitor")')).toBeVisible({ timeout: 10000 });

        // Polls
        await hostPage.click('#toggle-polls-btn');
        await hostPage.click('button:has-text("Create Poll")');
        await hostPage.fill('#poll-question', 'Is this working?');
        await hostPage.fill('#poll-option-1', 'Yes');
        await hostPage.fill('#poll-option-2', 'Definitely');
        await hostPage.click('#create-poll-submit-btn');

        await visitorPage.click('#toggle-polls-btn');
        const pollItem = visitorPage.locator('.poll-item:has-text("Is this working?")');
        await expect(pollItem).toBeVisible({ timeout: 10000 });
        await pollItem.getByRole('button', { name: 'Vote' }).first().click();
        await expect(hostPage.locator('.poll-item:has-text("Is this working?")')).toContainText('1 votes', { timeout: 10000 });

        // Close polls dialog before proceeding to whiteboard to avoid intercepting clicks
        await hostPage.click('#close-polls-btn');
        await expect(hostPage.locator('.modal-overlay')).not.toBeVisible();

        // Whiteboard
        await hostPage.click('#toggle-whiteboard-btn');
        await expect(hostPage.locator('canvas')).toBeVisible();
    });
});
