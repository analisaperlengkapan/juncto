with open('rust-app/tests/e2e/embed_meeting.spec.ts', 'r') as f:
    content = f.read()

content = content.replace("await page.click('button:has-text(\"Embed Meeting\")');", "await page.locator('button', { hasText: 'Embed Meeting' }).click();")

with open('rust-app/tests/e2e/embed_meeting.spec.ts', 'w') as f:
    f.write(content)
