/**
 * Product Card Component
 * Displays a product with image, price, shop info, and add to cart
 */

'use client'

import Image from 'next/image'
import { Star, ShoppingCart, Heart } from 'lucide-react'
import { Button } from '@/components/ui/button'
import type { Product, Shop } from '@/types/shopping'
import { useState } from 'react'

interface ProductCardProps {
  product: Product
  shop?: Shop
  onAddToCart?: (productId: string) => void
}

export function ProductCard({ product, shop, onAddToCart }: ProductCardProps) {
  const [isLiked, setIsLiked] = useState(false)
  const [isAdding, setIsAdding] = useState(false)

  const handleAddToCart = async () => {
    if (!onAddToCart) return
    setIsAdding(true)
    try {
      await onAddToCart(product.id)
    } finally {
      setIsAdding(false)
    }
  }

  return (
    <div className="group bg-card border border-border rounded-lg overflow-hidden hover:border-primary/50 transition-all duration-300 hover:shadow-lg hover:shadow-primary/10">
      {/* Product Image */}
      <div className="relative aspect-square overflow-hidden bg-secondary">
        <Image
          src={product.image}
          alt={product.name}
          fill
          className="object-cover group-hover:scale-105 transition-transform duration-300"
        />

        {/* Like Button */}
        <button
          onClick={() => setIsLiked(!isLiked)}
          className="absolute top-2 right-2 p-2 bg-card/80 backdrop-blur-sm rounded-full hover:bg-card transition-colors"
        >
          <Heart
            className={`w-5 h-5 ${isLiked ? 'fill-destructive text-destructive' : 'text-foreground'}`}
          />
        </button>

        {/* Rating Badge */}
        <div className="absolute top-2 left-2 px-2 py-1 bg-card/80 backdrop-blur-sm rounded-md flex items-center gap-1">
          <Star className="w-4 h-4 fill-primary text-primary" />
          <span className="text-sm font-medium text-foreground">{product.rating}</span>
        </div>
      </div>

      {/* Product Info */}
      <div className="p-4 space-y-3">
        {/* Shop Info */}
        {shop && (
          <div className="flex items-center gap-2">
            <Image
              src={shop.avatar}
              alt={shop.name}
              width={24}
              height={24}
              className="rounded-full"
            />
            <span className="text-sm text-primary hover:text-primary/80 cursor-pointer">
              {shop.name}
            </span>
          </div>
        )}

        {/* Product Name */}
        <h3 className="font-semibold text-foreground line-clamp-2 group-hover:text-primary transition-colors cursor-pointer">
          {product.name}
        </h3>

        {/* Description */}
        <p className="text-sm text-muted-foreground line-clamp-2">
          {product.description}
        </p>

        {/* Price and Stats */}
        <div className="flex items-center justify-between">
          <div>
            <div className="text-2xl font-bold text-foreground">
              ${product.price.toFixed(2)}
            </div>
            <div className="text-xs text-muted-foreground">
              {product.reviews} reviews
            </div>
          </div>

          {/* Add to Cart Button */}
          <Button
            onClick={handleAddToCart}
            disabled={isAdding}
            size="sm"
            className="bg-primary hover:bg-primary/80 text-primary-foreground"
          >
            <ShoppingCart className="w-4 h-4 mr-2" />
            {isAdding ? 'Adding...' : 'Add'}
          </Button>
        </div>

        {/* Category Tag */}
        <div className="pt-2 border-t border-border">
          <span className="inline-block px-2 py-1 text-xs bg-secondary text-secondary-foreground rounded">
            {product.category}
          </span>
        </div>
      </div>
    </div>
  )
}

/**
 * Product Card Skeleton
 * Loading state for product card
 */
export function ProductCardSkeleton() {
  return (
    <div className="bg-card border border-border rounded-lg overflow-hidden">
      {/* Image Skeleton */}
      <div className="aspect-square bg-secondary animate-pulse" />

      {/* Content Skeleton */}
      <div className="p-4 space-y-3">
        <div className="h-4 bg-secondary rounded animate-pulse w-1/3" />
        <div className="h-6 bg-secondary rounded animate-pulse w-4/5" />
        <div className="h-4 bg-secondary rounded animate-pulse w-full" />
        <div className="h-4 bg-secondary rounded animate-pulse w-2/3" />
        <div className="flex items-center justify-between">
          <div className="h-8 bg-secondary rounded animate-pulse w-24" />
          <div className="h-9 bg-secondary rounded animate-pulse w-20" />
        </div>
      </div>
    </div>
  )
}
