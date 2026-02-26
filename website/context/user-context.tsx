/**
 * User Context Provider
 *
 * Provides user data and balances throughout the app
 */

'use client'

import React, { createContext, useContext, useState, useEffect, useCallback } from 'react'
import type { TanaUser, TanaBalance, SessionVerifyResponse } from '@/types/tana-api'
import { getCurrentUserData } from '@/actions/user'

// ============================================================================
// TYPES
// ============================================================================

interface UserContextValue {
  // Data
  session: SessionVerifyResponse | null
  user: TanaUser | null
  balances: TanaBalance[]

  // State
  isLoading: boolean
  error: string | null

  // Actions
  refreshUser: () => Promise<void>
  logout: () => void
}

// ============================================================================
// CONTEXT
// ============================================================================

const UserContext = createContext<UserContextValue | undefined>(undefined)

// ============================================================================
// PROVIDER COMPONENT
// ============================================================================

interface UserProviderProps {
  children: React.ReactNode
}

export function UserProvider({ children }: UserProviderProps) {
  const [session, setSession] = useState<SessionVerifyResponse | null>(null)
  const [user, setUser] = useState<TanaUser | null>(null)
  const [balances, setBalances] = useState<TanaBalance[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  /**
   * Fetch user data from server
   */
  const fetchUserData = useCallback(async () => {
    console.log('[USER_CONTEXT] Starting auth check...')
    setIsLoading(true)
    setError(null)

    try {
      const userData = await getCurrentUserData()

      if (userData) {
        console.log('[USER_CONTEXT] User authenticated:', userData.user.username)
        setSession(userData.session)
        setUser(userData.user)
        setBalances(userData.balances)
      } else {
        console.log('[USER_CONTEXT] No valid session found')
        // No user data means not authenticated or session expired
        // Clear any stale localStorage data
        if (typeof window !== 'undefined') {
          localStorage.removeItem('tana_session_token')
          localStorage.removeItem('tana_user_id')
          localStorage.removeItem('tana_username')
          localStorage.removeItem('tana_token_expiry')
          console.log('[USER_CONTEXT] Cleared stale localStorage data')
        }
        setSession(null)
        setUser(null)
        setBalances([])
      }
    } catch (err) {
      console.error('[USER_CONTEXT] Error fetching user data:', err)
      setError(err instanceof Error ? err.message : 'Failed to fetch user data')
      // Clear localStorage on error too
      if (typeof window !== 'undefined') {
        localStorage.removeItem('tana_session_token')
        localStorage.removeItem('tana_user_id')
        localStorage.removeItem('tana_username')
        localStorage.removeItem('tana_token_expiry')
      }
      setSession(null)
      setUser(null)
      setBalances([])
    } finally {
      console.log('[USER_CONTEXT] Auth check complete, isLoading = false')
      setIsLoading(false)
    }
  }, [])

  /**
   * Refresh user data
   */
  const refreshUser = useCallback(async () => {
    await fetchUserData()
  }, [fetchUserData])

  /**
   * Logout user
   */
  const logout = useCallback(() => {
    console.log('[USER_CONTEXT] Logging out user...')
    setSession(null)
    setUser(null)
    setBalances([])
    // Clear localStorage session data
    if (typeof window !== 'undefined') {
      localStorage.removeItem('tana_session_token')
      localStorage.removeItem('tana_user_id')
      localStorage.removeItem('tana_username')
      localStorage.removeItem('tana_token_expiry')
      console.log('[USER_CONTEXT] Cleared localStorage on logout')
    }
  }, [])

  // Fetch user data on mount - RUN ONLY ONCE
  useEffect(() => {
    console.log('[USER_CONTEXT] Mounting - will fetch user data once')
    fetchUserData()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []) // Empty deps - run ONLY on mount

  const value: UserContextValue = {
    session,
    user,
    balances,
    isLoading,
    error,
    refreshUser,
    logout,
  }

  return <UserContext.Provider value={value}>{children}</UserContext.Provider>
}

// ============================================================================
// HOOK
// ============================================================================

/**
 * Hook to access user context
 */
export function useUser() {
  const context = useContext(UserContext)

  if (context === undefined) {
    throw new Error('useUser must be used within a UserProvider')
  }

  return context
}

// ============================================================================
// UTILITY HOOKS
// ============================================================================

/**
 * Hook to check if user is authenticated
 */
export function useIsAuthenticated() {
  const { user, isLoading } = useUser()
  return { isAuthenticated: user !== null, isLoading }
}

/**
 * Hook to get a specific balance by currency code
 */
export function useBalance(currencyCode: string) {
  const { balances } = useUser()
  return balances.find((b) => b.currencyCode === currencyCode)
}

/**
 * Hook to get all balances grouped by type
 */
export function useBalancesByType() {
  const { balances } = useUser()

  // We don't have currency type in balances, so we'll return all balances
  // You could enhance this by fetching currency metadata
  return {
    all: balances,
    fiat: balances.filter((b) => ['USD', 'EUR', 'GBP'].includes(b.currencyCode)),
    crypto: balances.filter((b) => !['USD', 'EUR', 'GBP'].includes(b.currencyCode)),
  }
}
