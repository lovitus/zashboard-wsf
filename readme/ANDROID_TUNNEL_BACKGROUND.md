# Android Tunnel Background Notes

This document records the current design boundary for built-in `gust` / `slider` tunnel usage on Android.

## Short Answer

The current Android implementation can be used successfully in foreground mode and may continue working in background mode for some time, but it is not designed or promised to behave like a dedicated long-running Android tunnel service.

## What The App Does Today

- starts `gust` / `slider` as child processes from the Tauri host app
- supports Android DNS fallback for sidecar tunnel processes
- can preserve a running tunnel while the app is backgrounded if Android leaves the process alive
- can restart dead tunnel processes when the app returns to the built-in setup flow

## What The App Does Not Do

- no Android foreground service
- no persistent service notification for tunnel lifetime
- no wake lock strategy
- no claim of guaranteed long-term pure-background localhost service for other apps

## Practical Meaning

If you start a tunnel and then switch to another app:

- it may continue serving localhost traffic for a while
- it may continue serving for a long time on some devices
- it may also be interrupted if Android or the device vendor aggressively limits background processes

There is no fixed timeout in project logic that intentionally stops an otherwise healthy tunnel after a certain number of minutes.

The main uncertainty is Android process management, not an explicit wrapper-side countdown.

## Recovery Expectations

Current recovery is best understood as foreground-oriented:

- if the tunnel process stays alive, localhost access can continue working
- if the tunnel process dies while the app is backgrounded, automatic recovery is not guaranteed in background
- after returning to the built-in setup flow, the app can perform tunnel status checks and attempt restart

This is intentionally battery-conscious behavior, not a service-grade background daemon.

## Recommended Usage

Use the current app when you want:

- normal foreground dashboard usage
- tunnel-assisted backend access during active app use
- occasional short background continuation as a convenience

Do not rely on the current app when you need:

- guaranteed long-duration Android background tunnel serving
- service-like localhost availability for external apps over many hours
- system-level persistence comparable to a VPN or dedicated proxy service

## If Stronger Guarantees Are Needed

The cleaner long-term path is a separate Android-focused tunnel service app with:

- foreground service lifecycle
- dedicated notification
- explicit background policy
- tunnel health supervision designed independently from the dashboard UI

That architecture is intentionally outside the current scope of `zashboard-wsf`.
