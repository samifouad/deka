'use client'

import { useState } from 'react'
import { Navbar } from '@/components/landing/Navbar'
import { APISidebar } from '@/components/api-docs/APISidebar'
import { Menu, Search } from 'lucide-react'
import { Button } from '@/components/ui/button'

export default function DocsDeployLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const [sidebarOpen, setSidebarOpen] = useState(false)

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
              placeholder="Search deploy server docs..."
              className="w-full pl-9 pr-4 py-2 bg-secondary/50 border border-border rounded-lg text-sm focus:outline-none focus:border-primary"
            />
          </div>
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        <APISidebar isOpen={sidebarOpen} basePath="/docs/deploy" />
        <main className="flex-1 overflow-y-auto scrollbar-hide bg-secondary/30">
          {children}
        </main>
      </div>
    </div>
  )
}
