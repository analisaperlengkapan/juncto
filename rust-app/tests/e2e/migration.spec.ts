import { test, expect } from '@playwright/test';

test('Juncto Migration E2E (WASM)', async ({ page, request }) => {
  // Clear room config to ensure clean state
  await request.post('http://localhost:3000/api/rooms', {
    data: {
      room_name: "Rust Meeting",
      is_locked: false,
      is_recording: false,
      is_lobby_enabled: false,
      max_participants: 100
    }
  });

  page.on('console', msg => console.log('BROWSER LOG:', msg.text()));
  page.on('pageerror', err => console.log('BROWSER ERROR:', err));

  // 1. Check API Health
  const healthResponse = await request.get('http://localhost:3000/health');
  expect(healthResponse.status()).toBe(200);

  // 2. Load Frontend
  await page.goto('/');
  await expect(page).toHaveTitle(/Juncto/);

  // Wait for WASM to hydrate and show content
  await expect(page.getByText('Welcome to Juncto (Rust Edition)')).toBeVisible({ timeout: 10000 });

  // 3. Interact (Start Meeting)
  const input = page.locator('input[type="text"]');
  await expect(input).toBeVisible();
  await input.fill('Rust Meeting');
  await page.click('button.create-btn');

  // 4. Prejoin Screen
  // "Rust Meeting" gets encoded to "Rust%20Meeting"
  await expect(page).toHaveURL(/\/room\/Rust%20Meeting/);
  await expect(page.getByText('Ready to join?')).toBeVisible();

  // Enter Name and Join
  const nameInput = page.locator('.prejoin-container input[type="text"]');
  await nameInput.fill('E2E User');
  await page.click('button.join-btn');

  // 5. Verify Room UI
  // The component likely displays the decoded parameter
  await expect(page.getByText('Meeting Room: Rust Meeting')).toBeVisible();

  // 5. Verify Chat Functionality
  const chatInput = page.locator('.chat-container input[type="text"]');
  const chatSendBtn = page.locator('.chat-container button');

  await expect(chatInput).toBeVisible();

  // Wait for connection
  await expect(chatSendBtn).toBeEnabled();
  await expect(chatSendBtn).toHaveText('Send');

  await chatInput.fill('Hello from E2E');
  await chatSendBtn.click();

  // Verify message appears (User ID "Me" is hardcoded in chat.rs)
  // Check for the content first as it's most unique
  await expect(page.getByText('Hello from E2E')).toBeVisible();

  // 6. Verify Participants List
  const participantsList = page.locator('.participants-list');
  await expect(participantsList).toBeVisible();
  // Should contain at least "User ..." because backend assigns random names starting with "User"
  // Actually, E2E User joins, so it should be E2E User.
  // Wait, backend assigns random name? In my implementation, I assign name from ClientMessage.
  // BUT the first user is Host.
  await expect(participantsList.locator('ul')).toContainText('E2E User');

  // 7. Verify Room Lock
  // Wait for host status to be synced
  // Because "E2E User" is the first one in this fresh room, they are the Host.
  const lockBtn = page.getByRole('button', { name: 'Lock Room' });
  await expect(lockBtn).toBeVisible({ timeout: 10000 });
  await lockBtn.click();

  // Verify button text changes to "Unlock Room"
  await expect(page.getByRole('button', { name: 'Unlock Room' })).toBeVisible();

  // 8. Verify Settings / Profile Update
  // Open Settings
  await page.getByRole('button', { name: 'Settings' }).click();
  await expect(page.getByText('Save Profile')).toBeVisible();

  // Change Name
  const nameSettingInput = page.locator('.modal-content input[type="text"]');
  await nameSettingInput.fill('Updated Name');
  await page.click('button:has-text("Save Profile")');

  // Verify modal closed
  await expect(page.getByText('Save Profile')).not.toBeVisible();

  // Verify name updated in Participants List
  // Ideally we wait for the update
  await expect(participantsList.locator('ul')).toContainText('Updated Name');

  // 9. Verify Reactions
  const likeBtn = page.getByRole('button', { name: '👍' });
  await expect(likeBtn).toBeVisible();
  await likeBtn.click();

  // Verify reaction appears in the overlay
  // Note: Animation lasts 2s, so we must be quick or just check existence
  await expect(page.locator('.reaction-layer')).toContainText('👍');

  // 10. Verify Recording
  const recordBtn = page.getByRole('button', { name: 'Start Recording' });
  await expect(recordBtn).toBeVisible();
  await recordBtn.click();

  // Verify REC indicator
  await expect(page.getByText('REC', { exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Stop Recording' })).toBeVisible();

  // Stop Recording
  await page.getByRole('button', { name: 'Stop Recording' }).click();
  await expect(page.getByText('REC', { exact: true })).not.toBeVisible();

  // 11. Verify Polls
  // Open Polls
  await page.getByRole('button', { name: 'Polls' }).click();
  await expect(page.getByRole('heading', { name: 'Polls' })).toBeVisible();

  // Create Poll
  await page.getByRole('button', { name: 'Create Poll' }).click(); // Click Tab

  const pollForm = page.locator('.modal-content .tab-content');
  await expect(pollForm).toBeVisible();

  await pollForm.locator('input').nth(0).fill('Fav Color?'); // Question
  await pollForm.locator('input').nth(1).fill('Red'); // Option 1
  await pollForm.locator('input').nth(2).fill('Blue'); // Option 2

  // Click the 'Create Poll' button inside the tab content (the submit button)
  await pollForm.locator('button:has-text("Create Poll")').click();

  // Verify Poll Created
  await expect(page.getByText('Fav Color?')).toBeVisible();
  await expect(page.getByText('0 votes')).toHaveCount(2);

  // Vote
  await page.locator('button:has-text("Vote")').first().click();

  // Verify Vote Count Updated
  await expect(page.getByText('1 votes')).toBeVisible();

  // Close Polls Dialog
  await page.locator('.modal-header button').click(); // Close button "×"
  await expect(page.getByRole('heading', { name: 'Polls' })).not.toBeVisible();

  // 12. Verify Raise Hand
  const handBtn = page.getByRole('button', { name: 'Raise Hand' });
  await expect(handBtn).toBeVisible();
  await handBtn.click();

  // Verify hand icon in participants list
  // Ideally, find the list item for "E2E User" or "User ..." and check for hand emoji
  await expect(page.locator('.participants-list li').filter({ hasText: 'Updated Name' })).toContainText('✋');

  // Lower hand
  await handBtn.click();
  // Verify hand icon removed (might need short wait or check lack of text)
  await expect(page.locator('.participants-list li').filter({ hasText: 'Updated Name' })).not.toContainText('✋');

  // 13. Verify Screen Share
  const screenBtn = page.getByRole('button', { name: 'Share Screen' });
  await expect(screenBtn).toBeVisible();
  await screenBtn.click();

  // Verify screen icon in participants list
  await expect(page.locator('.participants-list li').filter({ hasText: 'Updated Name' })).toContainText('🖥️');

  // 14. Verify Whiteboard
  const wbBtn = page.getByRole('button', { name: 'Whiteboard' });
  await expect(wbBtn).toBeVisible();
  await wbBtn.click();

  const canvas = page.locator('canvas');
  await expect(canvas).toBeVisible();

  // Simulate drawing
  const box = await canvas.boundingBox();
  if (box) {
      await page.mouse.move(box.x + 10, box.y + 10);
      await page.mouse.down();
      await page.mouse.move(box.x + 50, box.y + 50);
      await page.mouse.up();
  }

  // Close Whiteboard
  await wbBtn.click();
  await expect(canvas).not.toBeVisible();

  // Unlock the room to reset state for next test
  await page.getByRole('button', { name: 'Unlock Room' }).click();
  await expect(page.getByRole('button', { name: 'Lock Room' })).toBeVisible();
});

test('Lobby Feature E2E', async ({ browser }) => {
  // Scenario:
  // 1. Host creates room, enables Lobby.
  // 2. Guest attempts to join, sees Waiting Screen.
  // 3. Host sees Guest knocking, grants access.
  // 4. Guest enters room.

  // Host context
  const hostContext = await browser.newContext();
  const hostPage = await hostContext.newPage();

  // Guest context
  const guestContext = await browser.newContext();
  const guestPage = await guestContext.newPage();

  const roomName = `LobbyTestRoom_${Date.now()}`;

  // --- HOST ---
  await hostPage.goto('/');
  await hostPage.locator('input[type="text"]').fill(roomName);
  await hostPage.click('button.create-btn');

  await hostPage.locator('.prejoin-container input[type="text"]').fill('Host');
  await hostPage.click('button.join-btn');
  await expect(hostPage.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

  // Enable Lobby
  await hostPage.getByRole('button', { name: 'Enable Lobby' }).click();
  await expect(hostPage.getByRole('button', { name: 'Disable Lobby' })).toBeVisible();

  // --- GUEST ---
  // Need to get the room URL (encoded)
  const roomUrl = hostPage.url();
  await guestPage.goto(roomUrl);
  await guestPage.locator('.prejoin-container input[type="text"]').fill('Guest');
  await guestPage.click('button.join-btn');

  // Verify Guest sees Lobby/Waiting Screen
  await expect(guestPage.getByText('Waiting for host...')).toBeVisible();

  // --- HOST ---
  // Verify Host sees Guest knocking
  await expect(hostPage.locator('.knocking-list')).toBeVisible();
  await expect(hostPage.locator('.knocking-list')).toContainText('Guest');

  // Host allows Guest
  // Use .first() to avoid ambiguity if multiple elements match or if previous tests left artifacts
  await hostPage.locator('.knocking-list li').filter({ hasText: 'Guest' }).getByRole('button', { name: 'Allow' }).click();

  // --- GUEST ---
  // Verify Guest enters room
  await expect(guestPage.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

  // Cleanup
  await hostContext.close();
  await guestContext.close();
});

test('Max Participants E2E', async ({ browser, request }) => {
  // 1. Create Room via API with max_participants = 1
  const createRes = await request.post('http://localhost:3000/api/rooms', {
    data: {
      room_name: "FullRoom",
      is_locked: false,
      is_recording: false,
      is_lobby_enabled: false,
      max_participants: 1
    }
  });
  expect(createRes.status()).toBe(201);
  const roomData = await createRes.json();
  const roomName = roomData.config.room_name;

  // 2. User 1 Joins
  const user1Context = await browser.newContext();
  const user1Page = await user1Context.newPage();

  await user1Page.goto(`/room/${encodeURIComponent(roomName)}`);
  await user1Page.locator('.prejoin-container input[type="text"]').fill('User1');
  await user1Page.click('button.join-btn');
  await expect(user1Page.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

  // 3. User 2 Tries to Join (should fail or hang in prejoin or show alert)
  const user2Context = await browser.newContext();
  const user2Page = await user2Context.newPage();

  // Handling alert:
  user2Page.on('dialog', async dialog => {
    expect(dialog.message()).toContain('Room is full');
    await dialog.accept();
  });

  await user2Page.goto(`/room/${encodeURIComponent(roomName)}`);
  await user2Page.locator('.prejoin-container input[type="text"]').fill('User2');
  await user2Page.click('button.join-btn');

  // Verify User 2 is NOT in the room
  // Wait a bit to ensure it didn't transition
  await user2Page.waitForTimeout(1000);
  await expect(user2Page.getByText(`Meeting Room: ${roomName}`)).not.toBeVisible();

  // Verify User 1 sees no User 2
  await expect(user1Page.locator('.participants-list')).not.toContainText('User2');

  await user1Context.close();
  await user2Context.close();
});

test('Chat History E2E', async ({ browser, request }) => {
  // Reset config to allow multiple participants
  await request.post('http://localhost:3000/api/rooms', {
    data: {
      room_name: "HistoryRoom",
      is_locked: false,
      is_recording: false,
      is_lobby_enabled: false,
      max_participants: 100
    }
  });

  // 1. User 1 creates room and chats
  const context1 = await browser.newContext();
  const page1 = await context1.newPage();

  await page1.goto('/room/HistoryRoom');
  await page1.locator('.prejoin-container input[type="text"]').fill('Chatter1');
  await page1.click('button.join-btn');

  await expect(page1.getByText('Meeting Room: HistoryRoom')).toBeVisible();

  const chatInput1 = page1.locator('.chat-container input[type="text"]');
  await chatInput1.fill('Message before join');
  await page1.click('.chat-container button');
  await expect(page1.getByText('Message before join')).toBeVisible();

  // 2. User 2 joins and should see the message
  const context2 = await browser.newContext();
  const page2 = await context2.newPage();

  // Use same room URL
  const roomUrl = page1.url();
  await page2.goto(roomUrl);
  await page2.locator('.prejoin-container input[type="text"]').fill('Chatter2');
  await page2.click('button.join-btn');

  await expect(page2.getByText('Meeting Room: HistoryRoom')).toBeVisible();

  // Verify Chat History
  await expect(page2.locator('.chat-container')).toContainText('Message before join');

  await context1.close();
  await context2.close();
});

test('Typing Indicator E2E', async ({ browser, request }) => {
  const roomName = 'TypingRoom';
  // Reset config
  await request.post('http://localhost:3000/api/rooms', {
    data: {
      room_name: roomName,
      is_locked: false,
      is_recording: false,
      is_lobby_enabled: false,
      max_participants: 100
    }
  });

  const context1 = await browser.newContext();
  const page1 = await context1.newPage();
  const context2 = await browser.newContext();
  const page2 = await context2.newPage();

  // User 1 Join
  await page1.goto(`/room/${roomName}`);
  await page1.locator('.prejoin-container input[type="text"]').fill('User1');
  await page1.click('button.join-btn');
  await expect(page1.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

  // User 2 Join
  await page2.goto(`/room/${roomName}`);
  await page2.locator('.prejoin-container input[type="text"]').fill('User2');
  await page2.click('button.join-btn');
  await expect(page2.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

  // User 1 types
  const chatInput1 = page1.locator('.chat-container input[type="text"]');
  await chatInput1.type('Some text'); // 'type' simulates keystrokes

  // User 2 should see indicator
  // The typing indicator should appear. We check for the suffix because the name logic is complex.
  // Wait explicitly to ensure the websocket message propagates
  await expect(page2.locator('.typing-indicator')).toContainText('is typing...', { timeout: 10000 });

  // User 1 stops typing (wait 4s for timeout > 3000ms)
  // Or force stop by sending message
  await page1.click('.chat-container button'); // Send

  // User 2 should NOT see indicator
  await expect(page2.locator('.typing-indicator')).not.toContainText('is typing...');

  await context1.close();
  await context2.close();
});

test('Kick Participant E2E', async ({ browser, request }) => {
  const roomName = 'KickRoom';
  await request.post('http://localhost:3000/api/rooms', {
    data: {
      room_name: roomName,
      is_locked: false,
      is_recording: false,
      is_lobby_enabled: false,
      max_participants: 100
    }
  });

  // Host
  const hostContext = await browser.newContext();
  const hostPage = await hostContext.newPage();
  await hostPage.goto(`/room/${roomName}`);
  await hostPage.locator('.prejoin-container input[type="text"]').fill('Host');
  await hostPage.click('button.join-btn');
  await expect(hostPage.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

  // Guest
  const guestContext = await browser.newContext();
  const guestPage = await guestContext.newPage();
  await guestPage.goto(`/room/${roomName}`);
  await guestPage.locator('.prejoin-container input[type="text"]').fill('Guest');
  await guestPage.click('button.join-btn');
  await expect(guestPage.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

  // Host should see "Kick" button for Guest
  // Note: Guest item text contains "Guest"
  const guestItem = hostPage.locator('.participants-list li').filter({ hasText: 'Guest' });
  await expect(guestItem.getByRole('button', { name: 'Kick' })).toBeVisible();

  // Handle alert on guest page
  guestPage.on('dialog', async dialog => {
    expect(dialog.message()).toContain('kicked');
    await dialog.accept();
  });

  // Host Kicks Guest
  await guestItem.getByRole('button', { name: 'Kick' }).click();

  // Guest should be redirected to Prejoin
  await expect(guestPage.getByText('Ready to join?')).toBeVisible();

  // Host list should not have Guest
  await expect(hostPage.locator('.participants-list')).not.toContainText('Guest');

  await hostContext.close();
  await guestContext.close();
});

test('Room Lock E2E', async ({ browser, request }) => {
  const roomName = 'LockRoom';
  await request.post('http://localhost:3000/api/rooms', {
    data: {
      room_name: roomName,
      is_locked: false,
      is_recording: false,
      is_lobby_enabled: false,
      max_participants: 100
    }
  });

  // Host
  const hostContext = await browser.newContext();
  const hostPage = await hostContext.newPage();
  await hostPage.goto(`/room/${roomName}`);
  await hostPage.locator('.prejoin-container input[type="text"]').fill('Host');
  await hostPage.click('button.join-btn');
  await expect(hostPage.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

  // Guest
  const guestContext = await browser.newContext();
  const guestPage = await guestContext.newPage();
  await guestPage.goto(`/room/${roomName}`);
  await guestPage.locator('.prejoin-container input[type="text"]').fill('Guest');
  await guestPage.click('button.join-btn');
  await expect(guestPage.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

  // 1. Initial State: Unlocked
  // Wait for host status propagation (RoomUpdated -> set_room_config -> is_host derived)
  await expect(hostPage.getByRole('button', { name: 'Lock Room' })).toBeVisible({ timeout: 10000 });
  await expect(guestPage.getByText('Unlocked')).toBeVisible();
  await expect(guestPage.getByRole('button', { name: 'Lock Room' })).not.toBeVisible();

  // 2. Host Locks Room
  await hostPage.getByRole('button', { name: 'Lock Room' }).click();
  await expect(hostPage.getByRole('button', { name: 'Unlock Room' })).toBeVisible();

  // 3. Guest sees Locked state
  await expect(guestPage.getByText('Locked')).toBeVisible();
  await expect(guestPage.getByRole('button', { name: 'Unlock Room' })).not.toBeVisible();

  // 4. Host Unlocks Room
  await hostPage.getByRole('button', { name: 'Unlock Room' }).click();
  await expect(hostPage.getByRole('button', { name: 'Lock Room' })).toBeVisible();
  await expect(guestPage.getByText('Unlocked')).toBeVisible();

  await hostContext.close();
  await guestContext.close();
});

test('Breakout Rooms E2E', async ({ browser, request }) => {
  const roomName = 'BreakoutMain';
  await request.post('http://localhost:3000/api/rooms', {
    data: {
      room_name: roomName,
      is_locked: false,
      is_recording: false,
      is_lobby_enabled: false,
      max_participants: 100
    }
  });

  const hostContext = await browser.newContext();
  const hostPage = await hostContext.newPage();
  const guestContext = await browser.newContext();
  const guestPage = await guestContext.newPage();

  // Both join main room
  await hostPage.goto(`/room/${roomName}`);
  await hostPage.locator('.prejoin-container input[type="text"]').fill('Host');
  await hostPage.click('button.join-btn');
  await expect(hostPage.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

  await guestPage.goto(`/room/${roomName}`);
  await guestPage.locator('.prejoin-container input[type="text"]').fill('Guest');
  await guestPage.click('button.join-btn');
  await expect(guestPage.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

  // Host creates breakout room
  await hostPage.locator('input[placeholder="New Room Name"]').fill('Team A');
  await hostPage.getByRole('button', { name: 'Create' }).click();

  // Both should see new room listed
  await expect(hostPage.getByText('Team A')).toBeVisible();
  await expect(guestPage.getByText('Team A')).toBeVisible();

  // Host joins breakout room
  // Find the 'Join' button specifically for 'Team A'
  // Simplified: assuming only one room created
  await hostPage.getByRole('button', { name: 'Join' }).click();

  // Host UI should update
  await expect(hostPage.getByText('(In Breakout Room)')).toBeVisible();
  await expect(hostPage.getByText('Team A')).toBeVisible();

  // Guest stays in Main
  await expect(guestPage.getByText('(In Breakout Room)')).not.toBeVisible();

  // Host chats in Breakout
  await hostPage.locator('.chat-container input[type="text"]').fill('Secret Message');
  await hostPage.click('.chat-container button'); // Send

  // Guest should NOT see it
  await expect(guestPage.locator('.chat-container')).not.toContainText('Secret Message');

  // Guest chats in Main
  await guestPage.locator('.chat-container input[type="text"]').fill('Main Message');
  await guestPage.click('.chat-container button'); // Send

  // Host should NOT see it (in real app they might, but current logic filters strict room match)
  await expect(hostPage.locator('.chat-container')).not.toContainText('Main Message');

  // Host returns to Main
  await hostPage.getByRole('button', { name: 'Return to Main' }).click();
  await expect(hostPage.getByText('(In Breakout Room)')).not.toBeVisible();

  // Now Host should see/receive messages in main (if we resent or typed new ones)
  // Let's type a new one
  await hostPage.locator('.chat-container input[type="text"]').fill('Back in Main');
  await hostPage.click('.chat-container button'); // Send

  await expect(guestPage.locator('.chat-container')).toContainText('Back in Main');

  await hostContext.close();
  await guestContext.close();
});

test('Device Settings E2E', async ({ page }) => {
    page.on('console', msg => console.log('PAGE LOG:', msg.text()));
    page.on('pageerror', exception => console.log(`PAGE ERROR: "${exception}"`));

    // Join room
    await page.goto('/');
    await page.fill('input[type="text"]', 'Device Test Room');
    await page.click('button:has-text("Start Meeting")');
    await page.waitForURL(/\/room\//);
    await page.locator('.prejoin-container input[type="text"]').fill('Tester');
    await page.click('button:has-text("Join Meeting")');

    // Open Settings
    await page.click('button:has-text("Settings")');
    await page.click('button:has-text("Devices")');

    // Check for Camera and Mic selectors
    await expect(page.locator('label:has-text("Camera")')).toBeVisible();
    await expect(page.locator('label:has-text("Microphone")')).toBeVisible();

    // Check for video preview
    const video = page.locator('div.preview video');
    await expect(video).toBeVisible();

    // With fake devices, we should have options
    // Wait for enumeration (simple wait or retry assertion)
    await expect(async () => {
        const camOptions = await page.locator('select').first().locator('option').count();
        expect(camOptions).toBeGreaterThan(0);
    }).toPass();
});

test('Video Grid E2E', async ({ page }) => {
    // Join room
    await page.goto('/');
    await page.fill('input[type="text"]', 'Video Room');
    await page.click('button:has-text("Start Meeting")');
    await page.waitForURL(/\/room\//);
    await page.locator('.prejoin-container input[type="text"]').fill('Viewer');
    await page.click('button:has-text("Join Meeting")');

    // Check Video Grid exists
    const grid = page.locator('.video-grid');
    await expect(grid).toBeVisible();

    // Check Local Video card exists ("Me")
    await expect(grid.locator('.video-card:has-text("Me")')).toBeVisible();

    // Check camera toggle logic
    // Initially off
    await expect(grid.locator('.video-card:has-text("Camera Off")')).toBeVisible();

    // Toggle On
    await page.click('button:has-text("Toggle Camera")');
    // "Camera Off" should disappear, video element should be visible
    await expect(grid.locator('.video-card:has-text("Camera Off")')).not.toBeVisible();
    await expect(grid.locator('video')).toBeVisible();

    // Toggle Off
    await page.click('button:has-text("Toggle Camera")');
    await expect(grid.locator('.video-card:has-text("Camera Off")')).toBeVisible();
});

test('Chat Names E2E', async ({ browser, request }) => {
    // Reset config
    await request.post('http://localhost:3000/api/rooms', {
        data: {
            room_name: "ChatNameRoom",
            is_locked: false,
            is_recording: false,
            is_lobby_enabled: false,
            max_participants: 100
        }
    });

    const context1 = await browser.newContext();
    const page1 = await context1.newPage();
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();

    // User 1 Join
    await page1.goto('/room/ChatNameRoom');
    await page1.locator('.prejoin-container input[type="text"]').fill('Alice');
    await page1.click('button.join-btn');

    // User 2 Join
    await page2.goto('/room/ChatNameRoom');
    await page2.locator('.prejoin-container input[type="text"]').fill('Bob');
    await page2.click('button.join-btn');

    // Alice types
    await page1.locator('.chat-container input[type="text"]').fill('Hi Bob');
    await page1.click('.chat-container button');

    // Alice should see "Me": "Hi Bob"
    // The chat component renders: <strong>{sender_name}": "</strong> <span>{msg.content}</span>
    // The expect might be failing due to exact spacing or HTML structure matching in toContainText
    // Let's check for "Me" and "Hi Bob" separately within the last li.
    const lastMsg = page1.locator('.messages li').last();
    await expect(lastMsg).toContainText('Me');
    await expect(lastMsg).toContainText('Hi Bob');

    // Bob should see "Alice": "Hi Bob"
    const lastMsgBob = page2.locator('.messages li').last();
    await expect(lastMsgBob).toContainText('Alice');
    await expect(lastMsgBob).toContainText('Hi Bob');

    await context1.close();
    await context2.close();
});

test('Screen Share E2E', async ({ page }) => {
    // Join room
    await page.goto('/');
    await page.fill('input[type="text"]', 'Screen Share Room');
    await page.click('button:has-text("Start Meeting")');
    await page.waitForURL(/\/room\//);
    await page.locator('.prejoin-container input[type="text"]').fill('Presenter');
    await page.click('button:has-text("Join Meeting")');

    // Click Share Screen
    // Note: With --use-fake-ui-for-media-stream, this should auto-accept
    await page.click('button:has-text("Share Screen")');

    // Check if "My Screen" card appears
    await expect(page.locator('.video-card:has-text("My Screen")')).toBeVisible();
    await expect(page.locator('.video-card:has-text("My Screen") video')).toBeVisible();

    // Stop Sharing (click again)
    await page.click('button:has-text("Share Screen")');

    // Check if "My Screen" card disappears
    await expect(page.locator('.video-card:has-text("My Screen")')).not.toBeVisible();
});

test('Toast Notification E2E', async ({ browser, request }) => {
    // Scenario: User tries to join a full room.
    // We expect a Toast with "Room is full" instead of an alert.

    const roomName = 'FullRoomToast';
    // 1. Create Room via API with max_participants = 1
    const createRes = await request.post('http://localhost:3000/api/rooms', {
        data: {
            room_name: roomName,
            is_locked: false,
            is_recording: false,
            is_lobby_enabled: false,
            max_participants: 1
        }
    });
    expect(createRes.status()).toBe(201);

    const context = await browser.newContext();
    const page1 = await context.newPage(); // Occupy the spot
    await page1.goto('/room/FullRoomToast');
    await page1.locator('.prejoin-container input[type="text"]').fill('Occupant');
    await page1.click('button.join-btn');
    await expect(page1.getByText('Meeting Room: FullRoomToast')).toBeVisible();

    const page2 = await context.newPage(); // Try to join
    await page2.goto('/room/FullRoomToast');
    await page2.locator('.prejoin-container input[type="text"]').fill('Latecomer');

    // We do NOT expect a dialog/alert anymore.
    // Instead we check for .toast element.
    await page2.click('button.join-btn');

    // Wait for Toast
    await expect(page2.locator('.toast')).toBeVisible();
    await expect(page2.locator('.toast')).toContainText('Room is full');

    await context.close();
});

// Retry toast test with increased timeout or wait logic
test('Toast Notification E2E Retry', async ({ browser, request }) => {
    const roomName = 'FullRoomToastRetry';
    const createRes = await request.post('http://localhost:3000/api/rooms', {
        data: {
            room_name: roomName,
            is_locked: false,
            is_recording: false,
            is_lobby_enabled: false,
            max_participants: 1
        }
    });
    expect(createRes.status()).toBe(201);

    const context = await browser.newContext();
    const page1 = await context.newPage();
    await page1.goto('/room/FullRoomToastRetry');
    await page1.locator('.prejoin-container input[type="text"]').fill('Occupant');
    await page1.click('button.join-btn');
    await expect(page1.getByText('Meeting Room: FullRoomToastRetry')).toBeVisible();

    const page2 = await context.newPage();
    await page2.goto('/room/FullRoomToastRetry');
    await page2.locator('.prejoin-container input[type="text"]').fill('Latecomer');

    // Listen for console logs to debug
    page2.on('console', msg => console.log('PAGE 2 LOG:', msg.text()));

    await page2.click('button.join-btn');

    // Wait specifically for the toast container or toast
    // Increase timeout
    await expect(page2.locator('.toast')).toBeVisible({ timeout: 10000 });
    await expect(page2.locator('.toast')).toContainText('Room is full');

    await context.close();
});

test('Feature Toasts E2E', async ({ page, request }) => {
    // Join room
    await page.goto('/');
    await page.fill('input[type="text"]', 'ToastFeatureRoom');
    await page.click('button:has-text("Start Meeting")');
    await page.waitForURL(/\/room\//);
    await page.locator('.prejoin-container input[type="text"]').fill('FeatureTester');
    await page.click('button:has-text("Join Meeting")');

    // Toggle Recording
    await page.click('button:has-text("Start Recording")');
    // Check Toast
    await expect(page.locator('.toast')).toContainText('Recording Started');
    await expect(page.locator('.toast')).toBeVisible();

    // Wait for it to disappear or click dismiss? Let's toggle back.
    // Wait for animation/timeout or just check next event.
    // The test might be fast enough to see the first toast still there.

    // Toggle Recording Off
    await page.click('button:has-text("Stop Recording")');
    // Check Toast
    // Might be multiple toasts now
    await expect(page.locator('.toast').last()).toContainText('Recording Stopped');

    // Raise Hand
    await page.click('button:has-text("Raise Hand")');
    // Check Toast
    // "FeatureTester raised their hand"
    await expect(page.locator('.toast').last()).toContainText('FeatureTester raised their hand');
});

test('Whiteboard Color E2E', async ({ page, request }) => {
    // Join room
    await request.post('http://localhost:3000/api/rooms', {
        data: {
            room_name: "ColorRoom",
            is_locked: false,
            is_recording: false,
            is_lobby_enabled: false,
            max_participants: 100
        }
    });

    await page.goto('/room/ColorRoom');
    await page.locator('.prejoin-container input[type="text"]').fill('Artist');
    await page.click('button.join-btn');

    // Open Whiteboard
    await page.click('button:has-text("Whiteboard")');
    await expect(page.locator('canvas')).toBeVisible();

    // Check Color Picker
    const colorInput = page.locator('input[type="color"]');
    await expect(colorInput).toBeVisible();
    await expect(colorInput).toHaveValue('#000000');

    // Change color
    await colorInput.fill('#ff0000');
    await expect(colorInput).toHaveValue('#ff0000');

    // Draw something (simulate mouse events)
    const canvas = page.locator('canvas');
    const box = await canvas.boundingBox();
    if (box) {
        await page.mouse.move(box.x + 10, box.y + 10);
        await page.mouse.down();
        await page.mouse.move(box.x + 50, box.y + 50);
        await page.mouse.up();
    }
    // We can't easily verify the pixel color on canvas without screenshot analysis,
    // but verifying the input state change confirms the UI logic works.
});

test('Participant Sorting E2E', async ({ browser, request }) => {
    // Reset config
    await request.post('http://localhost:3000/api/rooms', {
        data: {
            room_name: "SortRoom",
            is_locked: false,
            is_recording: false,
            is_lobby_enabled: false,
            max_participants: 100
        }
    });

    const context1 = await browser.newContext();
    const page1 = await context1.newPage();
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();

    // Host Joins
    await page1.goto('/room/SortRoom');
    await page1.locator('.prejoin-container input[type="text"]').fill('HostUser');
    await page1.click('button.join-btn');

    // Guest Joins
    await page2.goto('/room/SortRoom');
    await page2.locator('.prejoin-container input[type="text"]').fill('GuestUser');
    await page2.click('button.join-btn');

    // Check for (Host) label on HostUser in Host's view
    await expect(page1.locator('.participants-list li').filter({ hasText: 'HostUser' })).toContainText('(Host)');
    // Check for (Host) label on HostUser in Guest's view
    await expect(page2.locator('.participants-list li').filter({ hasText: 'HostUser' })).toContainText('(Host)');

    // Guest raises hand
    await page2.click('button:has-text("Raise Hand")');

    // Verify GuestUser moves to top in Host's view (or above HostUser if alphabetical was Host > Guest, now Guest > Host)
    // "GuestUser" starts with G, "HostUser" with H. Alphabetically G is before H.
    // So GuestUser should be first anyway?
    // Let's create users such that alphabetical order is opposite to raised hand order.
    // Say "Zack" (Guest) and "Adam" (Host).
    // Default: Adam, Zack.
    // Zack raises hand: Zack, Adam.

    // Since we already used HostUser/GuestUser:
    // GuestUser (G) comes before HostUser (H).
    // So they are already G, H.
    // Raising hand keeps G at top.

    // Let's create a new test case or just rename users?
    // Renaming users in Prejoin is easier.
    // Let's retry with specific names.
});

test('Participant Sorting E2E Explicit', async ({ browser, request }) => {
    // Reset config
    await request.post('http://localhost:3000/api/rooms', {
        data: {
            room_name: "SortRoomExplicit",
            is_locked: false,
            is_recording: false,
            is_lobby_enabled: false,
            max_participants: 100
        }
    });

    const context1 = await browser.newContext();
    const page1 = await context1.newPage();
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();

    // "Adam" (Host)
    await page1.goto('/room/SortRoomExplicit');
    await page1.locator('.prejoin-container input[type="text"]').fill('Adam');
    await page1.click('button.join-btn');

    // "Zack" (Guest)
    await page2.goto('/room/SortRoomExplicit');
    await page2.locator('.prejoin-container input[type="text"]').fill('Zack');
    await page2.click('button.join-btn');

    // Initial Order: Adam (Host), Zack
    // We check the text of the first li
    await expect(page1.locator('.participants-list li').first()).toContainText('Adam');

    // Zack raises hand
    await page2.click('button:has-text("Raise Hand")');

    // Expected Order: Zack (Raised), Adam
    await expect(page1.locator('.participants-list li').first()).toContainText('Zack');

    // Zack lowers hand
    await page2.click('button:has-text("Raise Hand")');

    // Expected Order: Adam, Zack
    await expect(page1.locator('.participants-list li').first()).toContainText('Adam');

    await context1.close();
    await context2.close();
});

test('Layout Switching E2E', async ({ page, request }) => {
    // Join room
    await request.post('http://localhost:3000/api/rooms', {
        data: {
            room_name: "LayoutRoom",
            is_locked: false,
            is_recording: false,
            is_lobby_enabled: false,
            max_participants: 100
        }
    });

    await page.goto('/room/LayoutRoom');
    await page.locator('.prejoin-container input[type="text"]').fill('Viewer');
    await page.click('button.join-btn');

    // Check Toggle Button
    const toggleBtn = page.getByRole('button', { name: 'Switch to Spotlight' });
    await expect(toggleBtn).toBeVisible();

    // Verify initial grid layout class or style
    await expect(page.locator('.video-grid')).toHaveClass(/grid/);

    // Switch to Spotlight
    await toggleBtn.click();

    // Verify button text changes
    await expect(page.getByRole('button', { name: 'Switch to Grid' })).toBeVisible();

    // Verify layout class change
    await expect(page.locator('.video-grid')).toHaveClass(/spotlight/);

    // Switch back
    await page.getByRole('button', { name: 'Switch to Grid' }).click();
    await expect(page.getByRole('button', { name: 'Switch to Spotlight' })).toBeVisible();
});

test('Private Messaging E2E', async ({ browser, request }) => {
    // Reset config
    await request.post('http://localhost:3000/api/rooms', {
        data: {
            room_name: "PrivateChatRoom",
            is_locked: false,
            is_recording: false,
            is_lobby_enabled: false,
            max_participants: 100
        }
    });

    const context1 = await browser.newContext();
    const page1 = await context1.newPage();
    const context2 = await browser.newContext();
    const page2 = await context2.newPage();
    const context3 = await browser.newContext();
    const page3 = await context3.newPage();

    // Alice
    await page1.goto('/room/PrivateChatRoom');
    await page1.locator('.prejoin-container input[type="text"]').fill('Alice');
    await page1.click('button.join-btn');

    // Bob
    await page2.goto('/room/PrivateChatRoom');
    await page2.locator('.prejoin-container input[type="text"]').fill('Bob');
    await page2.click('button.join-btn');

    // Eve
    await page3.goto('/room/PrivateChatRoom');
    await page3.locator('.prejoin-container input[type="text"]').fill('Eve');
    await page3.click('button.join-btn');

    // Wait for everyone to join
    // Use more specific locator to avoid strict mode violation
    await expect(page1.locator('.participants-list').getByText('Eve')).toBeVisible();

    // Alice sends private message to Bob
    // Select Bob from dropdown
    // Note: The value of option is user ID. We need to find the option with text "Bob".
    // Or we just pick the second option (0 is Everyone, 1 is Bob/Eve).
    // Let's iterate options to find Bob's ID.
    // Or cleaner: locate by label.

    await page1.locator('select').selectOption({ label: 'Bob' });
    await page1.locator('.chat-container input[type="text"]').fill('Secret for Bob');
    await page1.click('.chat-container button');

    // Alice should see it (Private indicator)
    await expect(page1.locator('.messages li').last()).toContainText('(Private)');
    await expect(page1.locator('.messages li').last()).toContainText('Secret for Bob');

    // Bob should see it
    await expect(page2.locator('.messages li').last()).toContainText('(Private)');
    await expect(page2.locator('.messages li').last()).toContainText('Secret for Bob');

    // Eve should NOT see it
    // Check if Eve has any message. She might have system messages (joins) or nothing if chat is empty.
    // The test logic assumes "Secret for Bob" is not in Eve's chat.
    await expect(page3.locator('.messages')).not.toContainText('Secret for Bob');

    // Send public message
    await page1.locator('select').selectOption({ label: 'Everyone' });
    await page1.locator('.chat-container input[type="text"]').fill('Hello All');
    await page1.click('.chat-container button');

    // Everyone should see it
    await expect(page2.locator('.messages')).toContainText('Hello All');
    await expect(page3.locator('.messages')).toContainText('Hello All');

    await context1.close();
    await context2.close();
    await context3.close();
});

test('Poll Visuals E2E', async ({ browser, request }) => {
    // Reset config
    await request.post('http://localhost:3000/api/rooms', {
        data: {
            room_name: "PollVisualsRoom",
            is_locked: false,
            is_recording: false,
            is_lobby_enabled: false,
            max_participants: 100
        }
    });

    const context = await browser.newContext();
    const page = await context.newPage();
    await page.goto('/room/PollVisualsRoom');
    await page.locator('.prejoin-container input[type="text"]').fill('Voter');
    await page.click('button.join-btn');

    // Create Poll
    await page.click('button:has-text("Polls")');
    await page.click('button:has-text("Create Poll")');

    // Fill form
    const pollForm = page.locator('.modal-content .tab-content');
    await pollForm.locator('input').nth(0).fill('Visual Test?');
    await pollForm.locator('input').nth(1).fill('Yes');
    await pollForm.locator('input').nth(2).fill('No');
    await pollForm.locator('button:has-text("Create Poll")').click();

    // Verify initial 0%
    await expect(page.locator('.poll-bar').first()).toHaveCSS('width', '0px'); // 0%

    // Vote for Option 1
    await page.locator('button:has-text("Vote")').first().click();

    // Verify 100% width on first bar (approximate check via style attribute or existence)
    // The inline style sets width: 100%;
    await expect(page.locator('.poll-bar').first()).toHaveAttribute('style', /width: 100%;/);

    // Check text percentage
    // The received string has "(100%)", expected "(100 %)" (extra space from my format macro?)
    // format!("{:.0}", percent) -> "100"
    // " votes (" {format...} "%)" -> " votes (100%)" - wait, view! macro spacing rules?
    // Let's check received: "1 votes (100%)"
    // So no space.
    await expect(page.locator('.poll-item')).toContainText('(100%)');

    await context.close();
});

test('Chat Timestamp E2E', async ({ browser, request }) => {
    // Reset config
    await request.post('http://localhost:3000/api/rooms', {
        data: {
            room_name: "TimeRoom",
            is_locked: false,
            is_recording: false,
            is_lobby_enabled: false,
            max_participants: 100
        }
    });

    const context = await browser.newContext();
    const page = await context.newPage();
    await page.goto('/room/TimeRoom');
    await page.locator('.prejoin-container input[type="text"]').fill('TimeUser');
    await page.click('button.join-btn');

    await page.locator('.chat-container input[type="text"]').fill('Time Check');
    await page.click('.chat-container button');

    // Check for timestamp format [HH:MM]
    await expect(page.locator('.messages li').last()).toContainText(/\[\d{2}:\d{2}\]/);

    await context.close();
});

test('Allow All Lobby E2E', async ({ browser, request }) => {
    const roomName = 'LobbyAllowAll';
    await request.post('http://localhost:3000/api/rooms', {
        data: {
            room_name: roomName,
            is_locked: false,
            is_recording: false,
            is_lobby_enabled: true, // LOBBY ENABLED
            max_participants: 100
        }
    });

    const hostContext = await browser.newContext();
    const hostPage = await hostContext.newPage();
    await hostPage.goto(`/room/${roomName}`);
    await hostPage.locator('.prejoin-container input[type="text"]').fill('Host');
    await hostPage.click('button.join-btn');

    // Wait for Host to join
    await expect(hostPage.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

    // Guest 1
    const g1Context = await browser.newContext();
    const g1Page = await g1Context.newPage();
    await g1Page.goto(`/room/${roomName}`);
    await g1Page.locator('.prejoin-container input[type="text"]').fill('G1');
    await g1Page.click('button.join-btn');

    // Guest 2
    const g2Context = await browser.newContext();
    const g2Page = await g2Context.newPage();
    await g2Page.goto(`/room/${roomName}`);
    await g2Page.locator('.prejoin-container input[type="text"]').fill('G2');
    await g2Page.click('button.join-btn');

    // Verify Host sees 2 guests in waiting room
    await expect(hostPage.locator('.knocking-list li')).toHaveCount(2);

    // Click Allow All
    await hostPage.getByRole('button', { name: 'Allow All' }).click();

    // Verify guests enter room
    await expect(g1Page.getByText(`Meeting Room: ${roomName}`)).toBeVisible();
    await expect(g2Page.getByText(`Meeting Room: ${roomName}`)).toBeVisible();

    // Verify waiting list empty
    await expect(hostPage.locator('.knocking-list')).not.toBeVisible();

    await hostContext.close();
    await g1Context.close();
    await g2Context.close();
});

test('Leave Room E2E', async ({ page, request }) => {
    // Join room
    await request.post('http://localhost:3000/api/rooms', {
        data: {
            room_name: "LeaveRoom",
            is_locked: false,
            is_recording: false,
            is_lobby_enabled: false,
            max_participants: 100
        }
    });

    await page.goto('/room/LeaveRoom');
    await page.locator('.prejoin-container input[type="text"]').fill('Leaver');
    await page.click('button.join-btn');

    await expect(page.getByText('Meeting Room: LeaveRoom')).toBeVisible();

    // Click Leave
    await page.click('button:has-text("Leave")');

    // Should return to home
    await expect(page).toHaveURL(/\/$/);
    await expect(page.getByText('Welcome to Juncto')).toBeVisible();
});

test('Microphone Toggle E2E', async ({ page, request }) => {
    // Join room
    await request.post('http://localhost:3000/api/rooms', {
        data: {
            room_name: "MicRoom",
            is_locked: false,
            is_recording: false,
            is_lobby_enabled: false,
            max_participants: 100
        }
    });

    await page.goto('/room/MicRoom');
    await page.locator('.prejoin-container input[type="text"]').fill('Speaker');
    await page.click('button.join-btn');

    // Check Mute Button (initially Mute, green)
    const muteBtn = page.getByRole('button', { name: 'Mute' });
    await expect(muteBtn).toBeVisible();
    await expect(muteBtn).toHaveCSS('background-color', 'rgb(40, 167, 69)'); // #28a745

    // Click Mute
    await muteBtn.click();

    // Should change to Unmute (red)
    const unmuteBtn = page.getByRole('button', { name: 'Unmute' });
    await expect(unmuteBtn).toBeVisible();
    await expect(unmuteBtn).toHaveCSS('background-color', 'rgb(220, 53, 69)'); // #dc3545

    // Click Unmute
    await unmuteBtn.click();
    await expect(page.getByRole('button', { name: 'Mute' })).toBeVisible();
});

test('Meeting Timer E2E', async ({ page, request }) => {
    // Join room
    await request.post('http://localhost:3000/api/rooms', {
        data: {
            room_name: "TimerRoom",
            is_locked: false,
            is_recording: false,
            is_lobby_enabled: false,
            max_participants: 100
        }
    });

    await page.goto('/room/TimerRoom');
    await page.locator('.prejoin-container input[type="text"]').fill('TimerUser');
    await page.click('button.join-btn');

    await expect(page.getByText('Meeting Room: TimerRoom')).toBeVisible();

    const timer = page.locator('.meeting-timer');
    await expect(timer).toBeVisible();
    await expect(timer).toHaveText('00:00:00');

    // Wait for at least 1 second (allow some buffer)
    await page.waitForTimeout(1500);

    // Should have incremented
    const text = await timer.innerText();
    expect(text).not.toBe('00:00:00');
    expect(text).toMatch(/00:00:0\d/); // 01, 02...
});
