/**
 * Deka Sign In Page
 *
 * Server-rendered sign-in with Bluesky OAuth only.
 */

import { redirect } from 'next/navigation'
import { Server } from 'lucide-react'
import { BlueskyForm } from './BlueskyForm'

// Server Action for Bluesky auth
async function startBlueskyAuth(formData: FormData) {
  'use server'

  const handle = formData.get('handle') as string
  if (!handle?.trim()) {
    redirect('/signin?error=Please+enter+your+Bluesky+handle')
  }

  const identityUrl = process.env.NEXT_PUBLIC_IDENTITY_URL || 'http://localhost:8524'
  const websiteUrl = process.env.NEXT_PUBLIC_WEBSITE_URL || 'http://localhost:3000'

  const response = await fetch(`${identityUrl}/auth/atproto/authorize`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      handle: handle.trim(),
      returnUrl: `${websiteUrl}/auth/callback`,
    }),
  }).catch((err) => {
    console.error('[BlueskyAuth] Fetch error:', err)
    return null
  })

  if (!response || !response.ok) {
    const data = response ? await response.json().catch(() => ({})) : {}
    const errorMsg = encodeURIComponent(data.error || 'Failed to connect to identity service')
    redirect(`/signin?error=${errorMsg}`)
  }

  const { authorizationUrl } = await response.json()
  redirect(authorizationUrl)
}

interface SignInPageProps {
  searchParams: Promise<{ error?: string }>
}

export default async function SignInPage({ searchParams }: SignInPageProps) {
  const params = await searchParams
  const error = params.error

  return (
    <div className="min-h-screen bg-background flex items-center justify-center p-4">
      <div className="w-full max-w-lg">
        {/* Logo */}
        <div className="flex items-center justify-center gap-3 mb-8">
          <div className="w-10 h-10 bg-primary rounded-xl flex items-center justify-center">
            <Server className="w-6 h-6 text-primary-foreground" />
          </div>
          <h1 className="text-2xl font-bold text-foreground">Deka</h1>
        </div>

        {/* Sign in card */}
        <div className="bg-card rounded-xl border border-border overflow-hidden">
          <div className="p-8">
            <h2 className="text-xl font-semibold text-foreground mb-6 text-center">
              Sign In with Bluesky
            </h2>
            <BlueskyForm action={startBlueskyAuth} error={error} />
          </div>
        </div>

        <p className="text-center text-muted-foreground text-xs mt-6">
          Sign in to access billing, support, and account management
        </p>
      </div>
    </div>
  )
}
