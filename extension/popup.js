// Stash Popup - WASM Loader
// Loads the Leptos WASM application

import init from './pkg/stash_frontend.js';

// Initialize WASM module
async function initWasm() {
  try {
    await init();
    console.log('Stash WASM initialized');
  } catch (error) {
    console.error('Failed to initialize WASM:', error);
    document.getElementById('app').innerHTML = `
      <div class="error">
        <h2>Failed to load Stash</h2>
        <p>${error.message}</p>
        <button onclick="location.reload()">Retry</button>
      </div>
    `;
  }
}

initWasm();
