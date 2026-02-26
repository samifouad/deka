'use client'

import { useState } from 'react'
import { Sun, Moon } from 'lucide-react'
import { Button } from '@/components/ui/button'

interface ComponentPreviewProps {
  children: React.ReactNode
  showGrid?: boolean
}

export function ComponentPreview({ children, showGrid = false }: ComponentPreviewProps) {
  const [previewTheme, setPreviewTheme] = useState<'light' | 'dark'>('light')

  return (
    <div className="border border-border rounded-lg overflow-hidden">
      {/* Preview Toolbar */}
      <div className="flex items-center justify-between px-4 py-2 bg-secondary/30 border-b border-border">
        <div className="text-sm text-muted-foreground">Preview</div>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => setPreviewTheme(previewTheme === 'light' ? 'dark' : 'light')}
          className="h-8 w-8 p-0"
        >
          {previewTheme === 'dark' ? (
            <Sun className="w-4 h-4" />
          ) : (
            <Moon className="w-4 h-4" />
          )}
        </Button>
      </div>

      {/* Preview Area */}
      <div
        className={`${previewTheme} ${showGrid ? 'bg-grid-pattern' : ''}`}
        style={{ colorScheme: previewTheme }}
      >
        <div className={`
          min-h-[200px] flex items-center justify-center p-8
          ${previewTheme === 'dark' ? 'bg-slate-950 text-slate-50' : 'bg-white text-slate-950'}
        `}>
          {children}
        </div>
      </div>
    </div>
  )
}
