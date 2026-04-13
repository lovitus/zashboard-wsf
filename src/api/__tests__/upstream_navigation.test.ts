/**
 * @vitest-environment jsdom
 */
import { describe, expect, it } from 'vitest'
import { reloadForUiSwitch } from '@/api/upstream_navigation'

describe('upstream_navigation', () => {
  describe('reloadForUiSwitch', () => {
    it('navigates to wsf origin', async () => {
      const loc = { href: '' }
      Object.defineProperty(window, 'location', {
        value: loc,
        writable: true,
        configurable: true,
      })

      await reloadForUiSwitch()
      expect(loc.href).toBe('http://wsf.localhost/')
    })
  })
})
