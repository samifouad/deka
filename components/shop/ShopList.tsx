/**
 * Shop List Component
 * Sidebar showing featured shops
 */

'use client'

import Image from 'next/image'
import { Store, Star, Users } from 'lucide-react'
import type { Shop } from '@/types/shopping'

interface ShopListProps {
  shops: Shop[]
  selectedShopId?: string
  onSelectShop?: (shopId: string | null) => void
}

export function ShopList({ shops, selectedShopId, onSelectShop }: ShopListProps) {
  return (
    <div className="bg-card border border-border rounded-lg p-4 space-y-4 sticky top-6">
      {/* Header */}
      <div className="flex items-center gap-2 pb-3 border-b border-border">
        <Store className="w-5 h-5 text-primary" />
        <h3 className="font-semibold text-foreground">Featured Shops</h3>
      </div>

      {/* All Shops Button */}
      {selectedShopId && onSelectShop && (
        <button
          onClick={() => onSelectShop(null)}
          className="w-full px-3 py-2 text-sm bg-primary hover:bg-primary/80 text-primary-foreground rounded-lg transition-colors"
        >
          View All Shops
        </button>
      )}

      {/* Shop List */}
      <div className="space-y-3">
        {shops.map(shop => (
          <div
            key={shop.id}
            onClick={() => onSelectShop?.(shop.id)}
            className={`group p-3 rounded-lg border transition-all cursor-pointer ${
              selectedShopId === shop.id
                ? 'bg-primary/20 border-primary/50'
                : 'bg-secondary border-border hover:border-primary/50 hover:bg-secondary/80'
            }`}
          >
            {/* Shop Header */}
            <div className="flex items-start gap-3 mb-2">
              <Image
                src={shop.avatar}
                alt={shop.name}
                width={48}
                height={48}
                className="rounded-full ring-2 ring-border group-hover:ring-primary/50 transition-all"
              />
              <div className="flex-1 min-w-0">
                <h4 className="font-medium text-foreground truncate group-hover:text-primary transition-colors">
                  {shop.name}
                </h4>
                <p className="text-xs text-muted-foreground truncate">@{shop.owner}</p>
              </div>
            </div>

            {/* Shop Stats */}
            <div className="flex items-center gap-4 text-xs text-muted-foreground">
              <div className="flex items-center gap-1">
                <Star className="w-3 h-3 fill-primary text-primary" />
                <span>{shop.rating}</span>
              </div>
              <div className="flex items-center gap-1">
                <Users className="w-3 h-3" />
                <span>{shop.followers.toLocaleString()}</span>
              </div>
              <div>
                {shop.productCount} {shop.productCount === 1 ? 'product' : 'products'}
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* View All Link */}
      <div className="pt-3 border-t border-border">
        <button className="w-full text-sm text-primary hover:text-primary/80 transition-colors">
          View All Shops →
        </button>
      </div>
    </div>
  )
}
