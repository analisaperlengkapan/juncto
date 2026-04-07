import { test, expect } from '@playwright/test';

test.describe('Talk While Muted Feature', () => {
    test('Should display toast when user talks while muted (real audio simulation)', async ({ page, context }) => {
        await context.grantPermissions(['camera', 'microphone']);

        await page.addInitScript(() => {
            const originalGetUserMedia = navigator.mediaDevices.getUserMedia.bind(navigator.mediaDevices);
            navigator.mediaDevices.getUserMedia = async (constraints) => {
                if (constraints && constraints.audio) {
                    // We must wait for user gesture or audio context gets suspended sometimes,
                    // but in playwright headless it's usually allowed.
                    const audioCtx = new (window.AudioContext || (window as any).webkitAudioContext)();
                    const oscillator = audioCtx.createOscillator();
                    oscillator.type = 'square';
                    const gainNode = audioCtx.createGain();
                    gainNode.gain.value = 1.0;

                    oscillator.frequency.setValueAtTime(440, audioCtx.currentTime); // 440Hz
                    oscillator.start();

                    const dest = audioCtx.createMediaStreamDestination();
                    oscillator.connect(gainNode);
                    gainNode.connect(dest);

                    const fakeAudioTrack = dest.stream.getAudioTracks()[0];

                    let videoTrack;
                    if (constraints.video) {
                         const realStream = await originalGetUserMedia({video: true, audio: false});
                         videoTrack = realStream.getVideoTracks()[0];
                    }

                    const finalStream = new MediaStream();
                    if (fakeAudioTrack) finalStream.addTrack(fakeAudioTrack);
                    if (videoTrack) finalStream.addTrack(videoTrack);

                    return finalStream;
                }
                return originalGetUserMedia(constraints);
            };
        });

        const roomName = `TalkMuted_${Date.now()}`;
        await page.goto(`/room/${roomName}`);

        await page.locator('.prejoin-container input[type="text"]').fill('MutedUser');

        // Wait for prejoin screen to init streams and potentially the mic button to become active
        // The text might be "Turn Mic Off" or "Turn Off Mic" or just "Mic On/Off"
        const micBtn = page.locator('button:has-text("Turn Off Mic")');
        // Let's use a soft wait, maybe it's not visible or audio failed
        try {
            await micBtn.waitFor({ state: 'visible', timeout: 3000 });
            await micBtn.click();
        } catch (e) {
             console.log("Mic button wait failed");
        }

        await page.click('button:has-text("Join Meeting")');
        await expect(page.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

        // Once in the meeting, ensure we are muted
        const toolboxMuteBtn = page.locator('.room-toolbox button').filter({ hasText: /^Unmute$/ });
        if (!(await toolboxMuteBtn.isVisible())) {
             const muteBtn = page.locator('button').filter({ hasText: /^Mute$/ });
             if (await muteBtn.isVisible()) await muteBtn.click();
        }

        const toast = page.locator('.toast');
        await expect(toast).toContainText('You are muted. Please unmute to speak.', { timeout: 10000 });
    });
});
