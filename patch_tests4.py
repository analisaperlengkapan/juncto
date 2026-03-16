import os
import glob

def main():
    test_dir = "rust-app/tests/e2e"
    files = glob.glob(os.path.join(test_dir, "*.spec.ts"))

    for f in files:
        with open(f, "r") as file:
            content = file.read()

        # Kick Participant: The button is not visible even when forced to click. Let's make sure the participants panel is actually open.
        new_content = content.replace(
            """if (await hostPage.locator('.participants-list').isHidden()) { await hostPage.click('.toolbox button:has-text("Participants")'); }""",
            """await hostPage.click('.toolbox button:has-text("Participants")'); await hostPage.waitForSelector('.participants-list', { state: 'visible' });"""
        ).replace(
            """await guestItem.getByRole('button', { name: 'Kick' }).scrollIntoViewIfNeeded(); await guestItem.getByRole('button', { name: 'Kick' }).click({ force: true });""",
            """await guestItem.getByRole('button', { name: 'Kick' }).click();"""
        )

        # Breakout Rooms E2E: It expects a message but it doesn't see it.
        # Maybe the message is not being sent properly or we are looking in the wrong place.
        # Let's remove the `.first().toBeAttached` since it times out.
        if "Breakout Rooms E2E" in content:
            new_content = new_content.replace(
                """await expect(hostPage.locator('.messages')).toBeVisible({ timeout: 5000 }); await expect(hostPage.locator('.messages li').filter({ hasText: lastMsg }).first()).toBeAttached({ timeout: 5000 }); await expect(hostPage.locator('.messages li').filter({ hasText: lastMsg }).first()).toBeVisible({ timeout: 5000 });""",
                """await expect(hostPage.locator('.messages li').filter({ hasText: lastMsg }).first()).toBeVisible({ timeout: 5000 });"""
            )

        if new_content != content:
            with open(f, "w") as file:
                file.write(new_content)

if __name__ == "__main__":
    main()
