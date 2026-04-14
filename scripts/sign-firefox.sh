#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="${STASH_AMO_ENV:-$HOME/.config/stash/amo-env}"
CHANNEL="${1:-unlisted}"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "error: credentials file not found at $ENV_FILE" >&2
  echo "create it with MOZILLA_JWT_ISSUER and MOZILLA_JWT_SECRET exports (chmod 600)" >&2
  exit 1
fi

# shellcheck disable=SC1090
source "$ENV_FILE"

: "${MOZILLA_JWT_ISSUER:?MOZILLA_JWT_ISSUER not set in $ENV_FILE}"
: "${MOZILLA_JWT_SECRET:?MOZILLA_JWT_SECRET not set in $ENV_FILE}"

case "$CHANNEL" in
  listed|unlisted) ;;
  *) echo "error: channel must be 'listed' or 'unlisted' (got '$CHANNEL')" >&2; exit 1 ;;
esac

cd "$ROOT_DIR"

npx --yes web-ext sign \
  --source-dir extension/ \
  --artifacts-dir web-ext-artifacts \
  --channel "$CHANNEL" \
  --api-key "$MOZILLA_JWT_ISSUER" \
  --api-secret "$MOZILLA_JWT_SECRET"

printf '\nsigned XPI(s):\n'
ls -1 web-ext-artifacts/*.xpi 2>/dev/null || echo '(none — check web-ext output above)'
