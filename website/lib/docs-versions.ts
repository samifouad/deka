export const DOC_VERSIONS = [
  { id: 'latest', label: 'latest' },
  { id: 'v1.0', label: 'v1.0' },
  { id: 'v0.9', label: 'v0.9' },
]

export const DEFAULT_DOC_VERSION = DOC_VERSIONS[0].id

export function isDocsVersion(value?: string | null) {
  if (!value) return false
  return DOC_VERSIONS.some((version) => version.id === value)
}

export function parseDocsPath(pathname: string) {
  const parts = pathname.split('/').filter(Boolean)
  if (parts[0] !== 'docs' || parts.length < 2) return null

  const section = parts[1]
  const maybeVersion = parts[2]
  if (isDocsVersion(maybeVersion)) {
    return {
      section,
      version: maybeVersion,
      rest: parts.slice(3),
    }
  }

  return {
    section,
    version: DEFAULT_DOC_VERSION,
    rest: parts.slice(2),
  }
}

export function buildDocsPath(section: string, version: string, rest: string[] = []) {
  const parts = ['', 'docs', section]
  if (version && version !== DEFAULT_DOC_VERSION) {
    parts.push(version)
  }
  parts.push(...rest.filter(Boolean))
  return parts.join('/')
}

export function splitDocSlug(slug: string[]) {
  if (!slug.length) {
    return { version: DEFAULT_DOC_VERSION, slug: [] }
  }

  if (isDocsVersion(slug[0])) {
    return { version: slug[0], slug: slug.slice(1) }
  }

  return { version: DEFAULT_DOC_VERSION, slug }
}
