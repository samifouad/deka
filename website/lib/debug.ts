/**
 * Debug utilities for browser console
 *
 * These functions can be called from the browser console to help debug
 * authentication issues and clear stale data.
 *
 * Usage in browser console:
 * ```
 * window.dekaDebug.clearAuth()
 * window.dekaDebug.showAuthState()
 * ```
 */

import { clearSession, getSession } from './auth'

export const dekaDebug = {
  /**
   * Clear all authentication data (localStorage + cookies)
   */
  clearAuth() {
    console.log('%c[DEKA_DEBUG] Clearing all auth data...', 'color: #ff6b6b; font-weight: bold')
    clearSession()
    fetch('/api/auth/session', { method: 'DELETE' })
      .then(() => {
        console.log('%c[DEKA_DEBUG] Auth cleared. Refreshing page...', 'color: #ff6b6b; font-weight: bold')
        setTimeout(() => window.location.reload(), 500)
      })
      .catch((err) => console.error('[DEKA_DEBUG] Failed to clear session:', err))
  },

  /**
   * Show current authentication state
   */
  showAuthState() {
    console.log('%c[DEKA_DEBUG] Current Auth State:', 'color: #4ecdc4; font-weight: bold')

    // Check localStorage
    const localStorageData = {
      sessionToken: localStorage.getItem('deka_session_token'),
      userId: localStorage.getItem('deka_user_id'),
      username: localStorage.getItem('deka_username'),
      address: localStorage.getItem('deka_address'),
      tokenExpiry: localStorage.getItem('deka_token_expiry'),
    }
    console.log('localStorage:', localStorageData)

    // Check session helper
    const session = getSession()
    console.log('Session (from getSession()):', session)

    // Check cookies
    const cookies = document.cookie.split(';').reduce((acc, cookie) => {
      const [key, value] = cookie.trim().split('=')
      if (key.startsWith('deka_')) {
        acc[key] = value
      }
      return acc
    }, {} as Record<string, string>)
    console.log('Cookies (deka_*):', cookies)

    // Check if expired
    if (session?.expiresAt) {
      const expiryDate = new Date(session.expiresAt)
      const now = new Date()
      const isExpired = expiryDate < now
      console.log('Token expiry:', {
        expiresAt: expiryDate.toISOString(),
        now: now.toISOString(),
        isExpired,
        minutesUntilExpiry: isExpired ? 'EXPIRED' : Math.round((expiryDate.getTime() - now.getTime()) / 1000 / 60),
      })
    }
  },

  /**
   * Clear only localStorage
   */
  clearLocalStorage() {
    console.log('%c[DEKA_DEBUG] Clearing localStorage only...', 'color: #ffd93d; font-weight: bold')
    clearSession()
    console.log('%c[DEKA_DEBUG] localStorage cleared', 'color: #ffd93d; font-weight: bold')
  },

  /**
   * Clear only cookies
   */
  async clearCookies() {
    console.log('%c[DEKA_DEBUG] Clearing cookies only...', 'color: #ffd93d; font-weight: bold')
    try {
      await fetch('/api/auth/session', { method: 'DELETE' })
      console.log('%c[DEKA_DEBUG] Cookies cleared', 'color: #ffd93d; font-weight: bold')
    } catch (err) {
      console.error('[DEKA_DEBUG] Failed to clear cookies:', err)
    }
  },

  /**
   * Help text
   */
  help() {
    console.log('%c[DEKA_DEBUG] Available Commands:', 'color: #4ecdc4; font-weight: bold')
    console.log(`
Available debug commands:

  dekaDebug.showAuthState()     - Show current authentication state
  dekaDebug.clearAuth()          - Clear all auth data (localStorage + cookies) and reload
  dekaDebug.clearLocalStorage()  - Clear only localStorage
  dekaDebug.clearCookies()       - Clear only HTTP-only cookies
  dekaDebug.help()               - Show this help text

Examples:

  // Check why redirect loop is happening
  dekaDebug.showAuthState()

  // Clear everything and start fresh
  dekaDebug.clearAuth()
`)
  },
}

// Make it available globally in development
if (typeof window !== 'undefined') {
  ;(window as any).dekaDebug = dekaDebug

  // Only log once on page load
  if (!(window as any).__dekaDebugLoaded) {
    console.log(
      '%cDeka Debug Tools Available',
      'color: #4ecdc4; font-weight: bold; font-size: 14px; background: #1a1a1a; padding: 8px 16px; border-radius: 4px;'
    )
    console.log(
      '%cType "dekaDebug.help()" for available commands',
      'color: #95e1d3; font-size: 12px;'
    )
    ;(window as any).__dekaDebugLoaded = true
  }
}
