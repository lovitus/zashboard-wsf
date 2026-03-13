# Zashboard Native (zashboard-wsf)

Native Mihomo dashboard application built with [Tauri v2](https://v2.tauri.app/), based on [Zephyruso/zashboard](https://github.com/Zephyruso/zashboard).

`v1.0.1` is the current stable native release of this project.

## Project Scope

This repository is not just a repackaged upstream web UI. It adds a native runtime and operational features around zashboard:

- Native desktop/mobile wrapper for Mihomo dashboards
- Multi-backend management inside the app
- Tunnel integration with `gust` and `slider`
- Upstream UI version manager:
  download, activate, deactivate, delete, custom release URL support
- Android-specific recovery flow for returning from upstream UI to the built-in UI
- Release CI for Windows x64/x86, macOS Apple Silicon, and Android

## Versioning

This project intentionally uses two version lines:

- Native app release version:
  tracked in [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json) and [src-tauri/Cargo.toml](src-tauri/Cargo.toml)
- Built-in dashboard UI baseline:
  tracked by the upstream frontend version in [package.json](package.json)

Current baseline:

- Native app release line: `1.0.1`
- Built-in upstream UI baseline: `2.7.0`

Do not bump `package.json` just to publish a new native wrapper release unless the built-in upstream UI is actually updated.

## UI Modes

### Built-in UI

The built-in UI remains the safe control plane for:

- backend setup and selection
- tunnel management
- local app settings
- upstream UI version management

### Upstream UI

When a user activates a non-built-in UI version:

- the selected upstream UI is served from a local HTTP server on `127.0.0.1`
- backend entry can open directly into that upstream UI
- a floating `Setup` button is injected for returning to upstream setup
- a floating `Built-in UI` button is injected for switching back to the built-in UI

Android note:

- returning from upstream UI may briefly flash the screen while the recovery flow swaps back to the built-in UI
- this is acceptable behavior in the current design because reliability is prioritized over animation smoothness

## Current Release Targets

Current CI policy builds:

- Windows x64 (`x86_64-pc-windows-msvc`)
- Windows x86 (`i686-pc-windows-msvc`)
- macOS Apple Silicon (`aarch64-apple-darwin`)
- Android APK

Temporarily disabled:

- macOS Intel
- Linux
- iOS
- other architectures not yet validated end-to-end

Details live in [release.yml](.github/workflows/release.yml).

## Known Limitations

- In upstream UI mode, `Proxies` and `Rules` can still show transient `502` or `Network Error` in some backend states. Current evidence points primarily to upstream/backend timing behavior rather than a deterministic wrapper bug.
- Built-in UI and upstream UI run on different origins, so the wrapper must bridge request routing and localStorage migration. The implementation is recovery-oriented rather than elegant.
- Android recovery from upstream UI is intentionally defensive. It may create a brief visual flash during fallback or restart-based recovery.

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) >= 22
- [pnpm](https://pnpm.io/) >= 10
- [Rust](https://rustup.rs/) stable

### Setup

```bash
git clone https://github.com/lovitus/zashboard-wsf.git
cd zashboard-wsf
pnpm install
```

### Run In Dev

```bash
pnpm tauri dev
```

### Local Checks

```bash
pnpm -s vue-tsc --noEmit
pnpm -s eslint .
cargo check --manifest-path src-tauri/Cargo.toml
```

## Build And Release

### Local Build

```bash
pnpm tauri build
```

### CI Release Trigger

Release workflow triggers on:

- push tag: `v*`
- manual `workflow_dispatch`

Example:

```bash
git tag v1.0.1
git push origin v1.0.1
```

## Tunnel Usage

When adding a Mihomo backend, enable tunnel support if the backend is only reachable through SSH or relay paths.

| Tool | Example Args |
|------|--------------|
| `slider` | `-listen ltcp://:19090/127.0.0.1:9090 -forward ssh://user@host:22` |
| `gust` | `-L tcp://:19090/127.0.0.1:9090 -F relay+ssh://user@host:22` |

## Repository Layout

```text
src/                Vue frontend, largely based on upstream zashboard
src-tauri/          Rust backend, Tauri runtime, tunnel manager, UI manager
readme/             maintenance notes, release notes, screenshots
.github/workflows/  CI/CD workflows
```

## Documentation

- Release checklist: [readme/MAINTENANCE.md](readme/MAINTENANCE.md)
- Release notes: [readme/RELEASE_NOTES_v1.0.1.md](readme/RELEASE_NOTES_v1.0.1.md)
- Changelog: [CHANGELOG.md](CHANGELOG.md)

## License

MIT. Built on top of zashboard by Zephyruso.
