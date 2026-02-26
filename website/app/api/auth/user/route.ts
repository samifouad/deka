/**
 * User Info API Route
 *
 * Fetches user data from deka-identity given a session token
 */

import { NextRequest, NextResponse } from 'next/server'

export async function GET(request: NextRequest) {
  try {
    const authHeader = request.headers.get('authorization')
    const sessionToken = authHeader?.replace('Bearer ', '')

    console.log('[User API] Session token:', sessionToken?.substring(0, 20) + '...')

    if (!sessionToken) {
      console.error('[User API] No session token provided')
      return NextResponse.json(
        { error: 'Session token required' },
        { status: 401 }
      )
    }

    // Fetch user data from deka-identity
    const identityUrl = process.env.NEXT_PUBLIC_IDENTITY_URL || 'http://localhost:8524'
    const url = `${identityUrl}/auth/session/${sessionToken}`

    console.log('[User API] Fetching from:', url)

    const response = await fetch(url)

    console.log('[User API] Response status:', response.status)

    if (!response.ok) {
      const errorText = await response.text()
      console.error('[User API] Identity service error:', errorText)
      return NextResponse.json(
        { error: 'Invalid session token', details: errorText },
        { status: 401 }
      )
    }

    const data = await response.json()
    console.log('[User API] Session data:', JSON.stringify(data, null, 2))

    const userData = {
      userId: data.user_id || data.userId,
      username: data.username,
      address: data.address,
      did: data.user_id,
    }

    console.log('[User API] Returning user data:', userData)

    return NextResponse.json(userData)
  } catch (error) {
    console.error('[User API] Error fetching user data:', error)
    return NextResponse.json(
      { error: 'Failed to fetch user data', details: String(error) },
      { status: 500 }
    )
  }
}
