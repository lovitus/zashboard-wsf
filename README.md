# Zashboard Native (zashboard-wsf)

Native cross-platform Mihomo dashboard application built with [Tauri v2](https://v2.tauri.app/), wrapping the [zashboard](https://github.com/Zephyruso/zashboard) web UI.

## Features

- **Full zashboard UI** — all upstream features: proxies, rules, connections, logs, overview
- **Multi-backend management** — configure and switch between multiple Mihomo instances
- **Port forwarding tunnels** — built-in support for [slider](https://github.com/lovitus/slider), [gust](https://github.com/lovitus/gust), and [flyssh](https://github.com/lovitus/flyssh) to reach Mihomo instances behind SSH (listening on `127.0.0.1`)
- **Auto-start tunnels** — tunnels can be configured to start automatically on launch
- **Cross-platform** — Windows, macOS, Linux (Android via CI)
- **Lightweight** — native WebView, ~12 MB installer

## Download

Download the latest release from the [Releases](https://github.com/lovitus/zashboard-wsf/releases) page:

- **Windows**: `zashboard_x.x.x_x64-setup.exe` (NSIS installer)
- **macOS**: `zashboard_x.x.x_aarch64.dmg` / `zashboard_x.x.x_x64.dmg`
- **Linux**: `zashboard_x.x.x_amd64.deb` / `zashboard_x.x.x_amd64.AppImage`
- **Android**: `zashboard_x.x.x.apk`

## Tunnel Forwarding

When adding a Mihomo backend, enable **Port Forwarding Tunnel** to connect to instances behind SSH:

| Tool | Example Args |
|------|-------------|
| **slider** | `-listen ltcp://:19090/127.0.0.1:9090 -forward ssh://user@host:22` |
| **gust** | `-L tcp://:19090/127.0.0.1:9090 -F relay+ssh://user@host:22` |
| **flyssh** | `-L 19090:127.0.0.1:9090 user@host` |

The tunnel tool binary (`slider`, `gust`, or `flyssh`) must be in your system PATH.

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) >= 22
- [pnpm](https://pnpm.io/) >= 10
- [Rust](https://rustup.rs/) >= 1.85

### Setup

```bash
git clone https://github.com/lovitus/zashboard-wsf.git
cd zashboard-wsf
pnpm install
```

### Dev mode

```bash
pnpm tauri dev
```

### Production build

```bash
pnpm tauri build
# or specific bundle:
pnpm tauri build -- --bundles nsis   # Windows NSIS
pnpm tauri build -- --bundles dmg    # macOS DMG
pnpm tauri build -- --bundles deb    # Linux DEB
```

## Syncing with upstream zashboard

This project is based on [Zephyruso/zashboard](https://github.com/Zephyruso/zashboard) v2.7.0. To sync with upstream:

```bash
# Add upstream remote (one time)
git remote add upstream https://github.com/Zephyruso/zashboard.git

# Fetch and merge
git fetch upstream
git merge upstream/main
# Resolve conflicts in modified files:
#   - src/types/index.d.ts (TunnelConfig type)
#   - src/components/settings/EditBackendModal.vue (tunnel UI)
#   - src/i18n/*.ts (tunnel translations)
#   - vite.config.ts (Tauri integration)
#   - package.json (Tauri dependencies)
```

Only a few files are modified from upstream — merge conflicts should be minimal.

## Project Structure

```
zashboard-wsf/
├── src/                    # Vue.js frontend (from upstream zashboard)
│   ├── api/tunnel.ts       # Tunnel IPC API (new)
│   ├── types/index.d.ts    # Extended Backend type with TunnelConfig
│   └── components/settings/
│       └── EditBackendModal.vue  # Tunnel config UI (extended)
├── src-tauri/              # Rust backend (Tauri v2)
│   ├── src/lib.rs          # Tunnel process management, IPC commands
│   ├── tauri.conf.json     # Tauri configuration
│   └── capabilities/       # Permission scopes
├── .github/workflows/      # CI/CD
│   └── release.yml         # Cross-platform build + release
└── vite.config.ts          # Vite config with Tauri integration
```

## License

MIT — based on [zashboard](https://github.com/Zephyruso/zashboard) by Zephyruso
