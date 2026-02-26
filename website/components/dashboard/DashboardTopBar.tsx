/**
 * Dashboard Top Bar
 * Navigation bar at the top of the dashboard
 */

'use client'

import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { Button } from '@/components/ui/button'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import {
  Home,
  Bell,
  MessageSquare,
  LogOut,
  Menu,
  Search,
  ShoppingCart,
  Sun,
  Moon
} from 'lucide-react'
import { useTheme } from '@/context/theme-context'
import { Sheet, SheetContent, SheetTrigger } from '@/components/ui/sheet'
import type { TanaUser } from '@/types/tana-api'

interface DashboardTopBarProps {
  user: TanaUser
  onLogout: () => void
  cartItemCount?: number
  onSearch?: (query: string) => void
}

export function DashboardTopBar({ user, onLogout, cartItemCount = 0, onSearch }: DashboardTopBarProps) {
  const router = useRouter()
  const { theme, toggleTheme } = useTheme()

  const getUserInitials = () => {
    if (user.displayName) {
      const names = user.displayName.split(' ')
      if (names.length >= 2) {
        return `${names[0][0]}${names[1][0]}`.toUpperCase()
      }
      return user.displayName.substring(0, 2).toUpperCase()
    }
    return user.username.substring(0, 2).toUpperCase().replace('@', '')
  }

  return (
    <div className="sticky top-0 z-50 bg-card/95 backdrop-blur-sm border-b border-border">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16">
          {/* Left: Logo & Brand */}
          <div className="flex items-center gap-4">
            {/* Mobile Menu */}
            <Sheet>
              <SheetTrigger asChild className="lg:hidden">
                <Button variant="ghost" size="sm" className="text-primary">
                  <Menu className="w-5 h-5" />
                </Button>
              </SheetTrigger>
              <SheetContent side="left" className="bg-card border-border">
                <div className="flex flex-col gap-4 mt-8 h-full">
                  <Link
                    href="/dashboard"
                    className="flex items-center gap-3 px-4 py-2 rounded-lg hover:bg-accent text-primary transition-colors"
                  >
                    <Home className="w-5 h-5" />
                    <span>Home</span>
                  </Link>
                  <Link
                    href="/messages"
                    className="flex items-center gap-3 px-4 py-2 rounded-lg hover:bg-accent text-primary transition-colors"
                  >
                    <MessageSquare className="w-5 h-5" />
                    <span>Messages</span>
                  </Link>
                  <Link
                    href="/notifications"
                    className="flex items-center gap-3 px-4 py-2 rounded-lg hover:bg-accent text-primary transition-colors"
                  >
                    <Bell className="w-5 h-5" />
                    <span>Notifications</span>
                  </Link>
                  <div className="mt-auto space-y-2 pt-4 border-t border-border">
                    <button
                      onClick={toggleTheme}
                      className="w-full flex items-center gap-3 px-4 py-2 rounded-lg hover:bg-accent text-foreground transition-colors"
                    >
                      {theme === 'dark' ? (
                        <>
                          <Sun className="w-5 h-5" />
                          <span>Light Mode</span>
                        </>
                      ) : (
                        <>
                          <Moon className="w-5 h-5" />
                          <span>Dark Mode</span>
                        </>
                      )}
                    </button>
                    <button
                      onClick={onLogout}
                      className="w-full flex items-center gap-3 px-4 py-2 rounded-lg hover:bg-destructive/20 text-destructive transition-colors"
                    >
                      <LogOut className="w-5 h-5" />
                      <span>Logout</span>
                    </button>
                  </div>
                </div>
              </SheetContent>
            </Sheet>

            {/* Logo */}
            <Link href="/dashboard" className="flex items-center gap-2 group">
              <div className="w-8 h-8 bg-primary border border-primary/30 flex items-center justify-center transform rotate-45 group-hover:bg-primary/80 transition-colors">
                <div className="w-3 h-3 bg-background transform -rotate-45"></div>
              </div>
              <span className="hidden sm:block text-xl font-bold text-primary group-hover:text-primary/80 transition-colors">deka</span>
            </Link>
          </div>

          {/* Center: Search (Desktop) */}
          <div className="hidden md:flex flex-1 max-w-md mx-8">
            <div className="relative w-full">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
              <input
                type="text"
                placeholder="Search products..."
                onChange={(e) => onSearch?.(e.target.value)}
                className="w-full pl-10 pr-4 py-2 bg-input border border-border rounded-lg text-foreground placeholder-muted-foreground focus:outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/50"
              />
            </div>
          </div>

          {/* Right: Nav Icons & Profile */}
          <div className="flex items-center gap-2">
            {/* Desktop Nav Icons */}
            <div className="hidden lg:flex items-center gap-1">
              <Button
                variant="ghost"
                size="sm"
                className="text-primary hover:bg-accent"
                onClick={() => router.push('/dashboard')}
              >
                <Home className="w-5 h-5" />
              </Button>

              {/* Shopping Cart */}
              <Button
                variant="ghost"
                size="sm"
                className="text-primary hover:bg-accent relative"
                onClick={() => router.push('/cart')}
              >
                <ShoppingCart className="w-5 h-5" />
                {cartItemCount > 0 && (
                  <span className="absolute -top-1 -right-1 w-5 h-5 bg-primary text-primary-foreground text-xs font-bold rounded-full flex items-center justify-center">
                    {cartItemCount > 9 ? '9+' : cartItemCount}
                  </span>
                )}
              </Button>

              <Button
                variant="ghost"
                size="sm"
                className="text-primary hover:bg-accent relative"
                onClick={() => router.push('/messages')}
              >
                <MessageSquare className="w-5 h-5" />
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="text-primary hover:bg-accent relative"
                onClick={() => router.push('/notifications')}
              >
                <Bell className="w-5 h-5" />
                <span className="absolute top-1 right-1 w-2 h-2 bg-destructive rounded-full"></span>
              </Button>
            </div>

            {/* Profile Avatar */}
            <Sheet>
              <SheetTrigger asChild>
                <Button variant="ghost" size="sm" className="p-1 h-auto">
                  <Avatar className="h-8 w-8 border-2 border-primary/30">
                    {user.avatarData && (
                      <AvatarImage src={user.avatarData} alt={user.displayName} />
                    )}
                    <AvatarFallback className="bg-primary text-primary-foreground text-xs">
                      {getUserInitials()}
                    </AvatarFallback>
                  </Avatar>
                </Button>
              </SheetTrigger>
              <SheetContent className="bg-card border-border">
                <div className="flex flex-col gap-4 mt-8">
                  <div className="flex items-center gap-3 pb-4 border-b border-border">
                    <Avatar className="h-12 w-12 border-2 border-primary/30">
                      {user.avatarData && (
                        <AvatarImage src={user.avatarData} alt={user.displayName} />
                      )}
                      <AvatarFallback className="bg-primary text-primary-foreground">
                        {getUserInitials()}
                      </AvatarFallback>
                    </Avatar>
                    <div className="flex-1 min-w-0">
                      <p className="font-semibold text-foreground truncate">{user.displayName}</p>
                      <p className="text-sm text-muted-foreground truncate">{user.username}</p>
                      <p className="text-xs text-primary capitalize">{user.role}</p>
                    </div>
                  </div>

                  {user.bio && (
                    <div className="pb-4 border-b border-border">
                      <p className="text-sm text-muted-foreground">{user.bio}</p>
                    </div>
                  )}

                  <div className="mt-auto pt-4 space-y-2 border-t border-border">
                    <button
                      onClick={toggleTheme}
                      className="w-full flex items-center gap-3 px-4 py-2 rounded-lg hover:bg-accent text-foreground transition-colors"
                    >
                      {theme === 'dark' ? (
                        <>
                          <Sun className="w-5 h-5" />
                          <span>Light Mode</span>
                        </>
                      ) : (
                        <>
                          <Moon className="w-5 h-5" />
                          <span>Dark Mode</span>
                        </>
                      )}
                    </button>
                    <button
                      onClick={onLogout}
                      className="w-full flex items-center gap-3 px-4 py-2 rounded-lg hover:bg-destructive/20 text-destructive transition-colors"
                    >
                      <LogOut className="w-5 h-5" />
                      <span>Logout</span>
                    </button>
                  </div>
                </div>
              </SheetContent>
            </Sheet>
          </div>
        </div>
      </div>
    </div>
  )
}
