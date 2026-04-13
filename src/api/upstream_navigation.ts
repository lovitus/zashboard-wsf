// With the wsf:// protocol handler, upstream UI is served from the same origin.
// No localStorage transfer or origin switching is needed.
// Activate/deactivate just requires a page reload to pick up the new assets.

export async function reloadForUiSwitch(): Promise<void> {
  window.location.reload()
}
