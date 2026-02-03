import { test, expect } from '@playwright/test';
import fs from 'fs';

test.describe('File Sharing', () => {
    test('User can upload and share a file', async ({ browser }) => {
        const roomName = `FileRoom_${Date.now()}`;
        const contextA = await browser.newContext();
        const pageA = await contextA.newPage();
        await pageA.goto('/');

        // User A (Host)
        await pageA.fill('input[type="text"]', roomName);
        await pageA.click('button:has-text("Start Meeting")');
        await pageA.locator('.prejoin-container input[type="text"]').fill('Alice');
        await pageA.click('button:has-text("Join Meeting")');
        await expect(pageA.locator('.video-grid')).toBeVisible();

        // User B
        const contextB = await browser.newContext();
        const pageB = await contextB.newPage();
        await pageB.goto(`/room/${roomName}`);
        await pageB.locator('.prejoin-container input[type="text"]').fill('Bob');
        await pageB.click('button:has-text("Join Meeting")');
        await expect(pageB.locator('.video-grid')).toBeVisible();

        // Create a dummy file
        const filePath = 'test-file.txt';
        fs.writeFileSync(filePath, 'Hello World Content');

        // User A uploads file
        // Input type file is in the chat area.
        await pageA.setInputFiles('input[type="file"]', filePath);

        // Expect "Selected: test-file.txt"
        await expect(pageA.locator('text=Selected: test-file.txt')).toBeVisible();

        // Send
        // There are multiple buttons (Send for chat).
        // It's in the chat container.
        await pageA.click('.chat-container button:has-text("Send")');

        // Expect file link in chat for both
        // The link text format is "📎 test-file.txt"
        await expect(pageA.locator('a:has-text("test-file.txt")')).toBeVisible();
        await expect(pageB.locator('a:has-text("test-file.txt")')).toBeVisible();

        // Cleanup
        fs.unlinkSync(filePath);
        await contextA.close();
        await contextB.close();
    });
});
