'use client'

import { useEffect, useRef, useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Copy, Check } from 'lucide-react'

interface CodeExampleProps {
  title: string
  code: string
  result: any
}

declare global {
  interface Window {
    require: any
    monaco: any
    monacoLoaded?: boolean
  }
}

export function CodeExample({ title, code, result }: CodeExampleProps) {
  const [copied, setCopied] = useState(false)
  const editorRef = useRef<HTMLDivElement>(null)
  const monacoRef = useRef<any>(null)

  const copyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(code)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch (err) {
      console.error('Failed to copy code:', err)
    }
  }

  const stripLeadingComments = (code: string): string => {
    const lines = code.split('\n')
    const firstNonCommentIndex = lines.findIndex(line => 
      line.trim() && !line.trim().startsWith('//')
    )
    
    if (firstNonCommentIndex === -1) {
      return code // All lines are comments or empty
    }
    
    return lines.slice(firstNonCommentIndex).join('\n')
  }

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
            // Add tana type definitions
            window.monaco.languages.typescript.typescriptDefaults.addExtraLib(
              `declare module 'tana/core' {
                export const console: {
                  log(...args: unknown[]): void;
                  error(...args: unknown[]): void;
                };
              }`,
              'ts:filename/tana-core.d.ts'
            )

            window.monaco.languages.typescript.typescriptDefaults.addExtraLib(
              `declare module 'tana/data' {
                export const data: {
                  set(key: string, value: string | object): Promise<void>;
                  get(key: string): Promise<string | object | null>;
                  commit(): Promise<void>;
                };
              }`,
              'ts:filename/tana-data.d.ts'
            )

            window.monaco.languages.typescript.typescriptDefaults.addExtraLib(
              `declare module 'tana/tx' {
                export const tx: {
                  transfer(from: string, to: string, amount: number, currency: string): void;
                };
              }`,
              'ts:filename/tana-tx.d.ts'
            )

            window.monaco.languages.typescript.typescriptDefaults.addExtraLib(
              `declare module 'tana/context' {
                export const context: {
                  caller(): any;
                  input(): any;
                };
              }`,
              'ts:filename/tana-context.d.ts'
            )

            // Define custom blueprint theme
            window.monaco.editor.defineTheme('blueprint', {
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
                { token: 'delimiter.bracket', foreground: '94a3b8' },
                { token: 'delimiter.parenthesis', foreground: '94a3b8' },
                { token: 'delimiter.square', foreground: '94a3b8' },
                { token: 'delimiter.angle', foreground: '94a3b8' },
                { token: 'delimiter.curly', foreground: '94a3b8' },
                { token: 'operator', foreground: 'f472b6' },
                { token: 'tag', foreground: '38bdf8' },
                { token: 'attribute.name', foreground: '86efac' },
                { token: 'attribute.value', foreground: 'fbbf24' },
                { token: 'regexp', foreground: 'a78bfa' },
                { token: 'meta', foreground: '64748b' }
              ],
              colors: {
                'editor.background': '#020817',
                'editor.foreground': '#f1f5f9',
                'editor.lineHighlightBackground': '#0f172a',
                'editor.selectionBackground': '#1e3a8a',
                'editor.inactiveSelectionBackground': '#1e293b',
                'editorCursor.foreground': '#60a5fa',
                'editorWhitespace.foreground': '#334155',
                'editorIndentGuide.background': '#1e293b',
                'editorIndentGuide.activeBackground': '#334155',
                'editorLineNumber.foreground': '#475569',
                'editorLineNumber.activeForeground': '#94a3b8',
                'editor.selectionHighlightBackground': '#1e3a8a80',
                'editor.wordHighlightBackground': '#1e3a8a60',
                'editor.wordHighlightStrongBackground': '#1e3a8a80',
                'editor.findMatchBackground': '#1e40af',
                'editor.findMatchHighlightBackground': '#1e3a8a60',
                'editor.hoverHighlightBackground': '#1e3a8a40',
                'editor.lineNumbersBorder': '#1e293b',
                'editorOverviewRuler.border': '#1e293b',
                'editorOverviewRuler.background': '#020817',
                'editorError.foreground': '#f87171',
                'editorWarning.foreground': '#fbbf24',
                'editorInfo.foreground': '#60a5fa',
                'editorHint.foreground': '#86efac',
                'editorWidget.background': '#0f172a',
                'editorWidget.border': '#334155',
                'editorSuggestWidget.background': '#0f172a',
                'editorSuggestWidget.border': '#334155',
                'editorSuggestWidget.foreground': '#f1f5f9',
                'editorSuggestWidget.highlightForeground': '#60a5fa',
                'editorSuggestWidget.selectedBackground': '#1e3a8a',
                'editorSuggestWidget.selectedForeground': '#f1f5f9',
                'editor.peekView.background': '#0f172a',
                'editor.peekView.border': '#334155',
                'editor.peekViewTitle.background': '#1e293b',
                'editor.peekViewResult.background': '#0f172a',
                'editor.peekViewEditor.background': '#020817',
                'editor.peekViewEditor.matchHighlightBackground': '#1e3a8a60',
                'editor.peekViewResult.selectionBackground': '#1e3a8a',
                'editor.peekViewResult.lineForeground': '#94a3b8',
                'editor.peekViewResult.fileForeground': '#60a5fa',
                'editor.peekViewResult.selectionForeground': '#f1f5f9',
                'editor.peekViewResult.matchHighlightBackground': '#1e3a8a60',
                'scrollbar.shadow': '#00000000',
                'scrollbarSlider.background': '#33415580',
                'scrollbarSlider.hoverBackground': '#47556980',
                'scrollbarSlider.activeBackground': '#64748b80',
                'progressBar.background': '#3b82f6',
                'badge.background': '#1e3a8a',
                'badge.foreground': '#f1f5f9'
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

      // Create new editor
      const cleanCode = stripLeadingComments(code)
      monacoRef.current = window.monaco.editor.create(editorRef.current!, {
        value: cleanCode,
        language: 'typescript',
        theme: 'blueprint',
        automaticLayout: true,
        minimap: { enabled: false },
        fontSize: 14,
        lineNumbers: 'on',
        scrollBeyondLastLine: false,
        wordWrap: 'on',
        readOnly: true,
        renderLineHighlight: 'none',
        scrollbar: {
          vertical: 'auto',
          horizontal: 'auto'
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
    }

    return () => {
      if (monacoRef.current) {
        monacoRef.current.dispose()
        monacoRef.current = null
      }
    }
  }, [code])

  return (
    <Card className="bg-card border-border backdrop-blur-sm">
      <CardHeader className="flex flex-row items-center justify-between">
        <div>
          <CardTitle className="text-foreground">{title}</CardTitle>
          <CardDescription className="text-muted-foreground">
            TypeScript smart contract
          </CardDescription>
        </div>
        <Button
          variant="ghost"
          size="sm"
          onClick={copyToClipboard}
          className="text-muted-foreground hover:text-foreground hover:bg-accent"
        >
          {copied ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
        </Button>
      </CardHeader>
      <CardContent>
        <div className="relative">
          <div
            ref={editorRef}
            className="bg-background/50 border border-border rounded-lg overflow-hidden"
            style={{
              height: Math.max(150, Math.min(500, code.split('\n').length * 18 + 40)) + 'px',
              minHeight: '150px'
            }}
          />
        </div>
      </CardContent>
    </Card>
  )
}