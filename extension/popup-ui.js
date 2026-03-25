// Stash Popup - Plain JS UI
const extensionApi = globalThis.browser ?? globalThis.chrome;

const DEFAULT_SERVER_URL = 'http://localhost:3030';

async function getServerUrl() {
  const result = await extensionApi.storage.sync.get({ serverUrl: DEFAULT_SERVER_URL });
  return result.serverUrl;
}

async function apiPatch(id, body) {
  const serverUrl = await getServerUrl();
  const resp = await fetch(`${serverUrl}/api/bookmarks/${id}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
  return resp.json();
}

// --- Save current page ---
const saveBtn = document.getElementById('save-btn');
const saveStatus = document.getElementById('save-status');

saveBtn.addEventListener('click', async () => {
  saveBtn.disabled = true;
  saveBtn.textContent = 'Saving...';
  saveStatus.hidden = true;

  try {
    const [tab] = await extensionApi.tabs.query({ active: true, currentWindow: true });
    if (!tab || !tab.url) throw new Error('No active tab');

    const serverUrl = await getServerUrl();
    const resp = await fetch(`${serverUrl}/api/bookmarks`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ url: tab.url, title: tab.title || tab.url }),
    });

    if (!resp.ok) {
      const err = await resp.json().catch(() => ({}));
      throw new Error(err.error || err.message || `HTTP ${resp.status}`);
    }

    saveStatus.textContent = 'Saved!';
    saveStatus.className = 'save-status success';
    saveStatus.hidden = false;
    saveBtn.textContent = 'Saved';

    loadBookmarks();

    setTimeout(() => {
      saveBtn.textContent = '+ Save';
      saveBtn.disabled = false;
      saveStatus.hidden = true;
    }, 2000);
  } catch (err) {
    saveStatus.textContent = err.message;
    saveStatus.className = 'save-status error';
    saveStatus.hidden = false;
    saveBtn.textContent = '+ Save';
    saveBtn.disabled = false;
  }
});

// --- Filter tabs ---
let currentFilter = 'unread';
document.querySelectorAll('.filter-btn').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelector('.filter-btn.active').classList.remove('active');
    btn.classList.add('active');
    currentFilter = btn.dataset.filter;
    loadBookmarks();
  });
});

// --- Search ---
const searchInput = document.getElementById('search-input');
let searchTimeout;
searchInput.addEventListener('input', () => {
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => loadBookmarks(), 300);
});

// --- Keyboard navigation ---
let selectedIndex = -1;

document.addEventListener('keydown', (e) => {
  const items = bookmarkList.querySelectorAll('.bookmark-item');
  if (!items.length) return;

  if (e.key === 'ArrowDown' || (e.key === 'j' && !searchInput.matches(':focus'))) {
    e.preventDefault();
    selectedIndex = Math.min(selectedIndex + 1, items.length - 1);
    updateSelection(items);
  } else if (e.key === 'ArrowUp' || (e.key === 'k' && !searchInput.matches(':focus'))) {
    e.preventDefault();
    selectedIndex = Math.max(selectedIndex - 1, 0);
    updateSelection(items);
  } else if (e.key === 'Enter' && selectedIndex >= 0) {
    e.preventDefault();
    items[selectedIndex].click();
  } else if (e.key === 'a' && !searchInput.matches(':focus') && selectedIndex >= 0) {
    // Archive selected
    e.preventDefault();
    const archiveBtn = items[selectedIndex].querySelector('.archive-btn');
    if (archiveBtn) archiveBtn.click();
  } else if (e.key === '/' && !searchInput.matches(':focus')) {
    e.preventDefault();
    searchInput.focus();
  } else if (e.key === 'Escape') {
    searchInput.blur();
    selectedIndex = -1;
    updateSelection(items);
  }
});

function updateSelection(items) {
  items.forEach((item, i) => {
    item.classList.toggle('selected', i === selectedIndex);
  });
  if (selectedIndex >= 0) {
    items[selectedIndex].scrollIntoView({ block: 'nearest' });
  }
}

// --- Load and render bookmarks ---
const bookmarkList = document.getElementById('bookmark-list');
const emptyState = document.getElementById('empty-state');

async function loadBookmarks() {
  const serverUrl = await getServerUrl();
  const query = searchInput.value.trim();
  selectedIndex = -1;

  try {
    let bookmarks;
    if (query) {
      const resp = await fetch(`${serverUrl}/api/search?q=${encodeURIComponent(query)}&limit=30`);
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      const data = await resp.json();
      bookmarks = data.results.map(r => r.bookmark);
    } else {
      let url = `${serverUrl}/api/bookmarks?limit=30`;
      if (currentFilter === 'unread') url += '&status=unread';
      else if (currentFilter === 'archived') url += '&status=archived';

      const resp = await fetch(url);
      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
      const data = await resp.json();
      bookmarks = data.bookmarks;
    }

    renderBookmarks(bookmarks);
  } catch (err) {
    bookmarkList.innerHTML = '';
    emptyState.hidden = false;
    emptyState.querySelector('p').textContent = `Error: ${err.message}`;
  }
}

function renderBookmarks(bookmarks) {
  bookmarkList.innerHTML = '';

  if (!bookmarks || bookmarks.length === 0) {
    emptyState.hidden = false;
    emptyState.querySelector('p').textContent = 'No bookmarks found';
    return;
  }

  emptyState.hidden = true;

  for (const bm of bookmarks) {
    const li = document.createElement('li');
    li.className = 'bookmark-item';
    li.tabIndex = 0;

    const content = document.createElement('div');
    content.className = 'bookmark-content';

    const title = document.createElement('h3');
    title.className = 'bookmark-title';
    title.textContent = bm.title || bm.url;

    const meta = document.createElement('div');
    meta.className = 'bookmark-meta';
    const site = document.createElement('span');
    site.className = 'site-name';
    site.textContent = bm.site_name || new URL(bm.url).hostname;
    meta.appendChild(site);

    if (bm.estimated_read_time) {
      const sep = document.createElement('span');
      sep.className = 'separator';
      sep.textContent = ' \u2022 ';
      const time = document.createElement('span');
      time.className = 'read-time';
      time.textContent = `${bm.estimated_read_time} min`;
      meta.appendChild(sep);
      meta.appendChild(time);
    }

    content.appendChild(title);
    content.appendChild(meta);

    if (bm.tags && bm.tags.length > 0) {
      const tagsDiv = document.createElement('div');
      tagsDiv.className = 'bookmark-tags';
      for (const tag of bm.tags) {
        const span = document.createElement('span');
        span.className = 'tag';
        span.textContent = tag.name;
        tagsDiv.appendChild(span);
      }
      content.appendChild(tagsDiv);
    }

    // Action buttons
    const actions = document.createElement('div');
    actions.className = 'bookmark-actions';

    const archiveBtn = document.createElement('button');
    archiveBtn.className = 'archive-btn';
    archiveBtn.title = bm.status === 'archived' ? 'Unarchive' : 'Archive';
    archiveBtn.textContent = bm.status === 'archived' ? '\u21A9' : '\u2713';
    archiveBtn.addEventListener('click', async (e) => {
      e.stopPropagation();
      const newStatus = bm.status === 'archived' ? 'unread' : 'archived';
      await apiPatch(bm.id, { status: newStatus });
      loadBookmarks();
    });

    actions.appendChild(archiveBtn);
    li.appendChild(content);
    li.appendChild(actions);

    // Click opens URL and marks as read
    li.addEventListener('click', async () => {
      extensionApi.tabs.create({ url: bm.url });
      if (bm.status === 'unread') {
        await apiPatch(bm.id, { status: 'read' });
      }
    });

    bookmarkList.appendChild(li);
  }
}

// Initial load
loadBookmarks();
