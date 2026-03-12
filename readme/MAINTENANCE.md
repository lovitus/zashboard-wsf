# Maintenance Checklist

This file tracks practical checks to run before each tagged release.

## 1) Build Health

- `pnpm -s vue-tsc --noEmit`
- `pnpm -s eslint .`
- `cargo check --manifest-path src-tauri/Cargo.toml`

## 2) Core Runtime Checks

- Setup page can add/select backend
- Tunnel manager can save/start/stop tunnel
- App can switch to active upstream UI version
- `Built-in UI` button returns to built-in page
- Tray double-click restores the main window

## 3) Android UX Checks

- Top/bottom safe padding visible after entering backend
- User can return to setup page from mobile UI

## 4) CI / Release Checks

- Ensure `release.yml` target policy is still correct for current phase
- Trigger release by tag (`v*`) only when branch is clean

## 5) Security Follow-ups (Open)

- Review whether upstream storage snapshot should be encrypted at rest

## Android Release Secrets

The Android signing step in `release.yml` expects these repository secrets:

- `ANDROID_KEYSTORE_BASE64` (recommended)
- `ANDROID_KEYSTORE_ARCHIVE_PASSWORD`
- `ANDROID_KEYSTORE_PASSWORD`
- `ANDROID_KEY_ALIAS` (optional, defaults to `zashboard`)

Notes:
- If `ANDROID_KEYSTORE_BASE64` is provided, CI decodes it directly and does not depend on zip passwords.
- If `ANDROID_KEYSTORE_BASE64` is empty, CI falls back to decrypting `android_keystore_encrypted.zip`:
  - first with `ANDROID_KEYSTORE_ARCHIVE_PASSWORD`
  - then with `ANDROID_KEYSTORE_PASSWORD`

Example (PowerShell) to generate `ANDROID_KEYSTORE_BASE64`:
- `[Convert]::ToBase64String([IO.File]::ReadAllBytes("zashboard-release.keystore"))`
