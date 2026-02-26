/**
 * OAuth Callback Handler
 *
 * Processes the session token from AT Protocol OAuth callback
 * and sets the auth cookie before redirecting to portal.
 */

'use client'

import { Suspense, useEffect, useState, useRef } from 'react'
import { useRouter, useSearchParams } from 'next/navigation'
import { Loader2, CheckCircle, XCircle } from 'lucide-react'

function CallbackHandler() {
  const router = useRouter()
  const searchParams = useSearchParams()
  const [status, setStatus] = useState<'processing' | 'success' | 'error'>('processing')
  const [error, setError] = useState<string | null>(null)
  // Guard against double-execution from React Strict Mode
  const hasRun = useRef(false)

  useEffect(() => {
    // Prevent double execution
    if (hasRun.current) return
    hasRun.current = true

    async function handleCallback() {
      const sessionToken = searchParams.get('sessionToken')
      const sessionId = searchParams.get('sessionId')

      if (!sessionToken) {
        setError('Missing session token')
        setStatus('error')
        return
      }

      try {
        console.log('[Callback] Starting auth flow...')
        console.log('[Callback] Session token:', sessionToken?.substring(0, 20) + '...')

        // Save session token to HTTP-only cookie
        console.log('[Callback] Setting HTTP-only cookie...')
        const response = await fetch('/api/auth/session', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ sessionToken }),
        })

        console.log('[Callback] Cookie API response status:', response.status)

        if (!response.ok) {
          const errorData = await response.json()
          console.error('[Callback] Failed to save session:', errorData)
          throw new Error('Failed to save session')
        }

        console.log('[Callback] Cookie saved successfully')

        // Also store in localStorage for client-side reference
        localStorage.setItem('deka_session_token', sessionToken)
        if (sessionId) {
          localStorage.setItem('deka_session_id', sessionId)
        }

        console.log('[Callback] localStorage updated')

        setStatus('success')

        // Redirect to portal
        console.log('[Callback] Redirecting to /portal in 100ms...')
        setTimeout(() => {
          router.replace('/portal')
        }, 100)
      } catch (err) {
        console.error('Auth callback error:', err)
        setError(err instanceof Error ? err.message : 'Authentication failed')
        setStatus('error')
        hasRun.current = false // Allow retry on error
      }
    }

    handleCallback()
  }, [searchParams, router])

  return (
    <div className="text-center">
      {status === 'processing' && (
        <>
          <Loader2 className="w-12 h-12 text-blue-500 animate-spin mx-auto mb-4" />
          <h1 className="text-xl font-semibold text-zinc-100 mb-2">
            Completing sign in...
          </h1>
          <p className="text-zinc-400">Please wait while we verify your account.</p>
        </>
      )}

      {status === 'success' && (
        <>
          <CheckCircle className="w-12 h-12 text-green-500 mx-auto mb-4" />
          <h1 className="text-xl font-semibold text-zinc-100 mb-2">
            Sign in successful!
          </h1>
          <p className="text-zinc-400">Redirecting to your portal...</p>
        </>
      )}

      {status === 'error' && (
        <>
          <XCircle className="w-12 h-12 text-red-500 mx-auto mb-4" />
          <h1 className="text-xl font-semibold text-zinc-100 mb-2">
            Sign in failed
          </h1>
          <p className="text-zinc-400 mb-4">{error}</p>
          <button
            onClick={() => router.push('/signin')}
            className="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 text-zinc-100 rounded-lg transition-colors"
          >
            Try again
          </button>
        </>
      )}
    </div>
  )
}

function LoadingFallback() {
  return (
    <div className="text-center">
      <Loader2 className="w-12 h-12 text-blue-500 animate-spin mx-auto mb-4" />
      <h1 className="text-xl font-semibold text-zinc-100 mb-2">
        Loading...
      </h1>
    </div>
  )
}

export default function AuthCallbackPage() {
  return (
    <div className="min-h-screen bg-zinc-950 flex items-center justify-center">
      <Suspense fallback={<LoadingFallback />}>
        <CallbackHandler />
      </Suspense>
    </div>
  )
}
