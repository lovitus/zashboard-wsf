# Maintenance Checklist

This file tracks practical checks to run before each tagged release.

## 0) Versioning Rules

- Native app release version lives in `src-tauri/tauri.conf.json`
- Rust crate version lives in `src-tauri/Cargo.toml`
- Built-in upstream UI version lives in `package.json`
- Do not bump `package.json` for wrapper-only releases
- Create a matching release notes file in `readme/RELEASE_NOTES_vX.Y.Z.md`
- Add a native-project entry at the top of `CHANGELOG.md`

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
- User can return from upstream UI to built-in UI

## 4) CI / Release Checks

- Ensure `release.yml` target policy is still correct for current phase
- Trigger release by tag (`v*`) only when branch is clean
- After CI assets are attached, publish the release and replace draft notes with the versioned notes file:
  `gh release edit vX.Y.Z --notes-file readme/RELEASE_NOTES_vX.Y.Z.md --draft=false`

## 5) Release Notes Content

- Mention the native app release version and the built-in upstream UI baseline separately
- Summarize user-facing features first, implementation details second
- Document Android upstream-to-built-in recovery behavior honestly
- Keep known limitations explicit instead of pretending they are solved

## 6) Security Follow-ups (Open)

- Review whether upstream storage snapshot should be encrypted at rest

## Android Signing (Open Key)

The Android signing step now uses an open keystore file in repo:

- Keystore path: `src-tauri/open-release.keystore`
- Alias: `zashboard`
- Store/key password: `zashboard-open-2026`

No repository secret is required for Android signing in CI.
