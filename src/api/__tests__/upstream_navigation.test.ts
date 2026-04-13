/**
 * @vitest-environment jsdom
 */
import { describe, expect, it, vi } from 'vitest'
import { reloadForUiSwitch } from '@/api/upstream_navigation'

describe('upstream_navigation', () => {
  describe('reloadForUiSwitch', () => {
    it('calls window.location.reload', async () => {
      const reloadMock = vi.fn()
      Object.defineProperty(window, 'location', {
        value: { reload: reloadMock },
        writable: true,
        configurable: true,
      })

      await reloadForUiSwitch()
      expect(reloadMock).toHaveBeenCalledOnce()
    })
  })
})
