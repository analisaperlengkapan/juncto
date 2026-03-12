with open('rust-app/tests/e2e/migration.spec.ts', 'r') as f:
    content = f.read()

content = content.replace("// // await expect(hostPage.locator('.chat-container .messages')).not.toContainText('Main Message');", "await expect(hostPage.locator('.chat-container .messages')).not.toContainText('Main Message', { timeout: 2000 });")

with open('rust-app/tests/e2e/migration.spec.ts', 'w') as f:
    f.write(content)
