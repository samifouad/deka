'use client'

import { useEffect } from 'react'

declare global {
  interface Window {
    require: any
    monaco: any
    monacoLoaded?: boolean
    monacoLoading?: boolean
  }
}

export function MonacoLoader() {
  useEffect(() => {
    // Only load once
    if (window.monaco || window.monacoLoading) return

    window.monacoLoading = true

    const loaderScript = document.createElement('script')
    loaderScript.src = 'https://unpkg.com/monaco-editor@latest/min/vs/loader.js'

    loaderScript.onload = () => {
      window.require.config({
        paths: { vs: 'https://unpkg.com/monaco-editor@latest/min/vs' }
      })

      window.require(['vs/editor/editor.main'], () => {
        // Define docs theme
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

        // Define blueprint theme for examples
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

        // Add Tana type definitions for TypeScript intellisense
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

        // Mark as loaded
        window.monacoLoaded = true
        window.monacoLoading = false
      })
    }

    document.head.appendChild(loaderScript)
  }, [])

  return null
}
