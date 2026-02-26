/**
 * Activity Feed Component
 * Displays an infinite-scrolling feed of activity items
 */

'use client'

import { useEffect, useState, useRef, useCallback } from 'react'
import { FeedCard, FeedCardSkeleton, type FeedItem } from './FeedCard'
import { Button } from '@/components/ui/button'
import { RefreshCw, Inbox } from 'lucide-react'
import { getEnrichedFeed } from '@/actions/feed'

interface ActivityFeedProps {
  userId: string
  initialItems?: FeedItem[]
}

export function ActivityFeed({ userId, initialItems = [] }: ActivityFeedProps) {
  const [items, setItems] = useState<FeedItem[]>(initialItems)
  const [isLoading, setIsLoading] = useState(false)
  const [hasMore, setHasMore] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [isInitialLoad, setIsInitialLoad] = useState(initialItems.length === 0)
  const observerRef = useRef<IntersectionObserver | null>(null)
  const loadMoreRef = useRef<HTMLDivElement>(null)

  // Load more items - stable function that doesn't depend on items.length
  const loadMore = useCallback(async () => {
    setIsLoading(true)
    setError(null)

    try {
      // Use functional update to get current items length
      let currentOffset = 0
      setItems((prev) => {
        currentOffset = prev.length
        return prev
      })

      const newItems = await getEnrichedFeed(userId, 20, currentOffset)

      if (newItems.length === 0) {
        setHasMore(false)
      } else {
        setItems((prev) => [...prev, ...newItems])
      }
    } catch (err) {
      console.error('Error loading more items:', err)
      setError('Failed to load more items')
    } finally {
      setIsLoading(false)
      setIsInitialLoad(false)
    }
  }, [userId]) // Only depends on userId - prevents infinite loop

  // Initial load
  useEffect(() => {
    if (isInitialLoad) {
      loadMore()
    }
  }, [isInitialLoad, loadMore])

  // Refresh feed
  const refresh = async () => {
    setIsLoading(true)
    setError(null)
    setHasMore(true)

    try {
      const freshItems = await getEnrichedFeed(userId, 20, 0)
      setItems(freshItems)

      if (freshItems.length === 0) {
        setHasMore(false)
      }
    } catch (err) {
      console.error('Error refreshing feed:', err)
      setError('Failed to refresh feed')
    } finally {
      setIsLoading(false)
    }
  }

  // Set up intersection observer for infinite scroll
  useEffect(() => {
    if (!loadMoreRef.current || !hasMore || isLoading) return

    if (observerRef.current) {
      observerRef.current.disconnect()
    }

    observerRef.current = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting) {
          loadMore()
        }
      },
      { threshold: 0.1 }
    )

    observerRef.current.observe(loadMoreRef.current)

    return () => {
      if (observerRef.current) {
        observerRef.current.disconnect()
      }
    }
  }, [hasMore, isLoading]) // CRITICAL FIX: Removed loadMore from deps to prevent infinite loop

  return (
    <div className="space-y-4">
      {/* Header with refresh button */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-bold text-foreground">Activity Feed</h2>
        <Button
          onClick={refresh}
          variant="ghost"
          size="sm"
          disabled={isLoading}
          className="text-primary hover:bg-accent"
        >
          <RefreshCw className={`w-4 h-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {/* Error message */}
      {error && (
        <div className="p-4 bg-destructive/20 border border-destructive/20 rounded-lg">
          <p className="text-sm text-destructive">{error}</p>
          <Button
            onClick={refresh}
            variant="outline"
            size="sm"
            className="mt-2 border-destructive/30 text-destructive hover:bg-destructive/10"
          >
            Try Again
          </Button>
        </div>
      )}

      {/* Feed items */}
      {items.length > 0 ? (
        <>
          <div className="space-y-4">
            {items.map((item) => (
              <FeedCard key={item.id} item={item} />
            ))}
          </div>

          {/* Loading indicator */}
          {isLoading && (
            <div className="space-y-4">
              <FeedCardSkeleton />
              <FeedCardSkeleton />
            </div>
          )}

          {/* Load more trigger */}
          {hasMore && !isLoading && (
            <div ref={loadMoreRef} className="h-20 flex items-center justify-center">
              <div className="w-6 h-6 border-2 border-primary border-t-transparent rounded-full animate-spin"></div>
            </div>
          )}

          {/* End of feed message */}
          {!hasMore && (
            <div className="text-center py-8">
              <p className="text-sm text-muted-foreground">You've reached the end of your feed</p>
            </div>
          )}
        </>
      ) : (
        // Empty state
        !isLoading && (
          <div className="flex flex-col items-center justify-center py-16 px-4">
            <Inbox className="w-16 h-16 text-muted-foreground mb-4" />
            <h3 className="text-lg font-medium text-foreground mb-2">No activity yet</h3>
            <p className="text-sm text-muted-foreground text-center max-w-sm">
              Your activity feed will show transactions, deposits, withdrawals, and other network activity.
            </p>
          </div>
        )
      )}

      {/* Initial loading state */}
      {isLoading && items.length === 0 && (
        <div className="space-y-4">
          <FeedCardSkeleton />
          <FeedCardSkeleton />
          <FeedCardSkeleton />
          <FeedCardSkeleton />
        </div>
      )}
    </div>
  )
}
