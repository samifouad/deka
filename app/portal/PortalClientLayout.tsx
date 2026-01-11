'use client'

/**
 * Portal Client Layout
 *
 * Client-side UI components for the portal (sidebar, navigation, etc)
 */

import { useState } from 'react'
import { useRouter, usePathname } from 'next/navigation'
import Link from 'next/link'
import type { DekaUser } from '@/types/auth'
import {
  CreditCard,
  LifeBuoy,
  Database,
  LogOut,
  Menu,
  X,
  Server
} from 'lucide-react'

const navigation = [
  { name: 'Billing', href: '/portal/billing', icon: CreditCard },
  { name: 'Support', href: '/portal/support', icon: LifeBuoy },
  { name: 'Backups', href: '/portal/backups', icon: Database },
]

interface PortalClientLayoutProps {
  user: DekaUser
  children: React.ReactNode
}

export function PortalClientLayout({ user, children }: PortalClientLayoutProps) {
  const router = useRouter()
  const pathname = usePathname()
  const [isSidebarOpen, setIsSidebarOpen] = useState(false)

  const handleSignOut = async () => {
    // Clear localStorage
    localStorage.clear()

    // Clear session cookie
    await fetch('/api/auth/session', { method: 'DELETE' })

    // Redirect to home
    router.push('/')
  }

  return (
    <div className="min-h-screen bg-background">
      {/* Mobile sidebar toggle */}
      <div className="lg:hidden fixed top-0 left-0 right-0 z-50 bg-card border-b border-border px-4 py-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 bg-primary rounded-lg flex items-center justify-center">
              <Server className="w-5 h-5 text-primary-foreground" />
            </div>
            <span className="text-lg font-bold text-foreground">Deka Portal</span>
          </div>
          <button
            onClick={() => setIsSidebarOpen(!isSidebarOpen)}
            className="p-2 text-muted-foreground hover:text-foreground"
          >
            {isSidebarOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
          </button>
        </div>
      </div>

      <div className="lg:flex">
        {/* Sidebar */}
        <aside
          className={`
            ${isSidebarOpen ? 'block' : 'hidden'}
            lg:block
            fixed lg:sticky
            top-16 lg:top-0
            left-0
            w-64
            h-[calc(100vh-4rem)] lg:h-screen
            bg-card
            border-r border-border
            overflow-y-auto
            z-40
          `}
        >
          <div className="flex flex-col h-full">
            {/* Logo - Desktop only */}
            <div className="hidden lg:flex items-center gap-3 p-6 border-b border-border">
              <div className="w-10 h-10 bg-primary rounded-xl flex items-center justify-center">
                <Server className="w-6 h-6 text-primary-foreground" />
              </div>
              <div>
                <h1 className="text-lg font-bold text-foreground">Deka</h1>
                <p className="text-xs text-muted-foreground">User Portal</p>
              </div>
            </div>

            {/* Navigation */}
            <nav className="flex-1 p-4 space-y-1">
              {navigation.map((item) => {
                const isActive = pathname === item.href
                const Icon = item.icon
                return (
                  <Link
                    key={item.name}
                    href={item.href}
                    onClick={() => setIsSidebarOpen(false)}
                    className={`
                      flex items-center gap-3 px-4 py-3 rounded-lg transition-colors
                      ${
                        isActive
                          ? 'bg-primary/20 text-foreground font-semibold'
                          : 'text-muted-foreground hover:text-foreground hover:bg-secondary'
                      }
                    `}
                  >
                    <Icon className="w-5 h-5" />
                    <span className="font-medium">{item.name}</span>
                  </Link>
                )
              })}
            </nav>

            {/* User info & Sign Out */}
            <div className="p-4 border-t border-border">
              <div className="mb-3 px-4 py-2">
                <p className="text-xs text-muted-foreground">Signed in as</p>
                <p className="text-sm text-foreground font-medium truncate">
                  {user.username || user.address}
                </p>
              </div>
              <button
                onClick={handleSignOut}
                className="flex items-center gap-3 px-4 py-3 rounded-lg text-muted-foreground hover:text-foreground hover:bg-secondary transition-colors w-full"
              >
                <LogOut className="w-5 h-5" />
                <span className="font-medium">Sign Out</span>
              </button>
            </div>
          </div>
        </aside>

        {/* Main content */}
        <main className="flex-1 mt-16 lg:mt-0">
          <div className="max-w-7xl mx-auto p-6 lg:p-8">
            {children}
          </div>
        </main>
      </div>

      {/* Mobile sidebar backdrop */}
      {isSidebarOpen && (
        <div
          className="lg:hidden fixed inset-0 bg-black/50 z-30 top-16"
          onClick={() => setIsSidebarOpen(false)}
        />
      )}
    </div>
  )
}
