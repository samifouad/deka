/**
 * User Server Actions
 *
 * Server-side functions for fetching user data using session tokens
 */

'use server'

import { cookies } from 'next/headers'
import type { DekaUser } from '@/types/auth'

const SESSION_COOKIE_NAME = 'deka_session'
const IDENTITY_URL = process.env.NEXT_PUBLIC_IDENTITY_URL || 'https://id.deka.gg'

/**
 * Get current authenticated user data
 * Returns user info based on session token in HTTP-only cookie
 */
export async function getCurrentUserData(): Promise<DekaUser | null> {
  try {
    console.log('[USER_ACTION] Getting current user data...')
    const cookieStore = await cookies()
    const token = cookieStore.get(SESSION_COOKIE_NAME)?.value

    console.log('[USER_ACTION] Cookie name:', SESSION_COOKIE_NAME)
    console.log('[USER_ACTION] Token found:', token ? token.substring(0, 20) + '...' : 'NONE')

    if (!token) {
      console.error('[USER_ACTION] No session token in cookie')
      return null
    }

    // Fetch session data from deka-identity
    const url = `${IDENTITY_URL}/auth/session/${token}`
    console.log('[USER_ACTION] Fetching from:', url)

    const response = await fetch(url, {
      cache: 'no-store',
    })

    console.log('[USER_ACTION] Response status:', response.status)

    if (!response.ok) {
      const errorText = await response.text()
      console.error('[USER_ACTION] Session fetch failed:', response.status, errorText)
      return null
    }

    const sessionData = await response.json()
    console.log('[USER_ACTION] Session data:', JSON.stringify(sessionData, null, 2))

    const userData = {
      address: sessionData.address || '',
      username: sessionData.username || null,
      role: 'member', // Default role for website users
      avatar_url: null,
    }

    console.log('[USER_ACTION] Returning user data:', userData)
    return userData
  } catch (error) {
    console.error('[USER_ACTION] Error fetching user data:', error)
    return null
  }
}

/**
 * Check if user is authenticated
 */
export async function isUserAuthenticated(): Promise<boolean> {
  try {
    const userData = await getCurrentUserData()
    return userData !== null
  } catch {
    return false
  }
}

/**
 * Clear session token (logout)
 */
export async function clearSessionToken(): Promise<void> {
  const cookieStore = await cookies()
  cookieStore.delete(SESSION_COOKIE_NAME)
}
