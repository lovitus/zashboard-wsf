import { describe, expect, it, vi } from 'vitest'

// Mock @tauri-apps/api/core before importing the module under test
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
import {
  uiActivateVersion,
  uiDeactivate,
  uiDeleteVersion,
  uiDownloadVersion,
  uiFetchReleases,
  uiGetInfo,
  uiSetCustomUrls,
} from '@/api/ui_manager'
import type { UiVersionInfo, UpstreamRelease } from '@/api/ui_manager'

const mockedInvoke = vi.mocked(invoke)

describe('ui_manager API', () => {
  it('uiFetchReleases invokes ui_fetch_releases', async () => {
    const mockReleases: UpstreamRelease[] = [
      { tag_name: 'v1.0.0', name: 'Release 1', published_at: '', html_url: '', assets: [] },
    ]
    mockedInvoke.mockResolvedValueOnce(mockReleases)

    const result = await uiFetchReleases()
    expect(mockedInvoke).toHaveBeenCalledWith('ui_fetch_releases')
    expect(result).toEqual(mockReleases)
  })

  it('uiDownloadVersion invokes ui_download_version with tag', async () => {
    mockedInvoke.mockResolvedValueOnce('Downloaded version v1.0.0')

    const result = await uiDownloadVersion('v1.0.0')
    expect(mockedInvoke).toHaveBeenCalledWith('ui_download_version', { tag: 'v1.0.0' })
    expect(result).toBe('Downloaded version v1.0.0')
  })

  it('uiActivateVersion invokes ui_activate_version with tag only (no storageData)', async () => {
    mockedInvoke.mockResolvedValueOnce('Activated version v1.0.0')

    const result = await uiActivateVersion('v1.0.0')
    expect(mockedInvoke).toHaveBeenCalledWith('ui_activate_version', { tag: 'v1.0.0' })
    expect(result).toBe('Activated version v1.0.0')
  })

  it('uiDeactivate invokes ui_deactivate', async () => {
    mockedInvoke.mockResolvedValueOnce('Switched to built-in UI')

    const result = await uiDeactivate()
    expect(mockedInvoke).toHaveBeenCalledWith('ui_deactivate')
    expect(result).toBe('Switched to built-in UI')
  })

  it('uiGetInfo invokes ui_get_info and returns info without upstream_url', async () => {
    const mockInfo: UiVersionInfo = {
      active_version: 'v1.0.0',
      downloaded_versions: [{ tag: 'v1.0.0', size_bytes: 12345 }],
      custom_releases_url: null,
      custom_download_base: null,
    }
    mockedInvoke.mockResolvedValueOnce(mockInfo)

    const result = await uiGetInfo()
    expect(mockedInvoke).toHaveBeenCalledWith('ui_get_info')
    expect(result).toEqual(mockInfo)
    // Verify upstream_url is NOT in the interface
    expect(result).not.toHaveProperty('upstream_url')
  })

  it('uiDeleteVersion invokes ui_delete_version with tag', async () => {
    mockedInvoke.mockResolvedValueOnce('Deleted version v1.0.0')

    const result = await uiDeleteVersion('v1.0.0')
    expect(mockedInvoke).toHaveBeenCalledWith('ui_delete_version', { tag: 'v1.0.0' })
    expect(result).toBe('Deleted version v1.0.0')
  })

  it('uiSetCustomUrls invokes ui_set_custom_urls with normalized params', async () => {
    mockedInvoke.mockResolvedValueOnce('Custom URLs saved')

    const result = await uiSetCustomUrls('https://example.com', 'https://dl.example.com')
    expect(mockedInvoke).toHaveBeenCalledWith('ui_set_custom_urls', {
      releasesUrl: 'https://example.com',
      downloadBase: 'https://dl.example.com',
    })
    expect(result).toBe('Custom URLs saved')
  })

  it('uiSetCustomUrls sends null for empty strings', async () => {
    mockedInvoke.mockResolvedValueOnce('Custom URLs saved')

    await uiSetCustomUrls('', '')
    expect(mockedInvoke).toHaveBeenCalledWith('ui_set_custom_urls', {
      releasesUrl: null,
      downloadBase: null,
    })
  })
})
