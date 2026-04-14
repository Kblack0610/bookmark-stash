# Installing Stash in Firefox

Three tiers, from zero-setup (dev only) to persistent (personal daily use).

## 1. Temporary (dev only — wipes on Firefox restart)

```bash
npx web-ext run --source-dir extension/ --firefox /usr/bin/firefox
```

Or manually: open `about:debugging` → "This Firefox" → "Load Temporary Add-on" → pick `extension/manifest.json`.

## 2. Unsigned persistent (Dev Edition / Nightly / ESR only)

Release Firefox will not install unsigned extensions. If you use **Developer Edition**, **Nightly**, or **ESR**, set in `about:config`:

```
xpinstall.signatures.required = false
```

Then build a zip and drag it into Firefox:

```bash
npm run build:xpi
# artifact at web-ext-artifacts/stash-1.0.0.zip — rename to .xpi and drag into Firefox
```

## 3. Signed + persistent (recommended — works in any Firefox)

Uses Mozilla's **unlisted self-distribution** flow. The XPI is signed by Mozilla but not listed on addons.mozilla.org.

### One-time setup

1. Sign in at https://addons.mozilla.org/developers/
2. Generate API credentials at https://addons.mozilla.org/developers/addon/api/key/
3. Save them to `~/.config/stash/amo-env` (chmod 600):

   ```bash
   mkdir -p ~/.config/stash
   cat > ~/.config/stash/amo-env <<EOF
   export MOZILLA_JWT_ISSUER="user:XXXX:YYY"
   export MOZILLA_JWT_SECRET="<hex secret>"
   EOF
   chmod 600 ~/.config/stash/amo-env
   ```

### Sign + download

```bash
npm run sign:firefox              # defaults to --channel unlisted
# or
npm run sign:firefox listed       # for public AMO listing instead
```

The script reads `~/.config/stash/amo-env`, uploads the extension, waits for Mozilla's automated review (usually minutes), and downloads the signed XPI to `web-ext-artifacts/`.

### Install the signed XPI

1. Open Firefox → `about:addons`
2. Click the gear icon → "Install Add-on From File…"
3. Select the `.xpi` from `web-ext-artifacts/`

Survives restarts. Auto-updates are **not** wired up for unlisted extensions (you'd re-sign and reinstall to update).

## Notes

- Extension ID: `stash@kennethblack.me` (set in `extension/manifest.json` under `browser_specific_settings.gecko.id`). Changing this after publish creates a *new* extension, breaking updates.
- Extension expects the backend at `http://localhost:3030`. Start it with `cargo run -p stash-backend`. For daily use, wire a systemd user unit.
- Before submitting for signing, run `npm run lint:ext` to catch manifest issues that would fail AMO validation.
