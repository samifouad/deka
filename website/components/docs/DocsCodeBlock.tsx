'use client'

import { useEffect, useRef } from 'react'

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

    // Only load Monaco once globally
    if (!window.monaco) {
      const loadMonaco = async () => {
        const loaderScript = document.createElement('script')
        loaderScript.src = 'https://unpkg.com/monaco-editor@latest/min/vs/loader.js'
        loaderScript.onload = () => {
          window.require.config({
            paths: { vs: 'https://unpkg.com/monaco-editor@latest/min/vs' }
          })

          window.require(['vs/editor/editor.main'], () => {
            // Define custom theme matching docs
            window.monaco.editor.defineTheme('docs-dark', {
              base: 'vs-dark',
              inherit: true,
              rules: [
                { token: 'comment', foreground: '64748b' },
                { token: 'keyword', foreground: '38bdf8', fontStyle: 'bold' },
                { token: 'string', foreground: '86efac' },
                { token: 'number', foreground: 'fbbf24' },
                { token: 'type', foreground: 'c084fc', fontStyle: 'italic' },
                { token: 'function', foreground: '60a5fa' },
                { token: 'variable', foreground: 'e2e8f0' },
                { token: 'identifier', foreground: 'e2e8f0' },
                { token: 'delimiter', foreground: '94a3b8' },
                { token: 'operator', foreground: 'f472b6' },
              ],
              colors: {
                'editor.background': '#14151a',
                'editor.foreground': '#f1f5f9',
                'editor.lineHighlightBackground': '#1a1b21',
                'editorLineNumber.foreground': '#475569',
                'editorLineNumber.activeForeground': '#94a3b8',
              }
            })

            // Mark Monaco as loaded
            window.monacoLoaded = true
          })
        }
        document.head.appendChild(loaderScript)
      }

      loadMonaco()
    }

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

      const monacoLanguage = languageMap[language.toLowerCase()] || 'plaintext'

      // Create new editor
      monacoRef.current = window.monaco.editor.create(editorRef.current!, {
        value: code.trim(),
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
    }

    if (window.monaco && window.monacoLoaded) {
      createEditor()
    } else {
      // Wait for Monaco to load
      const checkMonaco = setInterval(() => {
        if (window.monaco && window.monacoLoaded) {
          clearInterval(checkMonaco)
          createEditor()
        }
      }, 100)

      return () => clearInterval(checkMonaco)
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
      className="bg-card border border-border rounded-lg overflow-hidden my-4"
      style={{
        height: Math.max(100, Math.min(600, code.trim().split('\n').length * 19 + 24)) + 'px',
        minHeight: '100px'
      }}
    />
  )
}
