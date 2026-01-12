'use client'

import { useEffect, useRef, useState } from 'react'
import { usePathname } from 'next/navigation'
import Link from 'next/link'
import { Menu, Search, X } from 'lucide-react'
import { Button } from '@/components/ui/button'

const navigation = [
  { name: 'serve', href: '/serve' },
  { name: 'run', href: '/run' },
  { name: 'build', href: '/build' },
  { name: 'compile', href: '/compile' },
  { name: 'deploy', href: '/deploy' }
]

type NavbarMode = 'sticky' | 'fixed'

export function Navbar({ mode = 'sticky' }: { mode?: NavbarMode }) {
  const [isOpen, setIsOpen] = useState(false)
  const [isScrolled, setIsScrolled] = useState(false)
  const [isSearchOpen, setIsSearchOpen] = useState(false)
  const searchRef = useRef<HTMLElement | null>(null)
  const inputRef = useRef<HTMLInputElement | null>(null)
  const pathname = usePathname()

  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 8)
    }

    handleScroll()
    window.addEventListener('scroll', handleScroll, { passive: true })
    return () => window.removeEventListener('scroll', handleScroll)
  }, [])

  useEffect(() => {
    if (!isSearchOpen) return

    const handleClick = (event: MouseEvent) => {
      if (!searchRef.current?.contains(event.target as Node)) {
        setIsSearchOpen(false)
      }
    }

    const focusTimer = window.setTimeout(() => {
      inputRef.current?.focus()
    }, 40)

    document.addEventListener('mousedown', handleClick)

    return () => {
      window.clearTimeout(focusTimer)
      document.removeEventListener('mousedown', handleClick)
    }
  }, [isSearchOpen])

  useEffect(() => {
    const handleKeydown = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null
      const tagName = target?.tagName?.toLowerCase()
      const isTypingField =
        tagName === 'input' ||
        tagName === 'textarea' ||
        target?.isContentEditable

      if (isTypingField) return

      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault()
        setIsSearchOpen((open) => !open)
      }

      if (event.key === 'Escape') {
        setIsSearchOpen(false)
      }
    }

    document.addEventListener('keydown', handleKeydown)
    return () => document.removeEventListener('keydown', handleKeydown)
  }, [])

  const toggleSearch = () => {
    setIsSearchOpen((open) => !open)
  }

  return (
    <nav
      ref={searchRef}
      className={`${mode === 'fixed' ? 'fixed left-0 right-0 top-0' : 'sticky top-0'} z-50 w-full transition-none ${
        isSearchOpen ? 'search-open search-active' : ''
      } ${
        isSearchOpen
          ? 'bg-white dark:bg-black'
          : isScrolled
          ? 'bg-white/60 backdrop-blur-xl shadow-[0_1px_0_rgba(0,0,0,0.02)] dark:bg-black/55'
          : 'bg-white/35 backdrop-blur-md dark:bg-black/30'
      }`}
    >
      <div className="relative z-50 mx-auto flex max-w-6xl items-center justify-between px-6 py-4">
        <Link href="/" className="text-lg font-semibold tracking-tight text-black dark:text-white">
          deka
        </Link>

        <div className="hidden md:flex items-center gap-2 text-sm text-black/90 dark:text-white/90">
          {navigation.map((item) => (
            <Link
              key={item.name}
              href={item.href}
              className={`rounded-full px-3 py-1.5 transition-colors hover:bg-black/5 hover:text-black dark:hover:bg-white/10 dark:hover:text-white ${
                pathname.startsWith(item.href)
                  ? 'bg-black/5 text-black dark:bg-white/10 dark:text-white'
                  : ''
              }`}
            >
              {item.name}
            </Link>
          ))}
          <Link
            href="/install"
            className="rounded-full px-3 py-1.5 font-semibold text-black transition-colors hover:bg-black/5 hover:text-black dark:text-white dark:hover:bg-white/10 dark:hover:text-white"
          >
            install
          </Link>
          <button
            type="button"
            aria-label="Search"
            onClick={toggleSearch}
            className="rounded-full p-2 text-black/90 transition-colors hover:bg-black/5 hover:text-black dark:text-white/90 dark:hover:bg-white/10 dark:hover:text-white"
          >
            <Search className="h-4 w-4" />
          </button>
        </div>

        <div className="md:hidden">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setIsOpen(!isOpen)}
            className="text-black/90 hover:text-black dark:text-white/90 dark:hover:text-white"
            aria-label="Toggle navigation"
          >
            {isOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
          </Button>
        </div>
      </div>

      {isOpen && (
        <div className="md:hidden border-t border-black/10 bg-white/90 px-6 py-4 dark:border-white/10 dark:bg-black/80">
          <div className="flex flex-col gap-3 text-sm text-black/90 dark:text-white/90">
            {navigation.map((item) => (
              <Link
                key={item.name}
                href={item.href}
                className={`rounded-full px-3 py-2 transition-colors hover:bg-black/5 hover:text-black dark:hover:bg-white/10 dark:hover:text-white ${
                  pathname.startsWith(item.href)
                    ? 'bg-black/5 text-black dark:bg-white/10 dark:text-white'
                    : ''
                }`}
                onClick={() => setIsOpen(false)}
              >
                {item.name}
              </Link>
            ))}
            <Link
              href="/install"
              className="rounded-full px-3 py-2 font-semibold text-black transition-colors hover:bg-black/5 hover:text-black dark:text-white dark:hover:bg-white/10 dark:hover:text-white"
              onClick={() => setIsOpen(false)}
            >
              install
            </Link>
            <div className="flex flex-col gap-2">
              <button
                type="button"
                aria-label="Search"
                onClick={toggleSearch}
                className="flex items-center gap-2 rounded-full px-3 py-2 text-black/90 transition-colors hover:bg-black/5 hover:text-black dark:text-white/90 dark:hover:bg-white/10 dark:hover:text-white"
              >
                <Search className="h-4 w-4" />
                search
              </button>
              {isSearchOpen && (
                <div className="rounded-2xl border border-black/10 bg-white/90 p-3 shadow-lg dark:border-white/10 dark:bg-black/80">
                  <div className="flex items-center gap-2 rounded-xl border border-black/10 bg-white px-3 py-2 text-sm text-black/70 dark:border-white/10 dark:bg-black/60 dark:text-white/70">
                    <Search className="h-4 w-4" />
                    <input
                      ref={inputRef}
                      type="search"
                      placeholder="Search docs, guides, pages"
                      className="w-full bg-transparent text-sm text-black placeholder:text-black/40 outline-none dark:text-white dark:placeholder:text-white/40"
                    />
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}

      <div className="search-panel hidden md:block absolute left-0 right-0 top-full z-40 h-[calc(100vh-72px)] border-t border-black/5 bg-white/80 px-6 py-10 backdrop-blur-2xl dark:border-white/10 dark:bg-black/70">
        <div className="mx-auto flex max-w-3xl items-center gap-3 rounded-full border border-black/10 bg-white px-6 py-4 text-base text-black/70 shadow-sm dark:border-white/10 dark:bg-black/60 dark:text-white/70">
          <Search className="h-5 w-5" />
          <input
            ref={inputRef}
            type="search"
            placeholder="Search docs, guides, pages"
            className="w-full bg-transparent text-base text-black placeholder:text-black/40 outline-none dark:text-white dark:placeholder:text-white/40"
          />
        </div>
      </div>
    </nav>
  )
}
