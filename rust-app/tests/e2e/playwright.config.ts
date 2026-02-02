import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './',
  use: {
    baseURL: 'http://localhost:3000',
  },
  webServer: {
    // Build frontend first, then run backend
    command: 'cd ../.. && ./build.sh && cargo run -p backend',
    port: 3000,
    reuseExistingServer: !process.env.CI,
    timeout: 300000, // Increase timeout for build
  },
});
