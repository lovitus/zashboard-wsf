# Zashboard Native v1.0.0

First stable release of the native `zashboard-wsf` application.

This release marks the point where the wrapper, tunnel management, upstream UI version switching, Android packaging, and fallback/recovery flow are considered usable together as a product instead of a prototype.

## Version Baseline

- Native app release version: `1.0.0`
- Built-in upstream UI baseline: `zashboard 2.7.0`

These are intentionally different version lines.

## User-Facing Highlights

- Built-in native app shell for zashboard on desktop and Android
- Multi-backend save/select workflow inside the app
- Tunnel management with `gust` and `slider`
- Upstream UI version manager:
  fetch releases, download, activate, deactivate, delete, custom URL support
- Android upstream UI support with return path back to the built-in UI

## Important Behavior

### Built-in vs Upstream UI

- The built-in UI remains the primary control plane for backend setup, selection, tunnel management, and local app settings.
- When an upstream UI version is activated, backend entry can open the selected upstream dashboard served from a local HTTP server.
- Returning from upstream UI to the built-in UI is recovery-oriented on Android. A brief screen flash can happen during successful recovery and is expected in the current implementation.

### Android

- Safe-area padding is handled for the built-in mobile shell.
- Upstream UI mode injects floating `Setup` and `Built-in UI` controls.
- If direct native recovery does not succeed immediately, the app falls back to a controlled recovery page instead of leaving the user on a browser error page.

## Build And Release Scope

Validated CI release targets for `v1.0.0`:

- Windows x64
- Windows x86
- macOS Apple Silicon
- Android

Temporarily excluded from CI for this release:

- Linux
- macOS Intel
- iOS
- other unvalidated architectures

## Internal Improvements Included Before 1.0.0

- embedded sidecar delivery for `gust` and `slider`
- CI-side sidecar download and packaging
- Windows sidecar handling aimed at reducing antivirus false-positive pressure
- Android writable storage fix for downloaded upstream UI assets
- Android safe-area and mobile tap-behavior fixes
- hardened upstream-to-built-in recovery flow with multiple fallback layers

## Known Limitations

- In upstream UI mode, `Proxies` and `Rules` may still show transient `502` or `Network Error` depending on upstream/backend behavior.
- The Android built-in return path prioritizes reliability over smoothness, so a visual flash during recovery is acceptable for now.
- Release targets are intentionally narrower than full Tauri platform coverage until the currently supported matrix is stable enough to expand.

## Documentation

- README: `README.md`
- Changelog: `CHANGELOG.md`
- Maintenance checklist: `readme/MAINTENANCE.md`

## Upgrade Guidance

- Existing users of `0.2.x` builds can treat `1.0.0` as the stable continuation of the same native wrapper line.
- If you rely on upstream UI mode on Android, prefer testing the built-in return flow once after upgrade so the new recovery path is confirmed on your device.
