/**
 * Dashboard Page
 * Clean, user-friendly dashboard showing user info and balances
 */

'use client'

import { useEffect } from 'react'
import { useRouter } from 'next/navigation'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { useUser } from '@/context/user-context'
import { LogOut, Wallet, User, Loader2, RefreshCw } from 'lucide-react'
import MenuBar from '@/components/menu/Bar'

export default function DashboardPage() {
  const router = useRouter()
  const { user, balances, isLoading, error, refreshUser, logout } = useUser()

  useEffect(() => {
    // Redirect to signin if not authenticated
    if (!isLoading && !user) {
      router.push('/signin')
    }
  }, [user, isLoading, router])

  const handleLogout = async () => {
    // Clear session cookie
    await fetch('/api/auth/session', { method: 'DELETE' })
    // Clear context
    logout()
    // Redirect to home
    router.push('/')
  }

  // Loading state
  if (isLoading) {
    return (
      <div className="min-h-screen bg-slate-950 flex items-center justify-center">
        <div className="flex items-center gap-3 text-blue-200">
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
      <div className="min-h-screen bg-slate-950 flex items-center justify-center p-8">
        <Card className="bg-slate-900/50 border-red-400/20 max-w-md">
          <CardHeader>
            <CardTitle className="text-red-200">Error Loading Dashboard</CardTitle>
            <CardDescription className="text-red-300/70">{error}</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex gap-3">
              <Button onClick={refreshUser} variant="outline" className="flex-1">
                <RefreshCw className="w-4 h-4 mr-2" />
                Retry
              </Button>
              <Button onClick={handleLogout} variant="outline" className="flex-1">
                Logout
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-slate-950 text-white">
      {/* Menu Bar */}
      <div className="h-[70px] border-b border-slate-800/50 bg-slate-900/30 backdrop-blur-sm sticky top-0 z-50">
        <MenuBar user={user.username.replace('@', '')} />
      </div>

      {/* Main Content */}
      <div className="max-w-7xl mx-auto p-4 sm:p-6 lg:p-8">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center justify-between mb-2">
            <div>
              <h1 className="text-2xl sm:text-3xl font-bold text-white">
                Welcome back, {user.displayName}
              </h1>
              <p className="text-blue-200/60 mt-1">
                {user.username}
              </p>
            </div>
            <Button
              onClick={handleLogout}
              variant="outline"
              size="sm"
              className="border-red-400/30 text-red-200 hover:bg-red-900/20"
            >
              <LogOut className="w-4 h-4 mr-2" />
              Logout
            </Button>
          </div>
        </div>

        {/* Grid Layout */}
        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
          {/* User Profile Card */}
          <Card className="bg-slate-900/50 border-blue-400/20">
            <CardHeader>
              <CardTitle className="text-blue-200 flex items-center text-lg">
                <User className="w-5 h-5 mr-2" />
                Profile
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-3 text-sm">
                <div className="flex justify-between items-center py-2 border-b border-blue-400/10">
                  <span className="text-blue-200/60">Display Name:</span>
                  <span className="text-white font-medium">{user.displayName}</span>
                </div>
                <div className="flex justify-between items-center py-2 border-b border-blue-400/10">
                  <span className="text-blue-200/60">Username:</span>
                  <span className="text-white font-medium">{user.username}</span>
                </div>
                <div className="flex justify-between items-center py-2 border-b border-blue-400/10">
                  <span className="text-blue-200/60">Role:</span>
                  <span className="text-white font-medium capitalize">{user.role}</span>
                </div>
                {user.bio && (
                  <div className="pt-2">
                    <p className="text-blue-200/60 text-xs mb-1">Bio:</p>
                    <p className="text-white text-sm">{user.bio}</p>
                  </div>
                )}
              </div>
            </CardContent>
          </Card>

          {/* Balances Cards */}
          {balances.length > 0 ? (
            balances.map((balance) => (
              <Card
                key={`${balance.currencyCode}-${balance.ownerId}`}
                className="bg-slate-900/50 border-green-400/20"
              >
                <CardHeader>
                  <CardTitle className="text-green-200 flex items-center text-lg">
                    <Wallet className="w-5 h-5 mr-2" />
                    {balance.currencyCode}
                  </CardTitle>
                  <CardDescription className="text-green-300/70">
                    {balance.ownerType === 'user' ? 'Personal Balance' : 'Team Balance'}
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="text-3xl font-bold text-white mb-4">
                    {parseFloat(balance.balance).toLocaleString(undefined, {
                      minimumFractionDigits: 2,
                      maximumFractionDigits: 8,
                    })}
                  </div>
                  <div className="text-xs text-green-200/60">
                    Last updated: {new Date(balance.updatedAt).toLocaleDateString()}
                  </div>
                </CardContent>
              </Card>
            ))
          ) : (
            <Card className="bg-slate-900/50 border-slate-400/20 md:col-span-2">
              <CardHeader>
                <CardTitle className="text-slate-200 flex items-center text-lg">
                  <Wallet className="w-5 h-5 mr-2" />
                  Balances
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-slate-400 text-sm">
                  You don't have any balances yet. Start by receiving your first transaction.
                </p>
              </CardContent>
            </Card>
          )}
        </div>

        {/* Info Section */}
        <div className="mt-8">
          <Card className="bg-slate-900/30 border-blue-400/10">
            <CardContent className="pt-6">
              <p className="text-blue-200/50 text-sm text-center">
                Your account is secured with Ed25519 cryptographic signatures.
                Your private keys are stored safely on your mobile device.
              </p>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}
