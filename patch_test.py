import re

with open("rust-app/tests/e2e/migration.spec.ts", "r") as f:
    content = f.read()

pattern = r'''  // Host should NOT see it \(in real app they might, but current logic filters strict room match\)
  await hostPage\.waitForTimeout\(1000\);

  // We need to look at actual messages list to be completely safe against test flakes
  // where it picks up the text from input field being typed and cleared.
  // Note: Guest's message goes to Main room, host is in Breakout, shouldn't receive.
  await expect\(hostPage\.locator\('\.chat-container \.messages'\)\)\.not\.toContainText\('Main Message', \{ timeout: 1000 \}\);'''

replacement = r'''  // Host should NOT see it (in real app they might, but current logic filters strict room match)
  await hostPage.waitForTimeout(1000);

  // We need to look at actual messages list to be completely safe against test flakes
  // where it picks up the text from input field being typed and cleared.
  // Note: Guest's message goes to Main room, host is in Breakout, shouldn't receive.
  await expect(hostPage.locator('.chat-container .messages')).not.toContainText('Main Message');'''

content = re.sub(pattern, replacement, content)

with open("rust-app/tests/e2e/migration.spec.ts", "w") as f:
    f.write(content)
