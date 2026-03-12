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

export async function openActiveUpstreamDashboardIfNeeded(): Promise<boolean> {
  if (!isTauri) return false

  try {
    const info = await uiGetInfo()
    if (!info.active_version) return false

    const storageB64 = encodeStorageSnapshot()
    const url = await uiActivateVersion(info.active_version, storageB64)

    if (url && /^https?:\/\//.test(url)) {
      window.location.href = url
      return true
    }
  } catch (e) {
    console.error('Failed to open upstream dashboard:', e)
  }

  return false
}

export { encodeStorageSnapshot }
