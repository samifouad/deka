/**
 * Filter Panel Component
 * Filters for products (price, category, rating)
 */

'use client'

import { useState } from 'react'
import { Filter, X, Star } from 'lucide-react'
import { Button } from '@/components/ui/button'
import type { Category, FilterOptions } from '@/types/shopping'

interface FilterPanelProps {
  categories: Category[]
  onFilterChange?: (filters: FilterOptions) => void
}

export function FilterPanel({ categories, onFilterChange }: FilterPanelProps) {
  const [selectedCategories, setSelectedCategories] = useState<string[]>([])
  const [priceRange, setPriceRange] = useState({ min: 0, max: 1000 })
  const [minRating, setMinRating] = useState<number | undefined>(undefined)

  const toggleCategory = (category: string) => {
    const updated = selectedCategories.includes(category)
      ? selectedCategories.filter(c => c !== category)
      : [...selectedCategories, category]

    setSelectedCategories(updated)
    applyFilters({ categories: updated })
  }

  const updatePriceRange = (type: 'min' | 'max', value: string) => {
    const numValue = parseInt(value) || 0
    const updated = type === 'min'
      ? { ...priceRange, min: numValue }
      : { ...priceRange, max: numValue }

    setPriceRange(updated)
    applyFilters({ priceRange: updated })
  }

  const updateRating = (rating: number) => {
    const newRating = minRating === rating ? undefined : rating
    setMinRating(newRating)
    applyFilters({ minRating: newRating })
  }

  const applyFilters = (partialFilters: Partial<FilterOptions>) => {
    if (!onFilterChange) return

    onFilterChange({
      categories: selectedCategories,
      shops: [],
      priceRange,
      minRating,
      ...partialFilters,
    })
  }

  const clearFilters = () => {
    setSelectedCategories([])
    setPriceRange({ min: 0, max: 1000 })
    setMinRating(undefined)

    onFilterChange?.({
      categories: [],
      shops: [],
      priceRange: { min: 0, max: 1000 },
    })
  }

  const hasActiveFilters =
    selectedCategories.length > 0 ||
    priceRange.min > 0 ||
    priceRange.max < 1000 ||
    minRating !== undefined

  return (
    <div className="bg-card border border-border rounded-lg p-4 space-y-6 sticky top-6">
      {/* Header */}
      <div className="flex items-center justify-between pb-3 border-b border-border">
        <div className="flex items-center gap-2">
          <Filter className="w-5 h-5 text-primary" />
          <h3 className="font-semibold text-foreground">Filters</h3>
        </div>
        {hasActiveFilters && (
          <Button
            onClick={clearFilters}
            variant="ghost"
            size="sm"
            className="text-xs text-muted-foreground hover:text-foreground"
          >
            Clear All
          </Button>
        )}
      </div>

      {/* Categories */}
      <div className="space-y-3">
        <h4 className="text-sm font-medium text-foreground">Categories</h4>
        <div className="space-y-2">
          {categories.map(category => (
            <label
              key={category.id}
              className="flex items-center gap-2 cursor-pointer group"
            >
              <input
                type="checkbox"
                checked={selectedCategories.includes(category.name)}
                onChange={() => toggleCategory(category.name)}
                className="w-4 h-4 rounded border-border bg-input text-primary focus:ring-primary focus:ring-offset-background"
              />
              <span className="text-sm text-muted-foreground group-hover:text-foreground transition-colors">
                {category.name}
              </span>
              <span className="text-xs text-muted-foreground ml-auto">
                ({category.productCount})
              </span>
            </label>
          ))}
        </div>
      </div>

      {/* Price Range */}
      <div className="space-y-3">
        <h4 className="text-sm font-medium text-foreground">Price Range</h4>
        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="text-xs text-muted-foreground mb-1 block">Min ($)</label>
            <input
              type="number"
              value={priceRange.min}
              onChange={(e) => updatePriceRange('min', e.target.value)}
              className="w-full px-3 py-2 bg-input border border-border rounded-lg text-sm text-foreground focus:border-primary focus:outline-none"
              placeholder="0"
              min="0"
            />
          </div>
          <div>
            <label className="text-xs text-muted-foreground mb-1 block">Max ($)</label>
            <input
              type="number"
              value={priceRange.max}
              onChange={(e) => updatePriceRange('max', e.target.value)}
              className="w-full px-3 py-2 bg-input border border-border rounded-lg text-sm text-foreground focus:border-primary focus:outline-none"
              placeholder="1000"
              min="0"
            />
          </div>
        </div>
      </div>

      {/* Rating */}
      <div className="space-y-3">
        <h4 className="text-sm font-medium text-foreground">Minimum Rating</h4>
        <div className="flex gap-2">
          {[5, 4, 3, 2, 1].map(rating => (
            <button
              key={rating}
              onClick={() => updateRating(rating)}
              className={`flex items-center gap-1 px-3 py-2 rounded-lg text-sm transition-all ${
                minRating === rating
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-secondary text-secondary-foreground hover:bg-secondary/80'
              }`}
            >
              <Star className={`w-4 h-4 ${minRating === rating ? 'fill-primary-foreground' : 'fill-primary text-primary'}`} />
              <span>{rating}+</span>
            </button>
          ))}
        </div>
      </div>

      {/* Active Filters Summary */}
      {hasActiveFilters && (
        <div className="pt-3 border-t border-border">
          <div className="flex flex-wrap gap-2">
            {selectedCategories.map(cat => (
              <span
                key={cat}
                className="inline-flex items-center gap-1 px-2 py-1 bg-primary/30 text-primary text-xs rounded-md"
              >
                {cat}
                <button
                  onClick={() => toggleCategory(cat)}
                  className="hover:text-primary/80"
                >
                  <X className="w-3 h-3" />
                </button>
              </span>
            ))}
            {(priceRange.min > 0 || priceRange.max < 1000) && (
              <span className="inline-flex items-center gap-1 px-2 py-1 bg-primary/30 text-primary text-xs rounded-md">
                ${priceRange.min} - ${priceRange.max}
              </span>
            )}
            {minRating && (
              <span className="inline-flex items-center gap-1 px-2 py-1 bg-primary/30 text-primary text-xs rounded-md">
                {minRating}+ stars
              </span>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
