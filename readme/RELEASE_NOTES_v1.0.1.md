# Zashboard Native v1.0.1

Release date: 2026-03-13

## Summary

`v1.0.1` is a targeted Android tunnel compatibility release.

It fixes domain resolution for built-in `gust` and `slider` tunnels on Android when the packaged app environment does not expose a usable `/etc/resolv.conf`.

## What Changed

- Android tunnel child processes now receive a Termux-compatible DNS environment at startup
- the app prepares a writable fallback file at `PREFIX/etc/resolv.conf`
- the fallback is injected only for Android tunnel sidecars and does not change desktop behavior

## Why This Fix Was Needed

On Android, packaged apps commonly run without:

- a normal `/etc/resolv.conf`
- a local DNS listener on `:53`

`gust` and `slider` already include Android/Termux-specific DNS fallback logic, but that logic depends on environment markers such as `TERMUX_VERSION` and `PREFIX`.

Before this release, `zashboard-wsf` started the sidecars without that environment, so their Android fallback path was never activated.

## Effective Fallback Chain

For Android tunnel sidecars in this release, DNS lookup now follows the intended order:

1. system `/etc/resolv.conf`
2. app-provided `PREFIX/etc/resolv.conf`
3. sidecar built-in public DNS fallback

Current app-provided fallback nameservers:

- `8.8.8.8`
- `1.1.1.1`

## Scope

This release is intentionally narrow:

- fixed: Android built-in tunnel domain resolution
- unchanged: desktop tunnel behavior
- unchanged: UI switching, backend management, release target matrix

## Android Background Behavior

This release improves Android tunnel DNS compatibility, but it does not change the project's background-service model.

Users should understand the current scope clearly:

- built-in `gust` / `slider` tunnels can continue working in background if Android keeps the app process alive
- the project is not implemented as an Android foreground service
- no wake lock or service-grade background persistence is claimed
- long-duration localhost serving for external apps should be treated as best-effort only

Recommended usage:

- foreground usage is the most reliable
- PiP or keeping the app visible may be more stable than full backgrounding
- if a workflow requires strong long-duration background guarantees, a separate dedicated Android tunnel service app is the more appropriate design

## Release Targets

Validated CI targets for `v1.0.1` remain:

- Windows x64
- Windows x86
- macOS arm64
- Android APK
