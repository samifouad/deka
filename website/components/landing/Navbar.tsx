'use client'

import { useEffect, useRef, useState } from 'react'
import { usePathname } from 'next/navigation'
import Link from 'next/link'
import { Menu, Search, X } from 'lucide-react'
import { Button } from '@/components/ui/button'

const navigation = [
  { name: 'run', href: '/run' },
  { name: 'serve', href: '/serve' },
  { name: 'build', href: '/build' },
  { name: 'install', href: '/install' },
  { name: 'create', href: '/create' },
  { name: 'introspect', href: '/introspect' },
  { name: 'compile', href: '/compile' },
  { name: 'desktop', href: '/desktop' },
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
        setIsSearchOpen((open) => {
          const next = !open
          if (next) {
            setIsOpen(false)
          }
          return next
        })
      }

      if (event.key === 'Escape') {
        setIsSearchOpen(false)
      }
    }

    document.addEventListener('keydown', handleKeydown)
    return () => document.removeEventListener('keydown', handleKeydown)
  }, [])

  const toggleSearch = () => {
    setIsSearchOpen((open) => {
      const next = !open
      if (next) {
        setIsOpen(false)
      }
      return next
    })
  }

  return (
    <nav
      ref={searchRef}
      className={`${mode === 'fixed' ? 'fixed left-0 right-0 top-0' : 'sticky top-0'} z-50 w-full transition-none ${
        isSearchOpen ? 'search-open search-active' : ''
      } ${
        isSearchOpen || isOpen
          ? 'bg-white dark:bg-black'
          : isScrolled
          ? 'bg-white/60 backdrop-blur-xl shadow-[0_1px_0_rgba(0,0,0,0.02)] dark:bg-black/55'
          : 'bg-white/35 backdrop-blur-md dark:bg-black/30'
      }`}
    >
      <div className="relative z-50 mx-auto flex max-w-6xl items-center justify-between px-6 py-4">
        <Link
          href="/"
          className="flex items-center gap-2 text-xl font-semibold tracking-tight text-black dark:text-white"
        >
          <img
            src="/android-chrome-512x512.png"
            alt="Deka"
            className="h-9 w-9 rounded-full"
          />
          deka
        </Link>

        <div className="hidden lg:flex items-center gap-2 text-sm text-black/90 dark:text-white/90 max-[1100px]:text-xs">
          {navigation.map((item) => (
            <Link
              key={item.name}
              href={item.href}
              className={`rounded-full px-3 py-1.5 transition-colors hover:bg-black/5 hover:text-black dark:hover:bg-white/10 dark:hover:text-white ${
                !isSearchOpen && pathname.startsWith(item.href)
                  ? 'bg-black/5 text-black dark:bg-white/10 dark:text-white'
                  : ''
              }`}
            >
              {item.name}
            </Link>
          ))}
          <button
            type="button"
            aria-label="Search"
            onClick={toggleSearch}
            className={`rounded-full p-2 text-black/90 transition-colors hover:bg-black/5 hover:text-black dark:text-white/90 dark:hover:bg-white/10 dark:hover:text-white ${
              isSearchOpen ? 'bg-black/5 text-black dark:bg-white/10 dark:text-white' : ''
            }`}
          >
            <Search className="h-4 w-4" />
          </button>
        </div>

        <div className="lg:hidden flex items-center gap-2">
          <button
            type="button"
            aria-label="Search"
            onClick={toggleSearch}
            className={`rounded-full p-2 text-black/90 transition-colors hover:bg-black/5 hover:text-black dark:text-white/90 dark:hover:bg-white/10 dark:hover:text-white ${
              isSearchOpen ? 'bg-black/5 text-black dark:bg-white/10 dark:text-white' : ''
            }`}
          >
            <Search className="h-4 w-4" />
          </button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => {
              setIsSearchOpen(false)
              setIsOpen((open) => !open)
            }}
            className="text-black/90 hover:text-black dark:text-white/90 dark:hover:text-white"
            aria-label="Toggle navigation"
          >
            {isOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
          </Button>
        </div>
      </div>

      <div
        aria-hidden={!isOpen}
        className={`lg:hidden absolute left-0 right-0 top-full z-40 h-[calc(100vh-72px)] h-[calc(100dvh-72px)] border-t border-black/5 bg-white/80 px-6 py-10 backdrop-blur-2xl transition-opacity duration-700 ease-[cubic-bezier(0.32,0.72,0,1)] dark:border-white/10 dark:bg-black/70 ${
          isOpen ? 'opacity-100' : 'opacity-0 pointer-events-none'
        }`}
      >
        <div
          className={`mx-auto flex h-full max-w-3xl flex-col gap-3 overflow-y-auto text-sm text-black/90 transition-transform duration-700 ease-[cubic-bezier(0.32,0.72,0,1)] dark:text-white/90 ${
            isOpen ? 'translate-y-0' : '-translate-y-2'
          }`}
        >
          {navigation.map((item) => (
            <Link
              key={item.name}
              href={item.href}
              tabIndex={isOpen ? 0 : -1}
              className={`rounded-full px-3 py-2 transition-colors hover:bg-black/5 hover:text-black dark:hover:bg-white/10 dark:hover:text-white ${
                !isSearchOpen && pathname.startsWith(item.href)
                  ? 'bg-black/5 text-black dark:bg-white/10 dark:text-white'
                  : ''
              }`}
              onClick={() => setIsOpen(false)}
            >
              {item.name}
            </Link>
          ))}
        </div>
      </div>

      <div className="search-panel absolute left-0 right-0 top-full z-40 h-[calc(100vh-72px)] h-[calc(100dvh-72px)] border-t border-black/5 bg-white/80 px-6 py-10 backdrop-blur-2xl dark:border-white/10 dark:bg-black/70">
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
