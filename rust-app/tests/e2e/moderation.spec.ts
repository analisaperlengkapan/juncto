import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    await page.click('button[title="Login"]');
    await page.fill('input[placeholder="Username"]', 'admin');
    await page.fill('input[placeholder="Password"]', 'admin123');
    await page.click('button:has-text("Login")');
}

test.describe('Moderation Controls', () => {
    test('Host can mute a participant', async ({ context }) => {
        const pageA = await context.newPage();
        await pageA.goto('http://localhost:3000/room/ModTest');
        await pageA.fill('#display-name', 'User A');
        await pageA.click('.join-btn');
        await loginAsAdmin(pageA);

        const pageB = await context.newPage();
        await pageB.goto('http://localhost:3000/room/ModTest');
        await pageB.fill('#display-name', 'User B');
        await pageB.click('.join-btn');

        await pageA.click('#toggle-participants-btn');
        const userBRow = pageA.locator('.participant-item:has-text("User B")');
        const muteBtn = userBRow.getByRole('button', { name: 'Mute', exact: true });
        await muteBtn.click();

        // Verify User B is muted
        await expect(muteBtn).not.toBeVisible();
        await expect(userBRow.locator('span:has-text("🔇")')).toBeVisible();
    });

    test('Host can transfer host role', async ({ context }) => {
        const pageA = await context.newPage();
        await pageA.goto('http://localhost:3000/room/HostTransfer');
        await pageA.fill('#display-name', 'User A');
        await pageA.click('.join-btn');
        await loginAsAdmin(pageA);

        const pageB = await context.newPage();
        await pageB.goto('http://localhost:3000/room/HostTransfer');
        await pageB.fill('#display-name', 'User B');
        await pageB.click('.join-btn');

        await pageA.click('#toggle-participants-btn');
        const userBRow = pageA.locator('.participant-item:has-text("User B")');
        await userBRow.getByRole('button', { name: 'Transfer Host' }).click();

        // Verify Transfer
        await expect(pageB.locator('button:has-text("End Meeting")')).toBeVisible();
        await expect(pageA.locator('button:has-text("End Meeting")')).not.toBeVisible();
    });
});
