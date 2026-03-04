#!/bin/bash
sed -i "s/\/\/ await expect(guestPage.locator('.chat-container')).not.toContainText('Secret Message');/await expect(guestPage.locator('.chat-container')).not.toContainText('Secret Message');/" rust-app/tests/e2e/migration.spec.ts
sed -i "s/\/\/ await expect(hostPage.locator('.chat-container')).not.toContainText('Main Message');/await expect(hostPage.locator('.chat-container')).not.toContainText('Main Message');/" rust-app/tests/e2e/migration.spec.ts
