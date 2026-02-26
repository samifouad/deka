/**
 * Dashboard Sidebar
 * Side panel showing user stats and network info
 */

'use client'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Wallet, TrendingUp, Users, Activity } from 'lucide-react'
import type { TanaBalance } from '@/types/tana-api'

interface DashboardSidebarProps {
  balances: TanaBalance[]
}

export function DashboardSidebar({ balances }: DashboardSidebarProps) {
  // Calculate total value (simplified - would need exchange rates in real app)
  const totalBalances = balances.length

  return (
    <div className="hidden lg:block w-80 space-y-4">
      {/* Balances Summary */}
      <Card className="bg-card border-border">
        <CardHeader>
          <CardTitle className="text-foreground flex items-center text-base">
            <Wallet className="w-4 h-4 mr-2 text-primary" />
            Your Balances
          </CardTitle>
        </CardHeader>
        <CardContent>
          {balances.length > 0 ? (
            <div className="space-y-3">
              {balances.slice(0, 3).map((balance) => (
                <div
                  key={`${balance.currencyCode}-${balance.ownerId}`}
                  className="flex items-center justify-between p-3 rounded-lg bg-muted/30 border border-border hover:border-primary/30 transition-colors"
                >
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-foreground truncate">
                      {balance.currencyCode}
                    </p>
                    <p className="text-xs text-muted-foreground">
                      {balance.ownerType === 'user' ? 'Personal' : 'Team'}
                    </p>
                  </div>
                  <div className="text-right ml-2">
                    <p className="text-sm font-bold text-primary">
                      {parseFloat(balance.balance).toLocaleString(undefined, {
                        minimumFractionDigits: 2,
                        maximumFractionDigits: 2,
                      })}
                    </p>
                  </div>
                </div>
              ))}
              {balances.length > 3 && (
                <p className="text-xs text-center text-muted-foreground pt-2">
                  +{balances.length - 3} more
                </p>
              )}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground text-center py-4">
              No balances yet
            </p>
          )}
        </CardContent>
      </Card>

      {/* Stats */}
      <Card className="bg-card border-border">
        <CardHeader>
          <CardTitle className="text-foreground flex items-center text-base">
            <Activity className="w-4 h-4 mr-2 text-primary" />
            Activity Stats
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <TrendingUp className="w-4 h-4 text-primary" />
                <span className="text-sm text-muted-foreground">Transactions</span>
              </div>
              <span className="text-sm font-bold text-foreground">-</span>
            </div>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Users className="w-4 h-4 text-primary" />
                <span className="text-sm text-muted-foreground">Connections</span>
              </div>
              <span className="text-sm font-bold text-foreground">-</span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Network Info */}
      <Card className="bg-card border-border">
        <CardHeader>
          <CardTitle className="text-foreground flex items-center text-base">
            Network Info
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-2 text-xs text-muted-foreground">
            <div className="flex justify-between">
              <span>Status:</span>
              <span className="text-primary font-semibold">Online</span>
            </div>
            <div className="flex justify-between">
              <span>Network:</span>
              <span className="text-foreground">Deka Main</span>
            </div>
            <div className="flex justify-between">
              <span>Your Balances:</span>
              <span className="text-foreground">{totalBalances}</span>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
