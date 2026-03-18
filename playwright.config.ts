import { defineConfig } from '@playwright/test';
import path from 'path';

export default defineConfig({
  testDir: './tests',
  timeout: 30000,
  retries: 0,
  use: {
    headless: false, // Extensions require headed mode
    viewport: { width: 1280, height: 720 },
  },
  projects: [
    {
      name: 'chromium-extension',
      use: {
        // Load extension in Chromium
        launchOptions: {
          args: [
            `--disable-extensions-except=${path.join(__dirname, 'extension')}`,
            `--load-extension=${path.join(__dirname, 'extension')}`,
          ],
        },
      },
    },
  ],
});
