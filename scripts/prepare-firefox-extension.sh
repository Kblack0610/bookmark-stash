#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${1:-$ROOT_DIR/dist/firefox-extension}"

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"
cp -R "$ROOT_DIR/extension/." "$OUT_DIR/"
cp "$ROOT_DIR/extension/manifest_firefox.json" "$OUT_DIR/manifest.json"
rm -f "$OUT_DIR/manifest_firefox.json"

printf 'Prepared Firefox extension in %s\n' "$OUT_DIR"
