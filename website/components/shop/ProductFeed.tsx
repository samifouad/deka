/**
 * Product Feed Component
 * Instagram-style grid of products with infinite scroll
 */

'use client'

import { useEffect, useState, useRef, useCallback } from 'react'
import { ProductCard, ProductCardSkeleton } from './ProductCard'
import { Button } from '@/components/ui/button'
import { RefreshCw, ShoppingBag } from 'lucide-react'
import type { Product, Shop, FilterOptions } from '@/types/shopping'

interface ProductFeedProps {
  initialProducts?: Product[]
  shops: Shop[]
  filters?: FilterOptions
  onAddToCart?: (productId: string) => void
}

// Simulated function to load more products (will be replaced with real API later)
async function loadProducts(
  offset: number,
  limit: number,
  filters?: FilterOptions
): Promise<Product[]> {
  // Import products dynamically
  const productsModule = await import('@/data/products.json')
  const allProducts: Product[] = productsModule.default

  // Apply filters
  let filtered = [...allProducts]

  if (filters?.categories && filters.categories.length > 0) {
    filtered = filtered.filter(p => filters.categories.includes(p.category))
  }

  if (filters?.shops && filters.shops.length > 0) {
    filtered = filtered.filter(p => filters.shops.includes(p.shopId))
  }

  if (filters?.priceRange) {
    filtered = filtered.filter(
      p => p.price >= filters.priceRange.min && p.price <= filters.priceRange.max
    )
  }

  if (filters?.minRating) {
    filtered = filtered.filter(p => p.rating >= filters.minRating!)
  }

  if (filters?.searchQuery) {
    const query = filters.searchQuery.toLowerCase()
    filtered = filtered.filter(
      p =>
        p.name.toLowerCase().includes(query) ||
        p.description.toLowerCase().includes(query) ||
        p.category.toLowerCase().includes(query)
    )
  }

  // Paginate
  return filtered.slice(offset, offset + limit)
}

export function ProductFeed({ initialProducts = [], shops, filters, onAddToCart }: ProductFeedProps) {
  const [products, setProducts] = useState<Product[]>(initialProducts)
  const [isLoading, setIsLoading] = useState(false)
  const [hasMore, setHasMore] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const observerRef = useRef<IntersectionObserver | null>(null)
  const loadMoreRef = useRef<HTMLDivElement>(null)

  // Create a shop lookup map
  const shopMap = new Map(shops.map(shop => [shop.id, shop]))

  // Stable load more function - prevents duplicate products
  const loadMore = useCallback(async () => {
    setIsLoading(true)
    setError(null)

    try {
      // Use functional update to get current products length
      let currentOffset = 0
      setProducts(prev => {
        currentOffset = prev.length
        return prev
      })

      const newProducts = await loadProducts(currentOffset, 12, filters)

      if (newProducts.length === 0) {
        setHasMore(false)
      } else {
        // Deduplicate products by ID before adding
        setProducts(prev => {
          const existingIds = new Set(prev.map(p => p.id))
          const uniqueNewProducts = newProducts.filter(p => !existingIds.has(p.id))
          return [...prev, ...uniqueNewProducts]
        })
      }
    } catch (err) {
      console.error('Error loading products:', err)
      setError('Failed to load products')
    } finally {
      setIsLoading(false)
    }
  }, [filters]) // Only depends on filters

  // Reset and reload when filters change (handles both initial load and filter updates)
  useEffect(() => {
    setProducts([])
    setHasMore(true)
    setError(null)

    // Load products with current filters
    const loadInitial = async () => {
      setIsLoading(true)
      setError(null)

      try {
        const newProducts = await loadProducts(0, 12, filters)

        if (newProducts.length === 0) {
          setHasMore(false)
        } else {
          setProducts(newProducts)
        }
      } catch (err) {
        console.error('Error loading products:', err)
        setError('Failed to load products')
      } finally {
        setIsLoading(false)
      }
    }

    loadInitial()
  }, [filters]) // When filters change, reload

  // Refresh feed
  const refresh = async () => {
    setIsLoading(true)
    setError(null)
    setHasMore(true)

    try {
      const freshProducts = await loadProducts(0, 12, filters)
      setProducts(freshProducts)

      if (freshProducts.length === 0) {
        setHasMore(false)
      }
    } catch (err) {
      console.error('Error refreshing products:', err)
      setError('Failed to refresh products')
    } finally {
      setIsLoading(false)
    }
  }

  // Set up intersection observer for infinite scroll - SAFE VERSION
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
  }, [hasMore, isLoading]) // Safe deps - no loadMore here to prevent loop

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold text-foreground">Discover Products</h2>
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
            className="mt-2 border-destructive/30 text-destructive"
          >
            Try Again
          </Button>
        </div>
      )}

      {/* Product Grid */}
      {products.length > 0 ? (
        <>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
            {products.map(product => (
              <ProductCard
                key={product.id}
                product={product}
                shop={shopMap.get(product.shopId)}
                onAddToCart={onAddToCart}
              />
            ))}
          </div>

          {/* Loading skeletons */}
          {isLoading && (
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
              <ProductCardSkeleton />
              <ProductCardSkeleton />
              <ProductCardSkeleton />
            </div>
          )}

          {/* Load more trigger */}
          {hasMore && !isLoading && (
            <div ref={loadMoreRef} className="h-20 flex items-center justify-center">
              <div className="w-6 h-6 border-2 border-primary border-t-transparent rounded-full animate-spin"></div>
            </div>
          )}

          {/* End of products message */}
          {!hasMore && (
            <div className="text-center py-8">
              <p className="text-sm text-muted-foreground">You've seen all products</p>
            </div>
          )}
        </>
      ) : (
        // Empty state
        !isLoading && (
          <div className="flex flex-col items-center justify-center py-16 px-4">
            <ShoppingBag className="w-16 h-16 text-muted-foreground mb-4" />
            <h3 className="text-lg font-medium text-foreground mb-2">No products found</h3>
            <p className="text-sm text-muted-foreground text-center max-w-sm">
              Try adjusting your filters or search query to find what you're looking for.
            </p>
          </div>
        )
      )}

      {/* Initial loading state */}
      {isLoading && products.length === 0 && (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
          <ProductCardSkeleton />
          <ProductCardSkeleton />
          <ProductCardSkeleton />
          <ProductCardSkeleton />
          <ProductCardSkeleton />
          <ProductCardSkeleton />
        </div>
      )}
    </div>
  )
}
