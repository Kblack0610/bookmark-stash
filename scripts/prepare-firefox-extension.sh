#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${1:-$ROOT_DIR/dist/firefox-extension}"

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"
cp -R "$ROOT_DIR/extension/." "$OUT_DIR/"
rm -f "$OUT_DIR/manifest_chromium.json"

printf 'Prepared Firefox extension in %s\n' "$OUT_DIR"
