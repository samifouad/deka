import fs from 'fs'
import path from 'path'
import type { APIDoc } from '@/lib/api'

const cache = new Map<string, APIDoc[]>()

function readBundle(filePath: string): APIDoc[] | null {
  if (!fs.existsSync(filePath)) return null
  const raw = fs.readFileSync(filePath, 'utf8')
  return JSON.parse(raw) as APIDoc[]
}

export function loadAPIDocs(lang?: string): APIDoc[] {
  const normalized = (lang || 'en').toLowerCase()
  if (cache.has(normalized)) {
    return cache.get(normalized)!
  }

  const defaultPath = path.join(process.cwd(), 'lib/bundled-api.json')
  const localizedPath = normalized !== 'en'
    ? path.join(process.cwd(), `lib/bundled-api.${normalized}.json`)
    : null

  const baseDocs = readBundle(defaultPath) || []
  const localizedDocs = localizedPath ? readBundle(localizedPath) : null

  if (!localizedDocs) {
    cache.set(normalized, baseDocs)
    return baseDocs
  }

  const merged = [...baseDocs]
  const indexByKey = new Map<string, number>()
  baseDocs.forEach((doc, index) => {
    const key = doc.slug.join('/')
    indexByKey.set(key, index)
  })

  for (const doc of localizedDocs) {
    const key = doc.slug.join('/')
    const index = indexByKey.get(key)
    if (index !== undefined) {
      merged[index] = doc
    } else {
      merged.push(doc)
    }
  }

  cache.set(normalized, merged)
  return merged
}
