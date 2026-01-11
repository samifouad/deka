'use client'

/**
 * Bluesky Form Component
 *
 * Minimal client component - only for useFormStatus to show pending state.
 * The form itself is server-rendered, action is a Server Action.
 */

import { useFormStatus } from 'react-dom'
import { AlertCircle, Loader2 } from 'lucide-react'

interface BlueskyFormProps {
  action: (formData: FormData) => Promise<void>
  error?: string
}

export function BlueskyForm({ action, error }: BlueskyFormProps) {
  return (
    <form action={action} className="space-y-4">
      {/* Handle input */}
      <div>
        <label htmlFor="handle" className="block text-sm font-medium text-foreground mb-2">
          Bluesky Handle
        </label>
        <div className="relative">
          <span className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground">@</span>
          <input
            id="handle"
            name="handle"
            type="text"
            placeholder="username.bsky.social"
            className="w-full pl-8 pr-4 py-3 bg-input border border-border rounded-lg text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary"
            autoComplete="username"
            autoCapitalize="off"
            autoCorrect="off"
            required
          />
        </div>
        <p className="mt-1.5 text-xs text-muted-foreground">
          Enter your handle, e.g., alice.bsky.social
        </p>
      </div>

      {/* Error message */}
      {error && (
        <div className="flex items-start gap-2 p-3 bg-red-500/10 border border-red-500/30 rounded-lg">
          <AlertCircle className="w-4 h-4 text-red-400 flex-shrink-0 mt-0.5" />
          <p className="text-sm text-red-400">{error}</p>
        </div>
      )}

      {/* Submit button with pending state */}
      <SubmitButton />

      {/* Info text */}
      <p className="text-xs text-muted-foreground text-center">
        You&apos;ll be redirected to Bluesky to authorize access
      </p>
    </form>
  )
}

function SubmitButton() {
  const { pending } = useFormStatus()

  return (
    <button
      type="submit"
      disabled={pending}
      className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-primary hover:bg-primary/90 disabled:bg-primary/50 text-primary-foreground font-medium rounded-lg transition-colors disabled:cursor-not-allowed"
    >
      {pending ? (
        <>
          <Loader2 className="w-4 h-4 animate-spin" />
          Connecting to Bluesky...
        </>
      ) : (
        <>
          <BlueskyIcon className="w-4 h-4" />
          Continue with Bluesky
        </>
      )}
    </button>
  )
}

function BlueskyIcon({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 360 320" fill="currentColor" className={className} aria-hidden="true">
      <path d="M180 141.964C163.699 110.262 119.308 51.1817 78.0347 22.044C38.4971 -5.86834 23.414 -1.03207 13.526 3.43594C2.08093 8.60755 0 26.1785 0 36.5164C0 46.8542 5.66748 121.272 9.36416 133.694C21.5786 174.738 65.0603 188.607 105.104 184.156C107.151 183.852 109.227 183.572 111.329 183.312C109.267 183.539 107.19 183.777 105.104 184.03C46.4204 192.038 -5.69621 214.388 62.6582 290.146C130.654 365.519 176.934 259.327 180 250.191C183.066 259.327 229.346 365.519 297.342 290.146C365.696 214.388 313.58 192.038 254.896 184.03C252.81 183.777 250.733 183.539 248.671 183.312C250.773 183.572 252.849 183.852 254.896 184.156C294.94 188.607 338.421 174.738 350.636 133.694C354.333 121.272 360 46.8542 360 36.5164C360 26.1785 357.919 8.60755 346.474 3.43594C336.586 -1.03207 321.503 -5.86834 281.965 22.044C240.692 51.1817 196.301 110.262 180 141.964Z" />
    </svg>
  )
}
