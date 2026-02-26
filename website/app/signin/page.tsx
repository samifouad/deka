/**
 * Deka Sign In Page
 *
 * Server-rendered sign-in with Bluesky OAuth only.
 */

import { redirect } from 'next/navigation'
import { BlueskyForm } from './BlueskyForm'
import styles from './signin.module.css'

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
    <div className={styles.page}>
      <div className={styles.shell}>
        {/* Logo */}
        <div className={styles.logoRow}>
          <div className={styles.logoMark}>
            <img
              src="/android-chrome-512x512.png"
              alt="deka"
              className="h-9 w-9 rounded-full"
            />
          </div>
          <h1 className={styles.logoTitle}>deka</h1>
        </div>

        {/* Sign in card */}
        <div className={styles.card}>
          <h2 className={styles.cardTitle}>Sign In with Bluesky</h2>
          <BlueskyForm action={startBlueskyAuth} error={error} />
        </div>

        <p className={styles.footerText}>Sign in to access billing, support, and account management</p>
      </div>
    </div>
  )
}
