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
    const app = document.getElementById('app');
    app.textContent = '';
    const errorDiv = document.createElement('div');
    errorDiv.className = 'error';
    const h2 = document.createElement('h2');
    h2.textContent = 'Failed to load Stash';
    const p = document.createElement('p');
    p.textContent = error.message;
    const btn = document.createElement('button');
    btn.textContent = 'Retry';
    btn.addEventListener('click', () => location.reload());
    errorDiv.append(h2, p, btn);
    app.appendChild(errorDiv);
  }
}

initWasm();
