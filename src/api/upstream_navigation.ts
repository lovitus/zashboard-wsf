// The wsf:// custom protocol serves upstream UI when active, bundled UI otherwise.
// On Windows/Android the origin is http://wsf.localhost/, on macOS/Linux wsf://localhost/.
// After activate/deactivate we navigate to the wsf origin so the protocol handler
// serves the correct assets. A plain reload() would stay on tauri://localhost/.

const WSF_ORIGIN = 'http://wsf.localhost/'

export function navigateToWsfSetup(): void {
  // Navigate to upstream UI's setup page so the user can choose a backend.
  // Without /#/setup the upstream SPA auto-connects to whichever backend was
  // stored in localStorage (shared origin), skipping the selection screen.
  window.location.href = WSF_ORIGIN + '#/setup'
}

export function navigateToWsfRoot(): void {
  window.location.href = WSF_ORIGIN
}
