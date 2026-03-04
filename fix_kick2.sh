#!/bin/bash
sed -i "s/await expect(guestPage.locator('button:has-text(\"Start Meeting\")')).toBeVisible({ timeout: 15000 });/await expect(guestPage.locator('input[type=\"text\"]')).toBeVisible({ timeout: 15000 });/" rust-app/tests/e2e/migration.spec.ts
