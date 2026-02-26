'use client'

import { useEffect, useRef } from 'react'
import { getDocsThemeName, getMonacoLanguage, waitForMonacoReady } from '@/lib/monaco'

interface DocsCodeBlockProps {
  code: string
  language: string
}

declare global {
  interface Window {
    require: any
    monaco: any
    monacoLoaded?: boolean
  }
}

export function DocsCodeBlock({ code, language }: DocsCodeBlockProps) {
  const editorRef = useRef<HTMLDivElement>(null)
  const monacoRef = useRef<any>(null)

  useEffect(() => {
    if (!editorRef.current) return

    // Create/update editor when Monaco is loaded
    const createEditor = () => {
      // Dispose existing editor if it exists
      if (monacoRef.current) {
        monacoRef.current.dispose()
        monacoRef.current = null
      }

      // Clear the container
      if (editorRef.current) {
        editorRef.current.innerHTML = ''
      }

      // Map language names to Monaco language IDs
      const monacoLanguage = getMonacoLanguage(language)
      const themeName = getDocsThemeName()
      const lineHeight = 21
      const padding = 16

      // Create new editor
      monacoRef.current = window.monaco.editor.create(editorRef.current!, {
        value: code.trim(),
        language: monacoLanguage,
        theme: themeName,
        automaticLayout: true,
        minimap: { enabled: false },
        fontFamily: 'Inconsolata, ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace',
        fontSize: 14,
        lineHeight,
        lineNumbers: 'on',
        scrollBeyondLastLine: false,
        wordWrap: 'on',
        readOnly: true,
        renderLineHighlight: 'none',
        lineNumbersMinChars: 4,
        glyphMargin: false,
        folding: false,
        lineDecorationsWidth: 12,
        overviewRulerBorder: false,
        overviewRulerLanes: 0,
        hideCursorInOverviewRuler: true,
        scrollbar: {
          vertical: 'hidden',
          horizontal: 'hidden',
          verticalScrollbarSize: 10,
          horizontalScrollbarSize: 10,
        },
        padding: {
          top: padding,
          bottom: padding,
        }
      })
    }

    if (window.monaco && window.monacoLoaded) {
      createEditor()
    } else {
      waitForMonacoReady().then(() => {
        if (editorRef.current) {
          createEditor()
        }
      })
    }

    return () => {
      if (monacoRef.current) {
        monacoRef.current.dispose()
        monacoRef.current = null
      }
    }
  }, [code, language])

  return (
    <div
      ref={editorRef}
      className="docs-monaco"
      style={{
        height: Math.max(80, code.trim().split('\n').length * lineHeight + padding * 2 + 6) + 'px',
        minHeight: '80px'
      }}
    />
  )
}
