'use client'

import { useState } from 'react'
import { Navbar } from '@/components/landing/Navbar'
import { DocsSidebar } from './DocsSidebar'
import { SidebarSection } from '@/lib/docs'
import { Menu, Search } from 'lucide-react'
import { Button } from '@/components/ui/button'

interface DocsLayoutProps {
  children: React.ReactNode
  sidebar: SidebarSection[]
  slug: string[]
}

export function DocsLayout({ children, sidebar, slug }: DocsLayoutProps) {
  const [sidebarOpen, setSidebarOpen] = useState(false)

  return (
    <div className="h-screen bg-background text-foreground overflow-hidden flex flex-col">
      <Navbar />

      {/* Mobile header with menu toggle and search */}
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
              placeholder="Search docs..."
              className="w-full pl-9 pr-4 py-2 bg-secondary/50 border border-border rounded-lg text-sm focus:outline-none focus:border-primary"
            />
          </div>
        </div>
      </div>

      {/* Two-panel layout */}
      <div className="flex flex-1 overflow-hidden">
        {/* Left Sidebar - Navigation */}
        <DocsSidebar sections={sidebar} isOpen={sidebarOpen} />

        {/* Main Content */}
        <main className="flex-1 overflow-y-auto scrollbar-hide bg-secondary/30">
          <div className="max-w-5xl mx-auto px-8 py-12">
            {children}
          </div>
        </main>
      </div>
    </div>
  )
}
