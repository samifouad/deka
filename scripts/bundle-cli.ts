#!/usr/bin/env bun
/**
 * Pre-bundle all CLI markdown files into a static JSON file
 * for Cloudflare Workers compatibility
 */

import fs from 'fs'
import path from 'path'
import matter from 'gray-matter'
import { remark } from 'remark'
import html from 'remark-html'

interface CLIDoc {
  slug: string[]
  metadata: {
    title: string
    description?: string
  }
  content: string
  html: string
  codeBlocks: Array<{lang: string, code: string}>
}

const cliDirectory = path.join(process.cwd(), 'content/cli')
const outputPath = path.join(process.cwd(), 'lib/bundled-cli.json')

async function convertToHtml(content: string): Promise<string> {
  try {
    const result = await remark()
      .use(html, { sanitize: false })
      .process(content)
    return result.toString()
  } catch (error) {
    console.error('Error converting markdown:', error)
    return `<p>Error rendering content</p>`
  }
}

async function getAllCLIDocs(): Promise<CLIDoc[]> {
  const docs: CLIDoc[] = []

  async function readDir(dir: string, slugParts: string[] = []) {
    const files = fs.readdirSync(dir)

    for (const file of files) {
      const filePath = path.join(dir, file)
      const stat = fs.statSync(filePath)

      if (stat.isDirectory()) {
        await readDir(filePath, [...slugParts, file])
      } else if (file.endsWith('.md') || file.endsWith('.mdx')) {
        const fileContents = fs.readFileSync(filePath, 'utf8')
        const { data, content } = matter(fileContents)

        const fileName = file.replace(/\.mdx?$/, '')
        const slug = [...slugParts, fileName]

        // Extract code blocks
        const codeBlocks: Array<{lang: string, code: string}> = []
        const contentWithPlaceholders = content.replace(/```(\w+)?\n([\s\S]*?)```/g, (match, lang, code) => {
          const index = codeBlocks.length
          codeBlocks.push({ lang: lang || 'text', code: code.trim() })
          return `\n\n<div class="code-block-placeholder" data-index="${index}"></div>\n\n`
        })

        // Convert markdown to HTML
        const htmlContent = await convertToHtml(contentWithPlaceholders)

        docs.push({
          slug,
          metadata: {
            title: data.title || fileName,
            description: data.description,
          },
          content,
          html: htmlContent,
          codeBlocks,
        })
      }
    }
  }

  await readDir(cliDirectory)
  return docs
}

// Generate the bundle
console.log('📦 Bundling CLI docs...')
const docs = await getAllCLIDocs()
const bundleContent = JSON.stringify(docs, null, 2)

// Write to output file
fs.writeFileSync(outputPath, bundleContent, 'utf8')

console.log(`✅ Bundled ${docs.length} CLI docs to ${outputPath}`)
console.log(`📊 Bundle size: ${(bundleContent.length / 1024).toFixed(2)} KB`)
