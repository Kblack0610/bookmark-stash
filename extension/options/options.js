// Stash Options Page

const extensionApi = globalThis.browser ?? globalThis.chrome;
const DEFAULT_SERVER_URL = 'http://localhost:3030';

// DOM Elements
const serverUrlInput = document.getElementById('serverUrl');
const apiTokenInput = document.getElementById('apiToken');
const saveBtn = document.getElementById('saveBtn');
const testBtn = document.getElementById('testBtn');
const statusDiv = document.getElementById('status');
const connectionStatusDiv = document.getElementById('connectionStatus');

// Load saved settings
async function loadSettings() {
  const result = await extensionApi.storage.sync.get({
    serverUrl: DEFAULT_SERVER_URL,
    apiToken: ''
  });

  serverUrlInput.value = result.serverUrl;
  apiTokenInput.value = result.apiToken;

  // Test connection on load
  testConnection(result.serverUrl, result.apiToken);
}

// Save settings
async function saveSettings() {
  const serverUrl = serverUrlInput.value.trim() || DEFAULT_SERVER_URL;
  const apiToken = apiTokenInput.value.trim();

  await extensionApi.storage.sync.set({ serverUrl, apiToken });

  showStatus('Settings saved successfully!', 'success');
  testConnection(serverUrl, apiToken);
}

// Test connection
async function testConnection(serverUrl, apiToken) {
  connectionStatusDiv.className = 'connection-status disconnected';
  connectionStatusDiv.querySelector('.text').textContent = 'Testing connection...';

  try {
    const response = await fetch(`${serverUrl}/api/stats`, {
      headers: apiToken ? { 'Authorization': `Bearer ${apiToken}` } : {}
    });

    if (response.ok) {
      connectionStatusDiv.className = 'connection-status connected';
      connectionStatusDiv.querySelector('.text').textContent = 'Connected to Stash server';
    } else {
      connectionStatusDiv.className = 'connection-status disconnected';
      connectionStatusDiv.querySelector('.text').textContent = `Connection failed: ${response.status}`;
    }
  } catch (error) {
    connectionStatusDiv.className = 'connection-status disconnected';
    connectionStatusDiv.querySelector('.text').textContent = `Connection failed: ${error.message}`;
  }
}

// Show status message
function showStatus(message, type) {
  statusDiv.textContent = message;
  statusDiv.className = `status ${type}`;

  setTimeout(() => {
    statusDiv.className = 'status';
  }, 3000);
}

// Event listeners
saveBtn.addEventListener('click', saveSettings);
testBtn.addEventListener('click', () => {
  const serverUrl = serverUrlInput.value.trim() || DEFAULT_SERVER_URL;
  const apiToken = apiTokenInput.value.trim();
  testConnection(serverUrl, apiToken);
});

// Load settings on page load
loadSettings();
