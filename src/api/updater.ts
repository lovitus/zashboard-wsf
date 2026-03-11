import { isTauri } from '@/api/tunnel'

export interface GithubRelease {
  tag_name: string
  name: string
  body: string
  published_at: string
  draft: boolean
  prerelease: boolean
  html_url: string
  assets: GithubAsset[]
}

export interface GithubAsset {
  name: string
  browser_download_url: string
  size: number
}

const DEFAULT_RELEASES_URL = 'https://api.github.com/repos/lovitus/zashboard-wsf/releases'
const CUSTOM_URL_KEY = 'zashboard_custom_update_url'

export function getCustomUpdateUrl(): string {
  return localStorage.getItem(CUSTOM_URL_KEY) || ''
}

export function setCustomUpdateUrl(url: string) {
  if (url.trim()) {
    localStorage.setItem(CUSTOM_URL_KEY, url.trim())
  } else {
    localStorage.removeItem(CUSTOM_URL_KEY)
  }
}

export async function fetchReleases(signal?: AbortSignal): Promise<GithubRelease[]> {
  const custom = getCustomUpdateUrl()
  const url = custom || DEFAULT_RELEASES_URL

  const res = await fetch(url, {
    headers: { Accept: 'application/vnd.github.v3+json' },
    signal,
  })

  if (!res.ok) {
    if (res.status === 403) {
      throw new Error('GitHub API rate limit exceeded. Try again later.')
    }
    throw new Error(`HTTP ${res.status}: ${res.statusText}`)
  }

  const data = await res.json()
  const releases: GithubRelease[] = Array.isArray(data) ? data : [data]
  return releases.filter((r) => !r.draft)
}

export function compareVersions(a: string, b: string): number {
  const parse = (v: string) =>
    v
      .replace(/^v/, '')
      .split(/[.\-]/)
      .map((p) => {
        const n = parseInt(p, 10)
        return isNaN(n) ? 0 : n
      })
  const pa = parse(a)
  const pb = parse(b)
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const na = pa[i] || 0
    const nb = pb[i] || 0
    if (na !== nb) return na - nb
  }
  return 0
}

export async function getCurrentVersion(): Promise<string> {
  if (!isTauri) return '0.0.0'
  try {
    const { getVersion } = await import('@tauri-apps/api/app')
    return await getVersion()
  } catch {
    return '0.0.0'
  }
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

export function getPlatformKeywords(): string[] {
  const ua = navigator.userAgent.toLowerCase()
  if (ua.includes('android')) return ['android', '.apk']
  if (ua.includes('win')) return ['windows', '.exe', 'nsis']
  if (ua.includes('mac')) {
    // Detect arm vs x86
    // navigator.platform is deprecated but still works
    const platform = (navigator as { platform?: string }).platform?.toLowerCase() || ''
    if (platform.includes('arm') || ua.includes('arm')) {
      return ['darwin', 'aarch64', 'arm64', '.dmg']
    }
    return ['darwin', 'x86_64', '.dmg']
  }
  if (ua.includes('linux')) return ['linux', '.appimage', '.deb']
  return []
}

export function filterAssetsForPlatform(assets: GithubAsset[]): GithubAsset[] {
  const keywords = getPlatformKeywords()
  if (keywords.length === 0) return assets

  return assets.filter((a) => {
    const name = a.name.toLowerCase()
    return keywords.some((kw) => name.includes(kw))
  })
}

export async function openUrl(url: string) {
  try {
    const { open } = await import('@tauri-apps/plugin-shell')
    await open(url)
  } catch {
    window.open(url, '_blank')
  }
}
