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
]

export default function Footer() {
  const { theme, toggleTheme } = useTheme()

  return (
    <footer className="relative px-4 py-12 lg:px-8 border-t border-border">
      <div className="mx-auto max-w-7xl">
        <div className="flex flex-col md:flex-row justify-between items-center">
          <Link href="/" className="mb-4 md:mb-0 group">
            <h3 className="text-xl font-bold text-primary group-hover:text-primary/80 transition-colors">deka</h3>
          </Link>
          <div className="flex gap-2 text-sm text-muted-foreground items-center">
            {footerLinks.map((item) => (
              <Link
                key={item.name}
                href={item.href}
                className="px-3 py-2 rounded-lg hover:bg-accent hover:text-foreground transition-colors"
              >
                {item.name}
              </Link>
            ))}
            <a href="https://github.com/tananetwork" className="px-3 py-2 rounded-lg hover:bg-accent hover:text-foreground transition-colors">GitHub</a>
            <button
              onClick={toggleTheme}
              className="p-2 rounded-lg hover:bg-accent transition-colors cursor-pointer"
              aria-label="Toggle theme"
            >
              {theme === 'dark' ? (
                <Sun className="w-4 h-4" />
              ) : (
                <Moon className="w-4 h-4" />
              )}
            </button>
          </div>
        </div>
      </div>
    </footer>
  )
}
