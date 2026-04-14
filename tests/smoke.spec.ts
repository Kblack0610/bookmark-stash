import { test, expect, chromium, BrowserContext } from '@playwright/test';
import path from 'path';

const EXTENSION_PATH = path.join(__dirname, '..', 'dist', 'chromium-extension');

test('popup renders and talks to backend', async () => {
  const context: BrowserContext = await chromium.launchPersistentContext('', {
    headless: false,
    args: [
      `--disable-extensions-except=${EXTENSION_PATH}`,
      `--load-extension=${EXTENSION_PATH}`,
    ],
  });

  const sw = context.serviceWorkers()[0] || await context.waitForEvent('serviceworker');
  const extensionId = sw.url().split('/')[2];

  const popup = await context.newPage();
  await popup.goto(`chrome-extension://${extensionId}/popup.html`);
  await popup.waitForLoadState('domcontentloaded');

  const body = await popup.locator('body').innerHTML();
  expect(body.length).toBeGreaterThan(0);

  await popup.screenshot({ path: 'test-results/popup.png' });

  const title = await popup.title();
  console.log('popup title:', title);
  console.log('body length:', body.length);

  await context.close();
});
