'use client'

import { useEffect, useRef } from 'react'
import { getDocsThemeName, getMonacoLanguage, waitForMonacoReady } from '@/lib/monaco'

interface DocsContentProps {
  html: string
  codeBlocks: Array<{lang: string, code: string}>
}

declare global {
  interface Window {
    require: any
    monaco: any
    monacoLoaded?: boolean
    monacoLoading?: boolean
  }
}

export function DocsContent({ html, codeBlocks }: DocsContentProps) {
  const contentRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!contentRef.current) return

    let cancelled = false

    const hydrateCodeBlocks = async () => {
      await waitForMonacoReady()
      if (cancelled || !contentRef.current) return

      const renderEditor = (container: HTMLElement, block: { lang: string, code: string }) => {
        const monacoLanguage = getMonacoLanguage(block.lang)
        const lineHeight = 21
        const padding = 16
        const lineCount = block.code.split('\n').length
        const height = Math.max(80, lineCount * lineHeight + padding * 2 + 6)
        const themeName = getDocsThemeName()

        container.className = 'docs-monaco'
        container.style.height = `${height}px`
        container.style.minHeight = '80px'

        const editorDiv = document.createElement('div')
        editorDiv.style.width = '100%'
        editorDiv.style.height = '100%'
        container.appendChild(editorDiv)

        window.monaco.editor.create(editorDiv, {
          value: block.code,
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
            alwaysConsumeMouseWheel: false,
          },
          padding: {
            top: padding,
            bottom: padding,
          }
        })
      }

      // Find all code block placeholders
      const placeholders = contentRef.current!.querySelectorAll('.code-block-placeholder')

      placeholders.forEach((placeholder) => {
        const index = parseInt(placeholder.getAttribute('data-index') || '0')
        const block = codeBlocks[index]
        if (!block) return

        const container = document.createElement('div')
        placeholder.parentNode?.replaceChild(container, placeholder)
        renderEditor(container, block)
      })

      // Fallback for pre > code blocks (non-placeholder HTML)
      const preBlocks = contentRef.current!.querySelectorAll('pre > code')
      preBlocks.forEach((codeElement) => {
        const pre = codeElement.parentElement
        if (!pre) return

        const datasetLang = pre.getAttribute('data-language')
        const classLang = Array.from(codeElement.classList)
          .map((name) => name.replace(/^language-/, '').replace(/^lang-/, ''))
          .find((name) => name !== 'language' && name !== 'lang' && name !== 'code')

        const language = (datasetLang || classLang || 'text').toLowerCase()
        const code = codeElement.textContent || ''

        const container = document.createElement('div')
        pre.parentNode?.replaceChild(container, pre)
        renderEditor(container, { lang: language, code })
      })

    }

    hydrateCodeBlocks()

    return () => {
      cancelled = true
    }
  }, [html, codeBlocks])

  return (
    <div
      ref={contentRef}
      className="docs-content"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  )
}
