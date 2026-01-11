'use client'

import { useState } from 'react'
import { usePathname } from 'next/navigation'
import Link from 'next/link'
import { Button } from '@/components/ui/button'
import { Menu, X } from 'lucide-react'

export function Navbar() {
  const [isOpen, setIsOpen] = useState(false)
  const pathname = usePathname()

  const isActive = (href: string) => {
    if (href === '/') return pathname === '/'
    return pathname.startsWith(href)
  }

  const navigation = [
    { name: 'JavaScript', href: '/' },
    { name: 'PHP', href: '/php' },
    { name: 'CLI', href: '/cli' },
    { name: 'Platform', href: '/platform' },
  ]

  return (
    <nav className="sticky top-0 z-50 px-4 py-6 lg:px-8 bg-card/25 backdrop-blur-md border-b border-border">
      <div className="mx-auto max-w-7xl">
        <div className="flex items-center justify-between">
          {/* Logo */}
          <Link href="/" className="flex items-center group">
            <div className="flex items-center space-x-3">
              <div className="w-8 h-8 bg-primary border border-primary/30 flex items-center justify-center transform rotate-45 group-hover:bg-primary/80 transition-colors">
                <div className="w-3 h-3 bg-background transform -rotate-45"></div>
              </div>
              <span className="text-xl font-bold text-primary group-hover:text-primary/80 transition-colors">deka</span>
            </div>
          </Link>

          {/* Desktop Navigation */}
          <div className="hidden md:flex items-center space-x-2">
            {navigation.map((item) => (
              <Link
                key={item.name}
                href={item.href}
                className={`px-3 py-1.5 rounded-md transition-colors duration-200 ${
                  isActive(item.href)
                    ? 'bg-primary/20 text-foreground font-semibold'
                    : 'text-muted-foreground hover:text-foreground hover:bg-secondary'
                }`}
              >
                {item.name}
              </Link>
            ))}
            <Button
              asChild
              className="bg-primary hover:bg-primary/90 text-primary-foreground border border-primary/30"
            >
              <Link href="/signin">Sign In</Link>
            </Button>
          </div>

          {/* Mobile menu button */}
          <div className="md:hidden">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setIsOpen(!isOpen)}
              className="text-muted-foreground hover:text-foreground hover:bg-accent"
            >
              {isOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
            </Button>
          </div>
        </div>

        {/* Mobile Navigation */}
        {isOpen && (
          <div className="md:hidden mt-4 pt-4 border-t border-border">
            <div className="flex flex-col space-y-4">
              {navigation.map((item) => (
                <Link
                  key={item.name}
                  href={item.href}
                  className={`px-3 py-2 rounded-md transition-colors duration-200 ${
                    isActive(item.href)
                      ? 'bg-primary/20 text-foreground font-semibold'
                      : 'text-muted-foreground hover:text-foreground hover:bg-secondary'
                  }`}
                  onClick={() => setIsOpen(false)}
                >
                  {item.name}
                </Link>
              ))}
              <Button
                asChild
                className="bg-primary hover:bg-primary/90 text-primary-foreground border border-primary/30 w-full flex items-center justify-center"
                onClick={() => setIsOpen(false)}
              >
                <Link href="/signin">Sign In</Link>
              </Button>
            </div>
          </div>
        )}
      </div>
    </nav>
  )
}
