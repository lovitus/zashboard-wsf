# Zashboard Native (zashboard-wsf)

Native Mihomo dashboard app built with [Tauri v2](https://v2.tauri.app/), based on [Zephyruso/zashboard](https://github.com/Zephyruso/zashboard).

## What This Project Adds

- Desktop/mobile wrapper for zashboard
- Multi-backend management
- Tunnel integration (`gust` / `slider`) and auto-start
- Upstream UI version manager (download, activate, deactivate, delete)
- Android CI packaging

## Current Release Targets (CI Policy)

Release workflow currently builds:

- Windows x64 (`x86_64-pc-windows-msvc`)
- Windows x86 (`i686-pc-windows-msvc`)
- macOS Apple Silicon (`aarch64-apple-darwin`)
- Android APK (separate `build-android` job)

Temporarily disabled in CI:

- macOS Intel
- Linux targets
- iOS and other architectures

Details are documented inline in [`.github/workflows/release.yml`](.github/workflows/release.yml).

## Upstream UI Mode (Important)

When a user activates a non-built-in UI version:

- The selected UI is hosted by a local HTTP server (`127.0.0.1:random_port`)
- Built-in UI is still used for setup/config/tunnel management
- Entering backend from setup/select flow can jump to the active upstream UI
- `Built-in UI` button in upstream page requests backend-side deactivation and returns to built-in page

Notes:

- Upstream UI mode depends on request proxying and localStorage transfer between origins.
- Transient `502` / network errors may still occur depending on upstream UI behavior and backend state.

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) >= 22
- [pnpm](https://pnpm.io/) >= 10
- [Rust](https://rustup.rs/) (stable)

### Setup

```bash
git clone https://github.com/lovitus/zashboard-wsf.git
cd zashboard-wsf
pnpm install
```

### Dev

```bash
pnpm tauri dev
```

### Local Checks

```bash
pnpm -s vue-tsc --noEmit
pnpm -s eslint .
cargo check --manifest-path src-tauri/Cargo.toml
```

## Build / Release

### Local Build

```bash
pnpm tauri build
```

### CI Release Trigger

`release.yml` is triggered by:

- Push tag: `v*`
- Manual `workflow_dispatch`

Example:

```bash
git tag v0.2.20
git push origin v0.2.20
```

## Tunnel Usage

When adding a Mihomo backend, enable tunnel to reach private endpoints.

| Tool | Example Args |
|------|-------------|
| `slider` | `-listen ltcp://:19090/127.0.0.1:9090 -forward ssh://user@host:22` |
| `gust` | `-L tcp://:19090/127.0.0.1:9090 -F relay+ssh://user@host:22` |

## Repository Layout

```text
src/                Vue frontend (mostly upstream zashboard)
src-tauri/          Rust backend (Tauri runtime, tunnel, UI manager)
.github/workflows/  CI/CD workflows
```

## Maintenance

- Release checklist: [readme/MAINTENANCE.md](readme/MAINTENANCE.md)

## License

MIT. Based on zashboard by Zephyruso.
