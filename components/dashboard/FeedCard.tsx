/**
 * Feed Card Component
 * Displays different types of activity in the dashboard feed
 */

'use client'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Separator } from '@/components/ui/separator'
import {
  ArrowUpRight,
  ArrowDownLeft,
  User as UserIcon,
  FileCode,
  Clock
} from 'lucide-react'
import { formatDistanceToNow } from '@/lib/date-utils'

// ============================================================================
// TYPES
// ============================================================================

export type FeedItemType = 'transaction' | 'user_created' | 'contract_deployed' | 'balance_update'

export interface FeedItem {
  id: string
  type: FeedItemType
  timestamp: string
  title: string
  description: string
  metadata?: Record<string, any>
  user?: {
    id: string
    username: string
    displayName: string
    avatarData?: string | null
  }
  amount?: string
  currencyCode?: string
  direction?: 'incoming' | 'outgoing'
}

// ============================================================================
// COMPONENT
// ============================================================================

interface FeedCardProps {
  item: FeedItem
}

export function FeedCard({ item }: FeedCardProps) {
  const getIcon = () => {
    switch (item.type) {
      case 'transaction':
        return item.direction === 'incoming' ? (
          <ArrowDownLeft className="w-5 h-5 text-primary" />
        ) : (
          <ArrowUpRight className="w-5 h-5 text-primary" />
        )
      case 'user_created':
        return <UserIcon className="w-5 h-5 text-primary" />
      case 'contract_deployed':
        return <FileCode className="w-5 h-5 text-primary" />
      default:
        return <Clock className="w-5 h-5 text-muted-foreground" />
    }
  }

  const getCardBorderColor = () => {
    return 'border-border hover:border-primary/30'
  }

  const getUserInitials = (username: string, displayName: string) => {
    if (displayName) {
      const names = displayName.split(' ')
      if (names.length >= 2) {
        return `${names[0][0]}${names[1][0]}`.toUpperCase()
      }
      return displayName.substring(0, 2).toUpperCase()
    }
    return username.substring(0, 2).toUpperCase().replace('@', '')
  }

  return (
    <Card className={`bg-card ${getCardBorderColor()} transition-colors`}>
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <div className="flex items-start gap-3 flex-1">
            {/* Icon */}
            <div className="mt-1">
              {getIcon()}
            </div>

            {/* Content */}
            <div className="flex-1 min-w-0">
              <CardTitle className="text-foreground text-base mb-1 flex items-center gap-2">
                {item.title}
                {item.amount && item.currencyCode && (
                  <span className="text-sm font-medium text-primary">
                    {item.direction === 'incoming' ? '+' : '-'}{item.amount} {item.currencyCode}
                  </span>
                )}
              </CardTitle>
              <CardDescription className="text-muted-foreground text-sm">
                {item.description}
              </CardDescription>
            </div>
          </div>

          {/* Timestamp */}
          <div className="text-xs text-muted-foreground ml-2 whitespace-nowrap">
            {formatDistanceToNow(new Date(item.timestamp))}
          </div>
        </div>
      </CardHeader>

      {/* User Info (if present) */}
      {item.user && (
        <>
          <Separator className="bg-border" />
          <CardContent className="pt-3 pb-3">
            <div className="flex items-center gap-3">
              <Avatar className="h-8 w-8 border-2 border-primary/30">
                {item.user.avatarData && (
                  <AvatarImage src={item.user.avatarData} alt={item.user.displayName} />
                )}
                <AvatarFallback className="bg-primary text-primary-foreground text-xs">
                  {getUserInitials(item.user.username, item.user.displayName)}
                </AvatarFallback>
              </Avatar>
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-foreground truncate">
                  {item.user.displayName}
                </p>
                <p className="text-xs text-muted-foreground truncate">
                  {item.user.username}
                </p>
              </div>
            </div>
          </CardContent>
        </>
      )}

      {/* Additional Metadata */}
      {item.metadata && Object.keys(item.metadata).length > 0 && (
        <>
          <Separator className="bg-border" />
          <CardContent className="pt-3 pb-3">
            <div className="grid grid-cols-2 gap-2">
              {Object.entries(item.metadata).map(([key, value]) => (
                <div key={key} className="flex flex-col">
                  <span className="text-xs text-muted-foreground capitalize">
                    {key.replace(/_/g, ' ')}
                  </span>
                  <span className="text-sm text-foreground truncate">
                    {String(value)}
                  </span>
                </div>
              ))}
            </div>
          </CardContent>
        </>
      )}
    </Card>
  )
}

// ============================================================================
// SKELETON LOADING STATE
// ============================================================================

export function FeedCardSkeleton() {
  return (
    <Card className="bg-card border-border">
      <CardHeader className="pb-3">
        <div className="flex items-start gap-3">
          <div className="w-5 h-5 bg-muted rounded animate-pulse mt-1" />
          <div className="flex-1 space-y-2">
            <div className="h-4 bg-muted rounded w-3/4 animate-pulse" />
            <div className="h-3 bg-muted rounded w-1/2 animate-pulse" />
          </div>
          <div className="h-3 bg-muted rounded w-16 animate-pulse" />
        </div>
      </CardHeader>
    </Card>
  )
}
