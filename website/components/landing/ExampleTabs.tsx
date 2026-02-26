'use client'

import { Button } from '@/components/ui/button'
import { useEffect, useState } from 'react'

interface CodeExample {
  id: string
  title: string
  description: string
  code: string
  result: any
  category: string
}

interface ExampleTabsProps {
  onExampleSelect: (example: CodeExample) => void
  selectedExample: CodeExample | null
}

export function ExampleTabs({ onExampleSelect, selectedExample }: ExampleTabsProps) {
  const [codeExamples, setCodeExamples] = useState<CodeExample[]>([])

  useEffect(() => {
    // Load examples dynamically
    const loadExamples = async () => {
      try {
        const response = await fetch('/api/examples')
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`)
        }
        const examples = await response.json()
        console.log('Loaded examples:', examples.length) // Debug log
        setCodeExamples(examples)
        
        // Auto-select first example if no example is currently selected
        if (examples.length > 0 && !selectedExample) {
          onExampleSelect(examples[0])
        }
      } catch (error) {
        console.error('Failed to load examples:', error)
      }
    }

    loadExamples()
  }, [])

  if (codeExamples.length === 0) {
    return (
      <div className="space-y-6">
        <div className="flex gap-2 overflow-x-auto pb-2 scrollbar-hide">
          <div className="text-muted-foreground">Loading examples...</div>
        </div>
        <div className="text-muted-foreground/50 text-sm">
          Debug: Check browser console for errors
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Horizontally scrollable chip tabs */}
      <div className="relative">
        <div className="absolute left-0 top-0 bottom-0 w-8 bg-gradient-to-r from-background to-transparent z-10 pointer-events-none"></div>
        <div className="absolute right-0 top-0 bottom-0 w-8 bg-gradient-to-l from-background to-transparent z-10 pointer-events-none"></div>
        
        <div className="flex gap-2 overflow-x-auto pb-2 scrollbar-hide">
          {codeExamples.length > 0 && codeExamples.map((example) => (
            <Button
              key={example.id}
              variant={selectedExample?.id === example.id ? "default" : "outline"}
              size="sm"
              onClick={() => onExampleSelect(example)}
              className={`whitespace-nowrap flex-shrink-0 ${
                selectedExample?.id === example.id
                  ? "bg-primary border-primary text-primary-foreground font-bold"
                  : "border-border text-foreground hover:bg-accent"
              }`}
            >
              {example.title}
            </Button>
          ))}
        </div>
      </div>
    </div>
  )
}