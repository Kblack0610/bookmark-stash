// Stash - Background Service Worker
// Handles hotkey capture and API communication

const extensionApi = globalThis.browser ?? globalThis.chrome;

// Default settings
const DEFAULT_SERVER_URL = 'http://localhost:3030';

// Get settings from storage
async function getSettings() {
  const result = await extensionApi.storage.sync.get({
    serverUrl: DEFAULT_SERVER_URL,
    apiToken: ''
  });
  return result;
}

async function setBadge(text, color, title) {
  if (!extensionApi.action) {
    return;
  }

  await Promise.allSettled([
    extensionApi.action.setBadgeText({ text }),
    extensionApi.action.setBadgeBackgroundColor({ color }),
    title ? extensionApi.action.setTitle({ title }) : Promise.resolve()
  ]);
}

// Save a URL to Stash
async function saveToStash(url, title) {
  const settings = await getSettings();

  try {
    const response = await fetch(`${settings.serverUrl}/api/bookmarks`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(settings.apiToken && { 'Authorization': `Bearer ${settings.apiToken}` })
      },
      body: JSON.stringify({ url, title })
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.message || error.error || 'Failed to save');
    }

    const bookmark = await response.json();
    return { success: true, bookmark };
  } catch (error) {
    console.error('Stash save error:', error);
    return { success: false, error: error.message };
  }
}

// Handle keyboard shortcut
extensionApi.commands.onCommand.addListener(async (command) => {
  if (command === 'save-page') {
    // Get the active tab
    const [tab] = await extensionApi.tabs.query({ active: true, currentWindow: true });

    if (!tab || !tab.url) {
      console.error('No active tab or URL');
      return;
    }

    // Skip chrome:// and other internal URLs
    if (tab.url.startsWith('chrome://') ||
        tab.url.startsWith('chrome-extension://') ||
        tab.url.startsWith('about:')) {
      console.log('Skipping internal URL');
      return;
    }

    await setBadge('...', '#2563eb', 'Saving page to Stash');
    const result = await saveToStash(tab.url, tab.title || tab.url);

    if (result.success) {
      await setBadge('OK', '#16a34a', `Saved to Stash: ${result.bookmark.title}`);
    } else {
      await setBadge('ERR', '#dc2626', `Stash save failed: ${result.error}`);
    }

    setTimeout(() => {
      void setBadge('', '#2563eb', 'Stash - Reading List');
    }, 2500);
  }
});

// Listen for messages from popup
extensionApi.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.type === 'save-url') {
    saveToStash(message.url, message.title).then(sendResponse);
    return true; // Keep channel open for async response
  }

  if (message.type === 'get-current-tab') {
    extensionApi.tabs.query({ active: true, currentWindow: true }).then(([tab]) => {
      sendResponse({ url: tab?.url, title: tab?.title });
    });
    return true;
  }
});

console.log('Stash background service worker loaded');
