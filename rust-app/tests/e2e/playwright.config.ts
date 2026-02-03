import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './',
  workers: 1, // Enforce serial execution to prevent state collisions on singleton backend
  use: {
    baseURL: 'http://localhost:3000',
    launchOptions: {
      args: [
        '--use-fake-ui-for-media-stream',
        '--use-fake-device-for-media-stream',
      ],
    },
  },
  webServer: {
    // Build frontend first, then run backend
    command: 'cd ../.. && ./build.sh && cargo run -p backend',
    port: 3000,
    reuseExistingServer: !process.env.CI,
    timeout: 300000, // Increase timeout for build
  },
});
