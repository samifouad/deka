#!/usr/bin/env bun
/**
 * Translate and bundle docs for all supported languages.
 *
 * Usage:
 *   bun scripts/build-i18n.ts
 *   bun scripts/build-i18n.ts --force
 */

import fs from 'fs'
import path from 'path'
import { languages } from '../i18n/i18n'

type TranslateTarget = {
  name: string
  source: string
  out: (lang: string) => string
  bundle: string
}

const targets: TranslateTarget[] = [
  {
    name: 'runtime docs',
    source: 'content/docs',
    out: (lang) => `content-i18n/${lang}/docs`,
    bundle: 'scripts/bundle-runtime.ts',
  },
  {
    name: 'cli docs',
    source: 'content/cli',
    out: (lang) => `content-i18n/${lang}/cli`,
    bundle: 'scripts/bundle-cli.ts',
  },
  {
    name: 'deploy/api docs',
    source: 'content/api',
    out: (lang) => `content-i18n/${lang}/api`,
    bundle: 'scripts/bundle-api.ts',
  },
]

function loadEnv() {
  const envPath = path.join(process.cwd(), '.env')
  if (!fs.existsSync(envPath)) return

  const raw = fs.readFileSync(envPath, 'utf8')
  for (const line of raw.split(/\r?\n/)) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#')) continue
    const idx = trimmed.indexOf('=')
    if (idx === -1) continue
    const key = trimmed.slice(0, idx).trim()
    let value = trimmed.slice(idx + 1).trim()
    if (
      (value.startsWith('"') && value.endsWith('"')) ||
      (value.startsWith("'") && value.endsWith("'"))
    ) {
      value = value.slice(1, -1)
    }
    if (!process.env[key]) {
      process.env[key] = value
    }
  }
}

function parseArgs(argv: string[]) {
  return {
    force: argv.includes('--force'),
  }
}

async function run(cmd: string, args: string[]) {
  const proc = Bun.spawn({
    cmd: [cmd, ...args],
    stdout: 'inherit',
    stderr: 'inherit',
  })
  const exitCode = await proc.exited
  if (exitCode !== 0) {
    throw new Error(`${cmd} ${args.join(' ')} exited with ${exitCode}`)
  }
}

async function translateAndBundle(lang: string, force: boolean) {
  for (const target of targets) {
    const outDir = target.out(lang)
    const translateArgs = [
      'scripts/translate-docs.ts',
      '--source',
      target.source,
      '--out',
      outDir,
      '--from',
      'en',
      '--to',
      lang,
    ]
    if (force) {
      translateArgs.push('--force')
    }

    console.log(`\n🌍 ${lang}: translating ${target.name}`)
    await run('bun', translateArgs)

    console.log(`📦 ${lang}: bundling ${target.name}`)
    await run('bun', [
      target.bundle,
      '--source',
      outDir,
      '--lang',
      lang,
    ])
  }
}

async function main() {
  const { force } = parseArgs(process.argv)
  loadEnv()

  if (!process.env.OPENAI_API_KEY) {
    throw new Error('OPENAI_API_KEY is required to build translations.')
  }

  const targetLangs = languages
    .map((lang) => lang.code.toLowerCase())
    .filter((code) => code !== 'en')

  for (const lang of targetLangs) {
    await translateAndBundle(lang, force)
  }
}

main().catch((error) => {
  console.error(error)
  process.exit(1)
})
