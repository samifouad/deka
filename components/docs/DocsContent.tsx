'use client'

import { useEffect, useRef, useState } from 'react'

interface DocsContentProps {
  html: string
  codeBlocks: Array<{lang: string, code: string}>
}

declare global {
  interface Window {
    monaco: any
    monacoLoaded?: boolean
  }
}

export function DocsContent({ html, codeBlocks }: DocsContentProps) {
  const contentRef = useRef<HTMLDivElement>(null)
  const [isHydrated, setIsHydrated] = useState(false)

  useEffect(() => {
    if (!contentRef.current) return

    const hydrateCodeBlocks = () => {
      if (!window.monaco || !window.monacoLoaded) {
        // Wait for Monaco to load
        setTimeout(hydrateCodeBlocks, 100)
        return
      }

      // Find all code block placeholders
      const placeholders = contentRef.current!.querySelectorAll('.code-block-placeholder')

      placeholders.forEach((placeholder) => {
        const index = parseInt(placeholder.getAttribute('data-index') || '0')
        const block = codeBlocks[index]

        if (!block) return

        // Map language names to Monaco language IDs
        const languageMap: Record<string, string> = {
          'bash': 'shell',
          'sh': 'shell',
          'shell': 'shell',
          'typescript': 'typescript',
          'ts': 'typescript',
          'javascript': 'javascript',
          'js': 'javascript',
          'json': 'json',
          'markdown': 'markdown',
          'md': 'markdown',
          'html': 'html',
          'css': 'css',
          'text': 'plaintext',
        }

        const monacoLanguage = languageMap[block.lang.toLowerCase()] || 'plaintext'

        // Create container
        const container = document.createElement('div')
        container.className = 'bg-card border border-border rounded-lg overflow-hidden my-4'
        const height = Math.max(100, Math.min(600, block.code.split('\n').length * 19 + 24))
        container.style.height = `${height}px`
        container.style.minHeight = '100px'

        // Create editor div
        const editorDiv = document.createElement('div')
        editorDiv.style.width = '100%'
        editorDiv.style.height = '100%'
        container.appendChild(editorDiv)

        // Replace placeholder with container
        placeholder.parentNode?.replaceChild(container, placeholder)

        // Create Monaco editor
        window.monaco.editor.create(editorDiv, {
          value: block.code,
          language: monacoLanguage,
          theme: 'docs-dark',
          automaticLayout: true,
          minimap: { enabled: false },
          fontSize: 14,
          lineNumbers: 'on',
          scrollBeyondLastLine: false,
          wordWrap: 'off',
          readOnly: true,
          renderLineHighlight: 'none',
          lineNumbersMinChars: 3,
          glyphMargin: false,
          folding: false,
          lineDecorationsWidth: 0,
          overviewRulerBorder: false,
          overviewRulerLanes: 0,
          hideCursorInOverviewRuler: true,
          scrollbar: {
            vertical: 'auto',
            horizontal: 'auto',
            verticalScrollbarSize: 10,
            horizontalScrollbarSize: 10,
          },
          padding: {
            top: 12,
            bottom: 12,
          }
        })
      })

      setIsHydrated(true)
    }

    hydrateCodeBlocks()
  }, [html, codeBlocks])

  return (
    <div
      ref={contentRef}
      className="docs-content"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  )
}
