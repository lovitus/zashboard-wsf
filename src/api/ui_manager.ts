import { invoke } from '@tauri-apps/api/core'

export interface UpstreamRelease {
  tag_name: string
  name: string
  published_at: string
  html_url: string
  assets: UpstreamAsset[]
}

export interface UpstreamAsset {
  name: string
  size: number
  browser_download_url: string
}

export interface UiVersionInfo {
  active_version: string | null
  downloaded_versions: DownloadedVersion[]
}

export interface DownloadedVersion {
  tag: string
  size_bytes: number
}

export async function uiFetchReleases(): Promise<UpstreamRelease[]> {
  return invoke<UpstreamRelease[]>('ui_fetch_releases')
}

export async function uiDownloadVersion(tag: string): Promise<string> {
  return invoke<string>('ui_download_version', { tag })
}

export async function uiActivateVersion(tag: string): Promise<string> {
  return invoke<string>('ui_activate_version', { tag })
}

export async function uiDeactivate(): Promise<string> {
  return invoke<string>('ui_deactivate')
}

export async function uiGetInfo(): Promise<UiVersionInfo> {
  return invoke<UiVersionInfo>('ui_get_info')
}

export async function uiDeleteVersion(tag: string): Promise<string> {
  return invoke<string>('ui_delete_version', { tag })
}
