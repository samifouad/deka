import fs from 'fs'
import path from 'path'
import type { RuntimeDocFile } from '@/lib/runtime-docs'

const cache = new Map<string, RuntimeDocFile[]>()

function readBundle(filePath: string): RuntimeDocFile[] | null {
  if (!fs.existsSync(filePath)) return null
  const raw = fs.readFileSync(filePath, 'utf8')
  return JSON.parse(raw) as RuntimeDocFile[]
}

export function loadRuntimeDocs(lang?: string): RuntimeDocFile[] {
  const normalized = (lang || 'en').toLowerCase()
  const isDev = process.env.NODE_ENV !== 'production'
  if (!isDev && cache.has(normalized)) {
    return cache.get(normalized)!
  }

  const defaultPath = path.join(process.cwd(), 'lib/bundled-runtime.json')
  const localizedPath = normalized !== 'en'
    ? path.join(process.cwd(), `lib/bundled-runtime.${normalized}.json`)
    : null

  const baseDocs = readBundle(defaultPath) || []
  const localizedDocs = localizedPath ? readBundle(localizedPath) : null

  if (!localizedDocs) {
    if (!isDev) {
      cache.set(normalized, baseDocs)
    }
    return baseDocs
  }

  const merged = [...baseDocs]
  const indexByKey = new Map<string, number>()
  baseDocs.forEach((doc, index) => {
    const key = `${doc.section}/${doc.slug.join('/')}`
    indexByKey.set(key, index)
  })

  for (const doc of localizedDocs) {
    const key = `${doc.section}/${doc.slug.join('/')}`
    const index = indexByKey.get(key)
    if (index !== undefined) {
      merged[index] = doc
    } else {
      merged.push(doc)
    }
  }

  if (!isDev) {
    cache.set(normalized, merged)
  }
  return merged
}
