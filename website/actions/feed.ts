/**
 * Feed Server Actions
 * Server-side functions for fetching activity feed data
 */

'use server'

import { cookies } from 'next/headers'
import { transactionsApi, usersApi } from '@/lib/tana-api'
import type { TanaTransaction, TanaUser } from '@/types/tana-api'
import type { FeedItem } from '@/components/dashboard/FeedCard'

// ============================================================================
// CONSTANTS
// ============================================================================

const SESSION_COOKIE_NAME = 'tana_session_token'

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

async function getSessionToken(): Promise<string | null> {
  const cookieStore = await cookies()
  const tokenCookie = cookieStore.get(SESSION_COOKIE_NAME)
  return tokenCookie?.value || null
}

/**
 * Convert a transaction to a feed item
 */
function transactionToFeedItem(tx: TanaTransaction, currentUserId?: string): FeedItem {
  const isIncoming = currentUserId && tx.to === currentUserId
  const isOutgoing = currentUserId && tx.from === currentUserId

  let title = ''
  let description = ''

  switch (tx.type) {
    case 'transfer':
      if (isIncoming) {
        title = 'Received Transfer'
        description = `You received ${tx.amount} ${tx.currencyCode}`
      } else if (isOutgoing) {
        title = 'Sent Transfer'
        description = `You sent ${tx.amount} ${tx.currencyCode}`
      } else {
        title = 'Transfer'
        description = `Transfer of ${tx.amount} ${tx.currencyCode}`
      }
      break

    case 'deposit':
      title = 'Deposit Request'
      description = tx.status === 'confirmed'
        ? `Deposit of ${tx.amount} ${tx.currencyCode} confirmed`
        : `Pending deposit of ${tx.amount} ${tx.currencyCode}`
      break

    case 'withdraw':
      title = 'Withdrawal Request'
      description = tx.status === 'confirmed'
        ? `Withdrawal of ${tx.amount} ${tx.currencyCode} completed`
        : `Pending withdrawal of ${tx.amount} ${tx.currencyCode}`
      break

    case 'user_creation':
      title = 'New User Created'
      description = 'A new user joined the network'
      break

    case 'contract_deployment':
      title = 'Contract Deployed'
      description = tx.contractId ? `Smart contract ${tx.contractId} deployed` : 'Smart contract deployed'
      break

    case 'contract_call':
      title = 'Contract Executed'
      description = tx.contractId ? `Executed contract ${tx.contractId}` : 'Smart contract executed'
      break

    default:
      title = 'Activity'
      description = `Transaction of type ${tx.type}`
  }

  return {
    id: tx.id,
    type: tx.type === 'transfer' ? 'transaction' : tx.type === 'user_creation' ? 'user_created' : 'transaction',
    timestamp: tx.createdAt,
    title,
    description,
    amount: tx.amount || undefined,
    currencyCode: tx.currencyCode || undefined,
    direction: isIncoming ? 'incoming' : isOutgoing ? 'outgoing' : undefined,
    metadata: tx.metadata || undefined,
  }
}

// ============================================================================
// SERVER ACTIONS
// ============================================================================

/**
 * Get activity feed for a user
 * Returns their recent transactions as feed items
 */
export async function getUserFeed(userId: string, limit = 20, offset = 0): Promise<FeedItem[]> {
  try {
    const token = await getSessionToken()

    // Get user's transactions
    const transactions = await transactionsApi.getAccountTransactions(
      userId,
      limit,
      offset,
      token || undefined
    )

    // Convert to feed items
    const feedItems = transactions.map(tx => transactionToFeedItem(tx, userId))

    // Sort by timestamp (newest first)
    feedItems.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())

    return feedItems
  } catch (error) {
    console.error('Error fetching user feed:', error)
    return []
  }
}

/**
 * Get global activity feed
 * Returns recent network activity
 */
export async function getGlobalFeed(limit = 20, offset = 0): Promise<FeedItem[]> {
  try {
    const token = await getSessionToken()

    // Get all recent transactions
    const transactions = await transactionsApi.getAllTransactions(limit, offset, token || undefined)

    // Convert to feed items
    const feedItems = transactions.map(tx => transactionToFeedItem(tx))

    // Sort by timestamp (newest first)
    feedItems.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())

    return feedItems
  } catch (error) {
    console.error('Error fetching global feed:', error)
    return []
  }
}

/**
 * Get enriched feed with user information
 * Fetches user details for each feed item
 */
export async function getEnrichedFeed(userId: string, limit = 20, offset = 0): Promise<FeedItem[]> {
  try {
    const token = await getSessionToken()

    // Get user's transactions
    const transactions = await transactionsApi.getAccountTransactions(
      userId,
      limit,
      offset,
      token || undefined
    )

    // Convert to feed items
    const feedItems = transactions.map(tx => transactionToFeedItem(tx, userId))

    // Get unique user IDs from transactions
    const userIds = new Set<string>()
    transactions.forEach(tx => {
      if (tx.from && tx.from !== userId) userIds.add(tx.from)
      if (tx.to && tx.to !== userId) userIds.add(tx.to)
    })

    // Fetch user details
    const userDetailsMap = new Map<string, TanaUser>()
    await Promise.all(
      Array.from(userIds).map(async (uid) => {
        try {
          const user = await usersApi.getUser(uid, token || undefined)
          if (user) {
            userDetailsMap.set(uid, user)
          }
        } catch (err) {
          console.error(`Error fetching user ${uid}:`, err)
        }
      })
    )

    // Enrich feed items with user data
    const enrichedItems = feedItems.map((item, index) => {
      const tx = transactions[index]

      // Determine which user to show (the other party in the transaction)
      let relevantUserId: string | undefined
      if (tx.from === userId && tx.to) {
        relevantUserId = tx.to
      } else if (tx.to === userId && tx.from) {
        relevantUserId = tx.from
      }

      if (relevantUserId) {
        const user = userDetailsMap.get(relevantUserId)
        if (user) {
          return {
            ...item,
            user: {
              id: user.id,
              username: user.username,
              displayName: user.displayName,
              avatarData: user.avatarData,
            },
          }
        }
      }

      return item
    })

    // Sort by timestamp (newest first)
    enrichedItems.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())

    return enrichedItems
  } catch (error) {
    console.error('Error fetching enriched feed:', error)
    return []
  }
}
