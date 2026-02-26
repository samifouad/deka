/**
 * Dashboard Page - Shopping Portal
 * Product discovery and shopping experience
 */

'use client'

import { useEffect, useState, useRef } from 'react'
import { useRouter } from 'next/navigation'
import { useUser } from '@/context/user-context'
import { Loader2 } from 'lucide-react'
import { DashboardTopBar } from '@/components/dashboard/DashboardTopBar'
import { ProductFeed } from '@/components/shop/ProductFeed'
import { FilterPanel } from '@/components/shop/FilterPanel'
import { ShopList } from '@/components/shop/ShopList'
import type { Shop, Category, FilterOptions } from '@/types/shopping'

// Import shop and category data
import shopsData from '@/data/shops.json'
import categoriesData from '@/data/categories.json'

// Initial filters - defined outside component to ensure stability
const INITIAL_FILTERS: FilterOptions = {
  categories: [],
  shops: [],
  priceRange: { min: 0, max: 1000 },
}

export default function DashboardPage() {
  const router = useRouter()
  const { user, balances, isLoading, error, logout } = useUser()

  // Shopping state
  const [filters, setFilters] = useState<FilterOptions>(INITIAL_FILTERS)
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedShopId, setSelectedShopId] = useState<string | null>(null)
  const [cartItems, setCartItems] = useState<string[]>([]) // Simple cart with product IDs

  // Cast imported data to proper types
  const shops = shopsData as Shop[]
  const categories = categoriesData as Category[]

  // Prevent redirect loops - track if we've already redirected
  const hasRedirectedRef = useRef(false)

  useEffect(() => {
    console.log('[DASHBOARD] Auth state:', {
      user: user?.username,
      isLoading,
      error,
      hasRedirected: hasRedirectedRef.current
    })

    // Don't redirect if we've already redirected once
    if (hasRedirectedRef.current) {
      console.log('[DASHBOARD] Already redirected, skipping to prevent loop')
      return
    }

    // Only redirect if we're NOT loading and user is NOT authenticated
    if (!isLoading && !user) {
      console.log('[DASHBOARD] User not authenticated, redirecting to signin...')
      hasRedirectedRef.current = true
      router.push('/signin')
    }
  }, [user, isLoading, router, error])

  const handleLogout = async () => {
    // Clear session cookie
    await fetch('/api/auth/session', { method: 'DELETE' })
    // Clear context
    logout()
    // Redirect to home
    router.push('/')
  }

  const handleSearch = (query: string) => {
    setSearchQuery(query)
    setFilters(prev => ({ ...prev, searchQuery: query }))
  }

  const handleFilterChange = (newFilters: FilterOptions) => {
    setFilters(newFilters)
  }

  const handleShopSelect = (shopId: string | null) => {
    setSelectedShopId(shopId)
    if (shopId) {
      setFilters(prev => ({ ...prev, shops: [shopId] }))
    } else {
      setFilters(prev => ({ ...prev, shops: [] }))
    }
  }

  const handleAddToCart = (productId: string) => {
    setCartItems(prev => {
      if (prev.includes(productId)) {
        return prev // Already in cart
      }
      return [...prev, productId]
    })
    // Show a toast or notification here (can be added later)
  }

  // Loading state
  if (isLoading) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="flex items-center gap-3 text-primary">
          <Loader2 className="w-6 h-6 animate-spin" />
          <span>Loading your dashboard...</span>
        </div>
      </div>
    )
  }

  // Not authenticated
  if (!user) {
    return null // Will redirect via useEffect
  }

  // Error state
  if (error) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center p-8">
        <div className="bg-card border border-destructive/20 rounded-lg p-6 max-w-md">
          <h2 className="text-xl font-bold text-destructive mb-2">Error Loading Dashboard</h2>
          <p className="text-destructive/70 mb-4">{error}</p>
          <button
            onClick={handleLogout}
            className="w-full px-4 py-2 bg-secondary hover:bg-secondary/80 text-secondary-foreground rounded-lg transition-colors"
          >
            Logout
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-background text-foreground">
      {/* Top Navigation Bar */}
      <DashboardTopBar
        user={user}
        onLogout={handleLogout}
        cartItemCount={cartItems.length}
        onSearch={handleSearch}
      />

      {/* Main Content Area - Shopping Portal Layout */}
      <div className="max-w-[1600px] mx-auto px-4 sm:px-6 lg:px-8 py-6">
        <div className="flex gap-6">
          {/* Left Sidebar - Filters (Desktop) */}
          <aside className="hidden lg:block w-64 flex-shrink-0">
            <FilterPanel
              categories={categories}
              onFilterChange={handleFilterChange}
            />
          </aside>

          {/* Main Feed - Products */}
          <main className="flex-1 min-w-0">
            <ProductFeed
              shops={shops}
              filters={filters}
              onAddToCart={handleAddToCart}
            />
          </main>

          {/* Right Sidebar - Shops (Desktop) */}
          <aside className="hidden xl:block w-80 flex-shrink-0">
            <ShopList
              shops={shops}
              selectedShopId={selectedShopId ?? undefined}
              onSelectShop={handleShopSelect}
            />
          </aside>
        </div>
      </div>
    </div>
  )
}
