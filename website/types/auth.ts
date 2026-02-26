/**
 * Authentication types for Deka authentication system
 */

export interface SessionTokenData {
  token: string
  userId: string
  username: string
  address: string
  expiresAt: string
}

export interface DekaUser {
  address: string
  username: string | null
  role: 'sovereign' | 'admin' | 'member'
  avatar_url?: string | null
}
