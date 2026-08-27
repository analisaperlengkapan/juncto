import { expect, test } from '@playwright/test';

test.describe('Responsive layouts at breakpoints', () => {
    test.beforeEach(async ({ page, request }) => {
        await request.post('http://localhost:3000/api/rooms', {
            data: { room_name: 'RespRoom', is_locked: false, is_recording: false, is_lobby_enabled: false, max_participants: 10 },
        });
        await page.goto('/room/RespRoom');
        await page.fill('.prejoin-container input[type="text"]', 'RespTest');
        await page.click('button.join-btn');
        await expect(page.locator('.room-container')).toBeVisible();
    });

    // Phone 480px → video grid stacks vertically
    test('phone viewport (480px) stacks video grid vertically', async ({ page }) => {
        await page.setViewportSize({ width: 480, height: 800 });
        const grid = page.locator('.video-grid');
        await expect(grid).toBeVisible();

        const direction = await grid.evaluate(el => getComputedStyle(el).flexDirection);
        expect(direction).toBe('column');
    });

    // Tablet landscape 768px → side panel compresses
    test('tablet viewport (768px) compresses side panel', async ({ page }) => {
        await page.setViewportSize({ width: 768, height: 1024 });
        const panel = page.locator('#participants-panel');
        const width = await panel.evaluate(el => el.getBoundingClientRect().width);
        expect(width).toBeLessThanOrEqual(282);
    });

    // Desktop ≥769px → full side panel width
    test('desktop viewport (769px+) uses full 320px side panel', async ({ page }) => {
        await page.setViewportSize({ width: 1280, height: 720 });

        const panel = page.locator('#participants-panel');
        const width = await panel.evaluate(el => el.getBoundingClientRect().width);
        expect(width).toBeGreaterThanOrEqual(300);
    });
});
