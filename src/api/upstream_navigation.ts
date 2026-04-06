import { uiActivateVersion, uiGetInfo } from '@/api/ui_manager'
import { isTauri } from '@/api/tunnel'

function encodeStorageSnapshot(): string {
  const storageEntries: Record<string, string> = {}
  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i)
    if (!key) continue
    const val = localStorage.getItem(key)
    if (val !== null) {
      storageEntries[key] = val
    }
  }
  return btoa(encodeURIComponent(JSON.stringify(storageEntries)))
}

function measureSafeAreaInsets(): string | null {
  try {
    const probe = document.createElement('div')
    probe.style.cssText =
      'position:fixed;top:env(safe-area-inset-top);right:env(safe-area-inset-right);bottom:env(safe-area-inset-bottom);left:env(safe-area-inset-left);pointer-events:none;visibility:hidden;'
    document.body.appendChild(probe)
    const cs = getComputedStyle(probe)
    const top = parseFloat(cs.top) || 0
    const right = parseFloat(cs.right) || 0
    const bottom = parseFloat(cs.bottom) || 0
    const left = parseFloat(cs.left) || 0
    document.body.removeChild(probe)
    if (top === 0 && right === 0 && bottom === 0 && left === 0) return null
    return `${Math.round(top)},${Math.round(right)},${Math.round(bottom)},${Math.round(left)}`
  } catch {
    return null
  }
}

export async function openActiveUpstreamDashboardIfNeeded(): Promise<boolean> {
  if (!isTauri) return false

  try {
    const info = await uiGetInfo()
    if (!info.active_version) return false

    const storageB64 = encodeStorageSnapshot()
    const safeAreaInsets = measureSafeAreaInsets()
    const url = await uiActivateVersion(info.active_version, storageB64, safeAreaInsets)

    if (url && /^https?:\/\//.test(url)) {
      window.location.href = url
      return true
    }
  } catch (e) {
    console.error('Failed to open upstream dashboard:', e)
  }

  return false
}

export { encodeStorageSnapshot, measureSafeAreaInsets }
