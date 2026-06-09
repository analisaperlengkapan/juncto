import { test, expect } from '@playwright/test';

async function loginAsAdmin(page) {
    await page.click('button[title="Login"]');
    await page.fill('input[placeholder="Username"]', 'admin');
    await page.fill('input[placeholder="Password"]', 'admin123');
    await page.click('button:has-text("Login")');
}

test.describe('Shared Video', () => {
    test('Host can share a video', async ({ browser }) => {
        const roomName = `VideoRoom_${Date.now()}`;
        const contextA = await browser.newContext();
        const pageA = await contextA.newPage();
        await pageA.goto('/');

        // User A (Host)
        await pageA.fill('#meeting-name', roomName);
        await pageA.click('.create-btn');
        await pageA.fill('#display-name', 'Alice');
        await pageA.click('.join-btn');
        await loginAsAdmin(pageA);

        // Click Share Video
        await pageA.click('button:has-text("Video")');
        await pageA.locator('input[placeholder*="youtube.com"]').fill('https://www.youtube.com/watch?v=dQw4w9WgXcQ');
        await pageA.locator('#submit-shared-video-btn').click();

        // Verify Video Card appears
        await expect(pageA.locator('.shared-video')).toBeVisible();

        // User B joins
        const contextB = await browser.newContext();
        const pageB = await contextB.newPage();
        await pageB.goto(`http://localhost:3000/room/${roomName}`);
        await pageB.fill('#display-name', 'Bob');
        await pageB.click('.join-btn');

        // Verify Video Card appears for Bob
        await expect(pageB.locator('.shared-video')).toBeVisible();

        // Host Stops Video
        await pageA.click('button:has-text("Stop Video")');

        // Verify removed
        await expect(pageA.locator('.shared-video')).not.toBeVisible();
    });
});
