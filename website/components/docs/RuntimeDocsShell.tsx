'use client'

import { useEffect, useRef, useState } from 'react'
import { usePathname } from 'next/navigation'
import { Navbar } from '@/components/landing/Navbar'
import { RuntimeSidebar } from '@/components/runtime-docs/RuntimeSidebar'
import { Menu, Search } from 'lucide-react'
import { Button } from '@/components/ui/button'
import type { RuntimeLanguage, RuntimeSidebarSection } from '@/lib/runtime-docs'

interface RuntimeDocsShellProps {
  children: React.ReactNode
  language: RuntimeLanguage
  sections?: RuntimeSidebarSection[]
  searchPlaceholder: string
}

export function RuntimeDocsShell({
  children,
  language,
  sections,
  searchPlaceholder,
}: RuntimeDocsShellProps) {
  const [sidebarOpen, setSidebarOpen] = useState(false)
  const pathname = usePathname()
  const mainRef = useRef<HTMLElement>(null)

  useEffect(() => {
    mainRef.current?.scrollTo({ top: 0, left: 0, behavior: 'auto' })
  }, [pathname])

  return (
    <div className="h-screen bg-background text-foreground overflow-hidden flex flex-col">
      <Navbar />

      <div className="md:hidden sticky top-16 z-40 bg-background border-b border-border p-4 flex items-center justify-between">
        <Button
          variant="ghost"
          size="sm"
          onClick={() => setSidebarOpen(!sidebarOpen)}
        >
          <Menu className="w-5 h-5" />
        </Button>
        <div className="flex-1 mx-4">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <input
              type="text"
              placeholder={searchPlaceholder}
              className="w-full pl-9 pr-4 py-2 bg-secondary/50 border border-border rounded-lg text-sm focus:outline-none focus:border-primary"
            />
          </div>
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        <RuntimeSidebar isOpen={sidebarOpen} language={language} sections={sections} />
        <main ref={mainRef} className="flex-1 overflow-y-auto scrollbar-hide bg-secondary/30">
          {children}
        </main>
      </div>
    </div>
  )
}
