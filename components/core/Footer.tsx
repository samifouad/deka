'use client'

import Link from 'next/link'
import { Sun, Moon } from 'lucide-react'
import { useTheme } from '@/context/theme-context'

const footerLinks = [
  { name: 'Help', href: '/help' },
  { name: 'CLI', href: '/cli' },
  { name: 'API', href: '/api' },
  { name: 'Blog', href: '/blog' },
  { name: 'Status', href: '/status' },
  { name: 'RFD', href: '/rfd' },
  { name: 'Sign In', href: '/signin' },
]

export default function Footer() {
  const { theme, toggleTheme } = useTheme()

  return (
    <footer className="static border-t border-black/10 bg-white/70 px-4 py-12 backdrop-blur-md dark:border-white/10 dark:bg-black/60 lg:px-8">
      <div className="mx-auto flex max-w-7xl flex-col items-center justify-between gap-4 md:flex-row">
        <Link href="/" className="group">
          <h3 className="text-xl font-semibold text-black transition-colors group-hover:text-black/70 dark:text-white dark:group-hover:text-white/80">
            deka
          </h3>
        </Link>
        <div className="flex flex-wrap items-center gap-2 text-sm text-black/70 dark:text-white/70">
          {footerLinks.map((item) => (
            <Link
              key={item.name}
              href={item.href}
              className="rounded-full px-3 py-2 transition-colors hover:bg-black/5 hover:text-black dark:hover:bg-white/10 dark:hover:text-white"
            >
              {item.name}
            </Link>
          ))}
          <a
            href="https://github.com/tananetwork"
            className="rounded-full px-3 py-2 transition-colors hover:bg-black/5 hover:text-black dark:hover:bg-white/10 dark:hover:text-white"
          >
            GitHub
          </a>
          <button
            onClick={toggleTheme}
            className="rounded-full p-2 transition-colors hover:bg-black/5 dark:hover:bg-white/10"
            aria-label="Toggle theme"
          >
            {theme === 'dark' ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
          </button>
        </div>
      </div>
    </footer>
  )
}
