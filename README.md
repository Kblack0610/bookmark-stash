# Stash

Local-first bookmark vault: Rust backend + Firefox extension.

Save URLs intentionally to a personal reading list, separate from the browser bookmark bar. Everything stays on your machine (SQLite at `~/.stash/stash.db`).

## Architecture

```
extension/          Firefox popup (plain JS) + background service worker
  popup-ui.js       Popup UI: save, search, filter, keyboard nav
  background.js     Hotkey handler, badge feedback
backend/            Axum REST API on localhost:3030
  src/api/          Route handlers (bookmarks, search, tags, import, stats)
  src/db/           SQLite via rusqlite, FTS5 full-text search, migrations
  src/content/      URL content fetcher + HTML extractor
shared/             Types shared between backend and extension
  src/lib.rs        Bookmark, Folder, Tag, request/response DTOs
```

## Quick Start

```bash
# Start the backend
cargo run -p stash-backend

# Load extension in Firefox
npx web-ext run --source-dir extension/ --firefox /usr/bin/firefox

# Or manually: about:debugging > Load Temporary Add-on > extension/manifest.json
```

Environment variables:
- `STASH_DB_PATH` - SQLite path (default: `~/.stash/stash.db`)
- `STASH_PORT` - Server port (default: `3030`)

## API

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/health` | Health check |
| POST | `/api/bookmarks` | Create bookmark (fetches content automatically) |
| GET | `/api/bookmarks` | List bookmarks (filter: `status`, `is_favorite`, `tag`, `folder_id`) |
| GET | `/api/bookmarks/:id` | Get single bookmark |
| GET | `/api/bookmarks/:id/content` | Get reader-formatted content |
| PATCH | `/api/bookmarks/:id` | Update bookmark (status, tags, folder, etc.) |
| DELETE | `/api/bookmarks/:id` | Delete bookmark |
| GET | `/api/search?q=...` | Full-text search (FTS5 with BM25 ranking) |
| GET | `/api/tags` | List all tags with counts |
| GET | `/api/stats` | Reading statistics |
| POST | `/api/import/firefox` | Import from Firefox HTML bookmark export |

## Extension Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+,` | Open Stash popup |
| `j` / `k` | Navigate bookmark list |
| `Enter` | Open selected bookmark (archives it) |
| `a` | Archive / unarchive selected |
| `d` | Delete selected (with confirmation) |
| `/` | Focus search |
| `Esc` | Clear focus |

Shortcuts are remappable at `about:addons` > gear > Manage Extension Shortcuts.

## Development

```bash
cargo build --workspace       # Build backend + shared
cargo test --workspace        # Run tests
npx web-ext lint --source-dir extension/  # Lint extension
```
