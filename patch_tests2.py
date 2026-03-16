import os
import glob

def main():
    test_dir = "rust-app/tests/e2e"
    files = glob.glob(os.path.join(test_dir, "*.spec.ts"))

    for f in files:
        with open(f, "r") as file:
            content = file.read()

        # Replace the hardcoded click with a conditional or just remove the click since it's true by default.
        # But wait, it might be necessary if some tests rely on it being closed.
        # Instead, let's change the test to conditionally open it if it's hidden.
        # Actually, in Playwright, we can just do:
        # `if (await page.locator('.participants-list').isHidden()) { await page.click('.toolbox button:has-text("Participants")'); }`

        new_content = content.replace(
            """await page.click('.toolbox button:has-text("Participants")');""",
            """if (await page.locator('.participants-list').isHidden()) { await page.click('.toolbox button:has-text("Participants")'); }"""
        ).replace(
            """await hostPage.click('.toolbox button:has-text("Participants")');""",
            """if (await hostPage.locator('.participants-list').isHidden()) { await hostPage.click('.toolbox button:has-text("Participants")'); }"""
        ).replace(
            """await guestPage.click('.toolbox button:has-text("Participants")');""",
            """if (await guestPage.locator('.participants-list').isHidden()) { await guestPage.click('.toolbox button:has-text("Participants")'); }"""
        ).replace(
            """await user1Page.click('.toolbox button:has-text("Participants")');""",
            """if (await user1Page.locator('.participants-list').isHidden()) { await user1Page.click('.toolbox button:has-text("Participants")'); }"""
        ).replace(
            """await page1.click('.toolbox button:has-text("Participants")');""",
            """if (await page1.locator('.participants-list').isHidden()) { await page1.click('.toolbox button:has-text("Participants")'); }"""
        ).replace(
            """await page2.click('.toolbox button:has-text("Participants")');""",
            """if (await page2.locator('.participants-list').isHidden()) { await page2.click('.toolbox button:has-text("Participants")'); }"""
        ).replace(
            """await pageA.click('.toolbox button:has-text("Participants")');""",
            """if (await pageA.locator('.participants-list').isHidden()) { await pageA.click('.toolbox button:has-text("Participants")'); }"""
        )

        if new_content != content:
            with open(f, "w") as file:
                file.write(new_content)

if __name__ == "__main__":
    main()
