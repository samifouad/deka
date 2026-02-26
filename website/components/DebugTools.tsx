/**
 * Debug Tools Component
 *
 * Loads debug utilities in the browser for development
 * This component only initializes the debug tools, doesn't render anything
 */

'use client'

import { useEffect } from 'react'

export function DebugTools() {
  useEffect(() => {
    // Dynamically import debug tools only in browser
    import('@/lib/debug').catch((err) => {
      console.error('Failed to load debug tools:', err)
    })
  }, [])

  // This component doesn't render anything
  return null
}
