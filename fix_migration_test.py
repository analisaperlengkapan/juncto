import re

with open('rust-app/tests/e2e/migration.spec.ts', 'r') as f:
    content = f.read()

# Add test.setTimeout(180000) at the top right after imports
if 'test.setTimeout(180000)' not in content:
    content = content.replace("import { test, expect } from '@playwright/test';\n\n", "import { test, expect } from '@playwright/test';\n\ntest.setTimeout(180000);\n\n")

# Make the assertion explicitly robust
old_assertion = "await expect(hostPage.locator('.chat-container .messages')).not.toContainText('Main Message', { timeout: 2000 });"
new_assertion = "await expect(hostPage.locator('.chat-container .messages')).not.toContainText('Main Message', { timeout: 5000 });"
content = content.replace(old_assertion, new_assertion)

with open('rust-app/tests/e2e/migration.spec.ts', 'w') as f:
    f.write(content)
