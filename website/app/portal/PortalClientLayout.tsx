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
import styles from './portal.module.css'

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
    <div className={styles.page}>
      {/* Mobile sidebar toggle */}
      <div className={`${styles.mobileHeader} lg:hidden`}>
        <div className="flex items-center justify-between">
          <div className={styles.brandRow}>
            <div className={styles.brandMark}>
              <Server className="h-4 w-4" />
            </div>
            <span className={styles.brandTitle}>Deka Portal</span>
          </div>
          <button
            onClick={() => setIsSidebarOpen(!isSidebarOpen)}
            className={styles.toggleButton}
          >
            {isSidebarOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
          </button>
        </div>
      </div>

      <div className={styles.layout}>
        {/* Sidebar */}
        <aside
          className={`
            ${isSidebarOpen ? 'block' : 'hidden'}
            lg:block
            ${styles.sidebar}
          `}
        >
          <div className="flex flex-col h-full">
            {/* Logo - Desktop only */}
            <div className={`hidden lg:flex ${styles.sidebarHeader}`}>
              <div className={styles.brandMark}>
                <Server className="h-5 w-5" />
              </div>
              <div>
                <h1 className={styles.sidebarTitle}>Deka</h1>
                <p className={styles.sidebarSubtitle}>User Portal</p>
              </div>
            </div>

            {/* Navigation */}
            <nav className={styles.nav}>
              {navigation.map((item) => {
                const isActive = pathname === item.href
                const Icon = item.icon
                return (
                  <Link
                    key={item.name}
                    href={item.href}
                    onClick={() => setIsSidebarOpen(false)}
                    className={`${styles.navLink} ${isActive ? styles.navLinkActive : ''}`}
                  >
                    <Icon className="h-5 w-5" />
                    <span>{item.name}</span>
                  </Link>
                )
              })}
            </nav>

            {/* User info & Sign Out */}
            <div className={styles.userBox}>
              <div className={styles.userMeta}>
                <p className={styles.userLabel}>Signed in as</p>
                <p className={styles.userName}>
                  <span className={styles.userIcon} aria-hidden="true">
                    <svg viewBox="0 0 360 320" fill="currentColor">
                      <path d="M180 141.964C163.699 110.262 119.308 51.1817 78.0347 22.044C38.4971 -5.86834 23.414 -1.03207 13.526 3.43594C2.08093 8.60755 0 26.1785 0 36.5164C0 46.8542 5.66748 121.272 9.36416 133.694C21.5786 174.738 65.0603 188.607 105.104 184.156C107.151 183.852 109.227 183.572 111.329 183.312C109.267 183.539 107.19 183.777 105.104 184.03C46.4204 192.038 -5.69621 214.388 62.6582 290.146C130.654 365.519 176.934 259.327 180 250.191C183.066 259.327 229.346 365.519 297.342 290.146C365.696 214.388 313.58 192.038 254.896 184.03C252.81 183.777 250.733 183.539 248.671 183.312C250.773 183.572 252.849 183.852 254.896 184.156C294.94 188.607 338.421 174.738 350.636 133.694C354.333 121.272 360 46.8542 360 36.5164C360 26.1785 357.919 8.60755 346.474 3.43594C336.586 -1.03207 321.503 -5.86834 281.965 22.044C240.692 51.1817 196.301 110.262 180 141.964Z" />
                    </svg>
                  </span>
                  @{user.username || user.address}
                </p>
              </div>
              <button
                onClick={handleSignOut}
                className={styles.signOut}
              >
                <LogOut className="h-5 w-5" />
                <span>Sign Out</span>
              </button>
            </div>
          </div>
        </aside>

        {/* Main content */}
        <main className={styles.main}>
          <div className={styles.mainInner}>
            {children}
          </div>
        </main>
      </div>

      {/* Mobile sidebar backdrop */}
      {isSidebarOpen && (
        <div
          className={`${styles.backdrop} lg:hidden`}
          onClick={() => setIsSidebarOpen(false)}
        />
      )}
    </div>
  )
}
