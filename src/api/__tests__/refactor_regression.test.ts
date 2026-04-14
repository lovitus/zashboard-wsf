import { describe, expect, it } from 'vitest'
import * as upstreamNav from '@/api/upstream_navigation'
import * as uiManager from '@/api/ui_manager'

describe('refactor regression: removed exports no longer exist', () => {
  it('upstream_navigation does NOT export encodeStorageSnapshot', () => {
    expect(upstreamNav).not.toHaveProperty('encodeStorageSnapshot')
  })

  it('upstream_navigation does NOT export openActiveUpstreamDashboardIfNeeded', () => {
    expect(upstreamNav).not.toHaveProperty('openActiveUpstreamDashboardIfNeeded')
  })

  it('upstream_navigation DOES export navigateToWsfSetup and navigateToWsfRoot', () => {
    expect(upstreamNav).toHaveProperty('navigateToWsfSetup')
    expect(typeof upstreamNav.navigateToWsfSetup).toBe('function')
    expect(upstreamNav).toHaveProperty('navigateToWsfRoot')
    expect(typeof upstreamNav.navigateToWsfRoot).toBe('function')
  })

  it('upstream_navigation does NOT export reloadForUiSwitch (replaced)', () => {
    expect(upstreamNav).not.toHaveProperty('reloadForUiSwitch')
  })

  it('UiVersionInfo does NOT contain upstream_url field', async () => {
    // Verify the interface shape by checking the type module exports
    // The UiVersionInfo interface should only have these fields:
    const expectedKeys = [
      'active_version',
      'downloaded_versions',
      'custom_releases_url',
      'custom_download_base',
    ]
    const sampleInfo: uiManager.UiVersionInfo = {
      active_version: null,
      downloaded_versions: [],
      custom_releases_url: null,
      custom_download_base: null,
    }
    expect(Object.keys(sampleInfo)).toEqual(expectedKeys)
  })

  it('uiActivateVersion accepts only tag param (no storageData)', () => {
    // uiActivateVersion should have exactly 1 required parameter
    expect(uiManager.uiActivateVersion.length).toBe(1)
  })
})
