/**
 * Authentication utilities for Deka Dashboard
 *
 * Uses deka-identity for both QR-based and Bluesky OAuth authentication,
 * then verifies the user is authorized on this Deka instance via deka_users table.
 */

import type { SessionTokenData } from '@/types/auth'

const TOKEN_STORAGE_KEY = 'deka_session_token'
const USER_ID_STORAGE_KEY = 'deka_user_id'
const USERNAME_STORAGE_KEY = 'deka_username'
const ADDRESS_STORAGE_KEY = 'deka_address'
const TOKEN_EXPIRY_STORAGE_KEY = 'deka_token_expiry'

/**
 * Get the deka-identity API URL from environment
 * This is the local identity service for Deka instances
 */
export function getIdentityApiUrl(): string {
  return process.env.NEXT_PUBLIC_IDENTITY_API_URL || 'http://localhost:8524'
}

/**
 * Get the deka-api URL from environment
 * Used for authorization checks and API calls
 */
export function getDekaApiUrl(): string {
  return process.env.DEKA_API_URL || 'http://localhost:8520'
}

/**
 * Save session token and user data to localStorage
 */
export function saveSession(data: {
  sessionToken: string
  userId: string
  username: string
  address: string
  expiresAt?: string
}): void {
  if (typeof window === 'undefined') return

  localStorage.setItem(TOKEN_STORAGE_KEY, data.sessionToken)
  localStorage.setItem(USER_ID_STORAGE_KEY, data.userId)
  localStorage.setItem(USERNAME_STORAGE_KEY, data.username)
  localStorage.setItem(ADDRESS_STORAGE_KEY, data.address)

  if (data.expiresAt) {
    localStorage.setItem(TOKEN_EXPIRY_STORAGE_KEY, data.expiresAt)
  }
}

/**
 * Get session token data from localStorage
 */
export function getSession(): SessionTokenData | null {
  if (typeof window === 'undefined') return null

  const token = localStorage.getItem(TOKEN_STORAGE_KEY)
  const userId = localStorage.getItem(USER_ID_STORAGE_KEY)
  const username = localStorage.getItem(USERNAME_STORAGE_KEY)
  const address = localStorage.getItem(ADDRESS_STORAGE_KEY)
  const expiresAt = localStorage.getItem(TOKEN_EXPIRY_STORAGE_KEY)

  if (!token || !userId || !address) {
    return null
  }

  return {
    token,
    userId,
    username: username || '',
    address,
    expiresAt: expiresAt || new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
  }
}

/**
 * Clear session data from localStorage
 */
export function clearSession(): void {
  if (typeof window === 'undefined') return

  localStorage.removeItem(TOKEN_STORAGE_KEY)
  localStorage.removeItem(USER_ID_STORAGE_KEY)
  localStorage.removeItem(USERNAME_STORAGE_KEY)
  localStorage.removeItem(ADDRESS_STORAGE_KEY)
  localStorage.removeItem(TOKEN_EXPIRY_STORAGE_KEY)
}

/**
 * Check if user is authenticated (localStorage check only)
 * For server-side auth, use the UserContext
 */
export function isAuthenticated(): boolean {
  const session = getSession()

  if (!session) return false

  // Check if token is expired
  const expiryDate = new Date(session.expiresAt)
  if (expiryDate < new Date()) {
    clearSession()
    return false
  }

  return true
}
