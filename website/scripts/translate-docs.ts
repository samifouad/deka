#!/usr/bin/env bun
/**
 * Translate markdown/MDX docs into a target locale using an LLM.
 * Usage:
 *   bun scripts/translate-docs.ts --source content --out content-i18n/es --from en --to es
 *   bun scripts/translate-docs.ts --source content --out content-i18n/es --from en --to es --force
 */

import fs from 'fs'
import path from 'path'
import matter from 'gray-matter'

type TranslateOptions = {
  source: string
  out: string
  from: string
  to: string
  dryRun: boolean
  limit: number | null
  force: boolean
}

type Placeholder = {
  token: string
  value: string
}

const DEFAULT_MODEL = 'gpt-4o-mini'

function parseArgs(argv: string[]): TranslateOptions {
  const args = new Map<string, string | boolean>()
  for (let i = 2; i < argv.length; i += 1) {
    const arg = argv[i]
    if (!arg.startsWith('--')) continue
    const [key, value] = arg.split('=')
    if (value === undefined) {
      const next = argv[i + 1]
      if (next && !next.startsWith('--')) {
        args.set(key, next)
        i += 1
      } else {
        args.set(key, true)
      }
    } else {
      args.set(key, value)
    }
  }

  const source = String(args.get('--source') || '')
  const out = String(args.get('--out') || '')
  const from = String(args.get('--from') || 'en')
  const to = String(args.get('--to') || '')
  const dryRun = Boolean(args.get('--dry-run'))
  const limitRaw = args.get('--limit')
  const limit = limitRaw ? Number(limitRaw) : null
  const force = Boolean(args.get('--force'))

  if (!source || !out || !to) {
    throw new Error('Missing required args. Use --source, --out, --to')
  }

  return { source, out, from, to, dryRun, limit, force }
}

function listDocs(dir: string): string[] {
  const entries = fs.readdirSync(dir, { withFileTypes: true })
  const files: string[] = []
  for (const entry of entries) {
    const entryPath = path.join(dir, entry.name)
    if (entry.isDirectory()) {
      files.push(...listDocs(entryPath))
    } else if (entry.name.endsWith('.md') || entry.name.endsWith('.mdx')) {
      files.push(entryPath)
    }
  }
  return files
}

function maskCodeBlocks(input: string): { text: string; placeholders: Placeholder[] } {
  const placeholders: Placeholder[] = []
  let index = 0
  const text = input.replace(/```[\s\S]*?```/g, (match) => {
    const token = `@@CODEBLOCK_${index}@@`
    placeholders.push({ token, value: match })
    index += 1
    return token
  })
  return { text, placeholders }
}

function maskInlineCode(input: string, offset: number): { text: string; placeholders: Placeholder[] } {
  const placeholders: Placeholder[] = []
  let index = offset
  const text = input.replace(/`[^`]+`/g, (match) => {
    const token = `@@INLINE_${index}@@`
    placeholders.push({ token, value: match })
    index += 1
    return token
  })
  return { text, placeholders }
}

function unmaskAll(input: string, placeholders: Placeholder[]): string {
  let output = input
  for (const { token, value } of placeholders) {
    output = output.split(token).join(value)
  }
  return output
}

async function translateText(text: string, from: string, to: string): Promise<string> {
  const apiKey = process.env.OPENAI_API_KEY
  if (!apiKey) {
    throw new Error('OPENAI_API_KEY is required for translation.')
  }

  const model = process.env.OPENAI_MODEL || DEFAULT_MODEL
  const response = await fetch('https://api.openai.com/v1/chat/completions', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${apiKey}`,
    },
    body: JSON.stringify({
      model,
      temperature: 0.2,
      messages: [
        {
          role: 'system',
          content:
            'You are a translation engine. Preserve markdown, code fences, inline code, URLs, and MDX tags. Return only translated text.',
        },
        {
          role: 'user',
          content: `Translate from ${from} to ${to}:\n\n${text}`,
        },
      ],
    }),
  })

  if (!response.ok) {
    const errorBody = await response.text()
    throw new Error(`Translation failed: ${response.status} ${errorBody}`)
  }

  const data = await response.json()
  const content = data?.choices?.[0]?.message?.content
  if (!content) {
    throw new Error('Translation returned empty content.')
  }
  return content.trim()
}

function translateFrontmatter(data: Record<string, unknown>, from: string, to: string): Promise<Record<string, unknown>> {
  const entries = Object.entries(data)
  const next: Record<string, unknown> = {}

  const translateValue = async (value: unknown): Promise<unknown> => {
    if (typeof value === 'string') {
      return translateText(value, from, to)
    }
    if (Array.isArray(value)) {
      const translated = []
      for (const item of value) {
        translated.push(await translateValue(item))
      }
      return translated
    }
    if (value && typeof value === 'object') {
      const objEntries = Object.entries(value as Record<string, unknown>)
      const obj: Record<string, unknown> = {}
      for (const [key, innerValue] of objEntries) {
        obj[key] = await translateValue(innerValue)
      }
      return obj
    }
    return value
  }

  return (async () => {
    for (const [key, value] of entries) {
      next[key] = await translateValue(value)
    }
    return next
  })()
}

async function run() {
  const options = parseArgs(process.argv)
  const sourceRoot = path.resolve(options.source)
  const outRoot = path.resolve(options.out)

  const files = listDocs(sourceRoot)
  const slice = options.limit ? files.slice(0, options.limit) : files

  console.log(`Translating ${slice.length} files from ${options.from} → ${options.to}`)

  for (const file of slice) {
    const rel = path.relative(sourceRoot, file)
    const outFile = path.join(outRoot, rel)
    if (!options.force && fs.existsSync(outFile)) {
      console.log(`↷ ${outFile}`)
      continue
    }
    const raw = fs.readFileSync(file, 'utf8')
    const parsed = matter(raw)

    const translatedFrontmatter = await translateFrontmatter(parsed.data, options.from, options.to)

    const { text: maskedBlocks, placeholders: blockPlaceholders } = maskCodeBlocks(parsed.content)
    const { text: maskedInline, placeholders: inlinePlaceholders } = maskInlineCode(
      maskedBlocks,
      blockPlaceholders.length
    )

    const translatedBody = await translateText(maskedInline, options.from, options.to)
    const unmaskedBody = unmaskAll(translatedBody, [...blockPlaceholders, ...inlinePlaceholders])

    const output = matter.stringify(unmaskedBody, translatedFrontmatter)

    if (options.dryRun) {
      console.log(`[dry-run] ${outFile}`)
      continue
    }

    fs.mkdirSync(path.dirname(outFile), { recursive: true })
    fs.writeFileSync(outFile, output, 'utf8')
    console.log(`✓ ${outFile}`)
  }
}

run().catch((error) => {
  console.error(error)
  process.exit(1)
})
