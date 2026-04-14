/**
 * @vitest-environment jsdom
 */
import { describe, expect, it } from 'vitest'
import { navigateToWsfRoot, navigateToWsfSetup } from '@/api/upstream_navigation'

describe('upstream_navigation', () => {
  describe('navigateToWsfSetup', () => {
    it('navigates to wsf origin setup page', () => {
      const loc = { href: '' }
      Object.defineProperty(window, 'location', {
        value: loc,
        writable: true,
        configurable: true,
      })

      navigateToWsfSetup()
      expect(loc.href).toBe('http://wsf.localhost/#/setup')
    })
  })

  describe('navigateToWsfRoot', () => {
    it('navigates to wsf origin root', () => {
      const loc = { href: '' }
      Object.defineProperty(window, 'location', {
        value: loc,
        writable: true,
        configurable: true,
      })

      navigateToWsfRoot()
      expect(loc.href).toBe('http://wsf.localhost/')
    })
  })
})
