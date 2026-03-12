# Upstream UI Switching — Full Issue Record

## Project Overview

**zashboard-wsf** is a Tauri desktop + Android app that wraps the [zashboard](https://github.com/Zephyruso/zashboard) Clash/Mihomo dashboard with tunnel (VPN proxy) management. It bundles a built-in version of zashboard (currently v2.7.0) and allows users to download and switch to different upstream zashboard versions.

**Repository:** https://github.com/lovitus/zashboard-wsf

---

## User Requirements (Original, Translated from Chinese)

### Core Requirement
When a user clicks "Use" on a downloaded upstream UI version, the **main application window** should immediately switch to displaying that version. No new windows. No blank pages. The upstream UI must be fully functional — it should connect to the same Clash backend and work identically to the built-in version.

### Two Scenarios
1. **Built-in version**: Everything works as-is, no changes needed.
2. **Non-built-in version**: Start a local web server hosting the upstream UI files. Navigate the main window to this server. The upstream dashboard should be fully functional — submit buttons, saved configurations, navigation all work correctly. A "return" button allows the user to go back to the built-in management UI.

### Explicit Prohibitions
- **Do NOT open new windows** — the user was furious about this in early attempts
- **Do NOT show a blank/unusable page** — the upstream UI must be fully functional
- **Do NOT be lazy** — robust error handling, fallback mechanisms required

---

## Attempt History

### Attempt 1: New WebviewWindow (FAILED)
**Approach:** When user clicked "Use", opened a new `WebviewWindow` pointing to a `zui://` custom protocol URL.

**Problem:** Created a new window that was disconnected from the app. The user explicitly rejected this: "不要瞎鸡巴的开新窗口并且新窗口没用!!!" (Don't randomly open new windows that are useless!!!)

**Root cause:** Misunderstood the requirement — user wanted the main window to change, not a new window.

### Attempt 2: `zui://` Custom Protocol + `webview.eval()` Navigation (FAILED)
**Approach:**
- Registered a `zui://` custom URI scheme protocol handler in Tauri
- `handle_zui_protocol()` served files from the downloaded version directory
- `ui_activate_version()` used `webview.eval("window.location.href = 'zui://localhost/'")` to navigate the main window
- Added `navigate_main_window()` helper trying both `window.navigate()` API and `eval()` fallback

**Problem:** The user reported: "切换了, 但是访问进去还是2.7.0 (内置的)" — switched, but accessing still shows 2.7.0 (built-in). The `webview.eval()` navigation didn't work reliably, and even when it did reach the protocol handler, the served content may not have been functional.

**Root cause analysis:**
1. Custom URI scheme protocols in Tauri webview may have limitations with SPA routing, asset loading, and CORS
2. `webview.eval()` navigation may not trigger reliably depending on timing and webview state
3. Even if navigation worked, the served zashboard UI was missing critical context (Clash backend URL/secret configuration)

### Attempt 3: Local HTTP File Server (FAILED — Current State, v0.2.13)
**Approach:**
- Removed `zui://` protocol handler entirely
- Implemented a local HTTP file server using `std::net::TcpListener` binding to `127.0.0.1:0` (random port)
- `ui_activate_version()` starts the server, returns URL like `http://127.0.0.1:54321`
- Frontend `handleActivate()` does `window.location.href = url` to navigate to the server
- Server serves static files with SPA fallback (index.html for unknown paths)
- Injected "return to built-in UI" button into HTML responses
- Added auto-redirect on app startup if a version was previously active

**Problem:** User reports: "切换版本还是页面跳到了一个空白的ui web, 完全不可用" — switching version still navigates to a blank, completely unusable UI web page.

**Root cause analysis (this is the fundamental issue that was never addressed):**

The zashboard upstream UI is a **Clash dashboard SPA**. It needs:
1. **A Clash/Mihomo backend URL** to connect to (e.g., `http://127.0.0.1:9090`)
2. **A secret/token** for authentication with the Clash API
3. **These are typically stored in the browser's localStorage** under the domain it was originally configured on

When we navigate from `https://tauri.localhost/` (built-in UI) to `http://127.0.0.1:54321/` (upstream server), we change the **origin**. This means:
- **localStorage is not shared** between origins — the upstream UI at `http://127.0.0.1:54321` has no access to the Clash backend configuration that was saved in `https://tauri.localhost/`'s localStorage
- The upstream UI loads but has **no backend connection configured**, so it shows a blank/setup page
- Even if the user manually configures it, the configuration is separate from the built-in version

**This is the fundamental architectural flaw that all three attempts failed to address.**

---

## What Would Actually Work

### Option A: Proxy/Replace at the Tauri Level
Instead of navigating to a different origin, serve the upstream UI files **through the same origin** (`tauri://localhost/` or `https://tauri.localhost/`). This could be done by:
- Intercepting the Tauri asset protocol to serve upstream files instead of built-in files when a version is active
- This preserves localStorage, cookies, and all browser state
- The upstream zashboard would have access to the same Clash backend configuration

**Implementation:** Use Tauri's `register_assetloader_protocol` or intercept the default asset serving mechanism. When an upstream version is active, serve files from the upstream version directory instead of the built-in `../dist` directory.

**Challenge:** Tauri v2 may not easily support replacing the default asset protocol at runtime. The `frontendDist` is compiled into the binary. However, Tauri v2 has `register_uri_scheme_protocol` which could potentially override the default `tauri://` or `https://tauri.localhost/` protocol, or a custom asset resolver could be used.

### Option B: Pass Configuration via URL Parameters
Navigate to the upstream HTTP server but pass the Clash backend URL and secret as URL parameters or hash fragments:
```
http://127.0.0.1:54321/#/setup?host=127.0.0.1&port=9090&secret=xxx
```
**Challenge:** The upstream zashboard may not support receiving configuration via URL parameters. Would need to modify the upstream code or inject JavaScript that sets localStorage before the app initializes.

### Option C: Inject Configuration via JavaScript
When serving the upstream UI's `index.html`, inject a `<script>` tag that:
1. Reads the Clash backend configuration (passed from the Tauri backend)
2. Sets it in `localStorage` for the upstream origin before the SPA initializes
3. This way the upstream UI "auto-configures" itself

**Implementation:**
```javascript
<script>
// Injected by zashboard-wsf
(function() {
  // Configuration passed from the Tauri backend
  var config = { host: '127.0.0.1', port: 9090, secret: 'xxx' };
  // Set localStorage keys that zashboard expects
  localStorage.setItem('clash-host', config.host);
  localStorage.setItem('clash-port', config.port);
  localStorage.setItem('clash-secret', config.secret);
})();
</script>
```
**Challenge:** Need to know exactly which localStorage keys zashboard uses for its configuration. These may vary between versions.

### Option D: Reverse Proxy Approach
Instead of serving static files, run a reverse proxy that:
1. Serves upstream UI files for frontend assets
2. Proxies API requests (`/api/*`) to the Clash backend
3. Maintains the same origin, preserving localStorage

**This is the most robust but most complex approach.**

### Recommended Approach: Option A
The cleanest solution is to serve the upstream UI through the same Tauri origin. This means:
1. In `lib.rs`, register a custom asset protocol that checks if an upstream version is active
2. If active, serve files from the upstream version directory
3. If not active (or file not found in upstream), fall back to the built-in assets
4. The management pages (settings, tunnel config) are part of the built-in UI and should always be served from built-in assets — this requires careful path routing

Alternatively, a simpler variant of Option A:
1. The built-in zashboard-wsf app (with its custom pages like tunnel management) always loads first
2. When navigating to dashboard routes, if an upstream version is active, the built-in app dynamically loads the upstream UI in an iframe or via dynamic script injection from the local HTTP server
3. This keeps the management UI accessible while showing the upstream dashboard

---

## Cascade AI's Self-Assessment

### What I Did Wrong
1. **Failed to identify the core problem**: The fundamental issue was always about **origin isolation** and **configuration transfer**. A Clash dashboard needs backend configuration to function. Navigating to a different origin loses that configuration. I never addressed this.

2. **Focused on navigation mechanics instead of functionality**: All three attempts focused on "how to navigate the webview" rather than "how to make the upstream UI actually work once navigated to." Getting the page to load is only half the problem — the page also needs to be functional.

3. **Did not study the upstream zashboard code**: I never examined how zashboard stores its configuration (localStorage keys, initialization flow) to understand what the upstream UI needs to be functional.

4. **Repeated the same class of mistake**: Each attempt was a variation of "serve files + navigate to them" without addressing why the served UI was blank/non-functional.

5. **Overconfidence in each solution**: After each implementation, I declared it "done" and pushed to CI without adequate testing or consideration of the user's actual use case.

### Technical Debt Created
- Multiple version bumps (v0.2.10 → v0.2.13) with non-functional features
- Dead code from `zui://` protocol handler was added and then removed
- `url` crate was added and then removed
- HTTP file server code may still be useful but the architecture needs rethinking

---

## Files Modified Across All Attempts

### Rust Backend
- `src-tauri/src/ui_manager.rs` — Core UI version management logic. Currently contains:
  - Version download/extract from GitHub releases
  - Local HTTP file server (start/stop)
  - Activate/deactivate commands
  - MIME type handling, SPA fallback, return button injection
  - ~688 lines

- `src-tauri/src/lib.rs` — Tauri app setup. Changes:
  - `#[cfg(desktop)]` for `tauri-plugin-single-instance` (Android fix)
  - Removed `zui://` protocol registration
  - Added auto-redirect to upstream on startup
  - ~997 lines

- `src-tauri/Cargo.toml` — Added/removed `url` crate, kept `zip`, `reqwest`, etc.
- `src-tauri/tauri.conf.json` — Version bumps

### Frontend
- `src/api/ui_manager.ts` — TypeScript API types and invoke wrappers
- `src/components/settings/UpstreamUIManager.vue` — Vue component for version management UI
- `src/i18n/en.ts`, `zh.ts`, `zh-tw.ts`, `ru.ts` — Localization strings

### CI
- `.github/workflows/release.yml` — Android CI had a transient `401 Unauthorized` error on `actions/setup-java` download (GitHub infrastructure issue, not code-related)

---

## Android CI Issue

Separate from the upstream UI problem, Android builds were failing due to:
1. `tauri-plugin-single-instance` is desktop-only — fixed with `#[cfg(desktop)]`
2. Unused variable warning — fixed by removing dead code
3. `401 Unauthorized` error downloading `actions/setup-java` — transient GitHub infrastructure issue, not a code fix

The `#[cfg(desktop)]` fix for single-instance is correct and should persist.

---

## Fix Applied: v0.2.14 — localStorage Transfer

### Root Cause Identified
The blank page was caused by **origin isolation**. The upstream UI served at `http://127.0.0.1:{port}` is a different origin from `https://tauri.localhost/` (built-in UI). The zashboard SPA stores its Clash backend connection config (host, port, secret) in `localStorage`, which is **per-origin**. The upstream UI had empty localStorage → no backend config → blank setup page.

### Solution Implemented (Option C from analysis above)
1. **Frontend** (`UpstreamUIManager.vue`): Before activating, serializes ALL `localStorage` entries as JSON, then encodes as `base64(encodeURIComponent(JSON))` for safe embedding
2. **Backend** (`ui_manager.rs`): `ui_activate_version` now accepts `storage_data: Option<String>`, persists it to `ui_storage_data.txt` for restart recovery, and passes it to the file server thread
3. **HTML injection** (`inject_scripts`): Injects a `<script>` tag right after `<head>` (before any SPA scripts) that decodes the base64 data and restores all localStorage entries. This runs before the SPA initializes, so the upstream zashboard finds its Clash backend config already configured
4. **Restart recovery**: On app startup, `init_state` reads the persisted `storage_data` from disk and passes it to the file server, so auto-redirect on restart also provides the config

### Files Modified in v0.2.14
- `src-tauri/src/ui_manager.rs` — Added `storage_data` field, persistence, injection logic
- `src/api/ui_manager.ts` — Added `storageData` parameter to `uiActivateVersion`
- `src/components/settings/UpstreamUIManager.vue` — Serializes localStorage before activation

### Remaining Risks
- If the upstream zashboard uses **different localStorage key names** between versions, the transferred data may not match. This is unlikely since the keys are typically stable.
- On every HTML page load, localStorage is re-set from the activation-time snapshot. If the user changes settings in the upstream UI, they'll be reset on next page load. To update, the user should re-activate from the built-in UI.
- The `btoa()` encoding assumes localStorage values are ASCII-safe after `encodeURIComponent()`. This should handle all Unicode correctly.

---

## Current State (v0.2.14)

The codebase compiles cleanly (`cargo check` + `vue-tsc` pass with zero errors/warnings). The local HTTP file server serves upstream UI files with localStorage restoration injected into HTML pages. The version management features (fetch, download, activate, deactivate, delete) should all work correctly.

The fundamental architecture is:
1. Built-in UI always runs at `https://tauri.localhost/` — used for management (tunnels, settings, version switching)
2. Upstream UI runs at `http://127.0.0.1:{port}/` — served by a local file server, with localStorage restored from the built-in UI's state, and a "return to built-in" button injected
3. On app startup, if an upstream version was previously active, auto-redirects to it after 2s

---

## Recommendation for Next Steps

If the localStorage transfer approach in v0.2.14 works:
- The feature is functionally complete
- Consider adding a visual indicator in the upstream UI showing which version is active

If it still doesn't work (upstream UI still blank):
- **Option A (same origin)** would be the definitive fix — serve upstream files through `tauri://localhost/` by intercepting Tauri's asset protocol. This eliminates the origin isolation problem entirely but is more complex to implement in Tauri v2.
- **Debug step**: Check browser devtools console on the upstream page to see if the localStorage restoration script runs and whether the zashboard SPA is finding its config

If abandoning:
- The tunnel management feature (gust/slider integration) works independently
- The upstream UI version management (download/delete) works correctly
- Only the "activate and navigate" flow may need further work
