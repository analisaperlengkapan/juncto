import os
import glob

def main():
    test_dir = "rust-app/tests/e2e"
    files = glob.glob(os.path.join(test_dir, "*.spec.ts"))

    for f in files:
        with open(f, "r") as file:
            content = file.read()

        new_content = content.replace(
            """await guestItem.getByRole('button', { name: 'Kick' }).click({ force: true });""",
            """await guestItem.getByRole('button', { name: 'Kick' }).scrollIntoViewIfNeeded(); await guestItem.getByRole('button', { name: 'Kick' }).click({ force: true });"""
        ).replace(
            """await expect(hostPage.locator('.messages li').filter({ hasText: lastMsg }).first()).toBeVisible({ timeout: 5000 });""",
            """await expect(hostPage.locator('.messages li').filter({ hasText: lastMsg }).first()).toBeAttached({ timeout: 5000 }); await expect(hostPage.locator('.messages li').filter({ hasText: lastMsg }).first()).toBeVisible({ timeout: 5000 });"""
        )

        # for breakout rooms, wait for messages list to be visible before looking for the LI
        if "Breakout Rooms E2E" in content:
            new_content = new_content.replace(
                """await expect(hostPage.locator('.messages li').filter({ hasText: lastMsg }).first()).toBeAttached({ timeout: 5000 });""",
                """await expect(hostPage.locator('.messages')).toBeVisible({ timeout: 5000 }); await expect(hostPage.locator('.messages li').filter({ hasText: lastMsg }).first()).toBeAttached({ timeout: 5000 });"""
            )

        if new_content != content:
            with open(f, "w") as file:
                file.write(new_content)

if __name__ == "__main__":
    main()
