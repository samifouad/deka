// Import pre-bundled docs for Cloudflare Workers compatibility
import bundledDocsData from './bundled-docs.json'

export interface DocMetadata {
  title: string
  description?: string
  section?: string
  category?: string
  categoryLabel?: string
  categoryOrder?: number
  sidebar?: {
    order?: number
  }
}

export interface DocFile {
  slug: string[]
  metadata: DocMetadata
  content: string
  html: string // Pre-compiled HTML from build time
  codeBlocks: Array<{lang: string, code: string}> // Extracted code blocks for Monaco
  tableOfContents: TableOfContentsItem[] // Pre-computed TOC from build time
}

export interface SidebarSection {
  label: string
  items: SidebarItem[]
}

export interface SidebarItem {
  label: string
  href: string
  items?: SidebarItem[]
  order?: number
}

export interface TableOfContentsItem {
  id: string
  text: string
  level: number
}

// Get all doc files from pre-bundled JSON
export function getAllDocs(): DocFile[] {
  return bundledDocsData as DocFile[]
}

// Get a single doc by slug
export function getDoc(slug: string[]): DocFile | null {
  const allDocs = getAllDocs()
  return allDocs.find(doc =>
    JSON.stringify(doc.slug) === JSON.stringify(slug)
  ) || null
}

// Generate sidebar structure from file system
export function generateSidebar(): SidebarSection[] {
  const allDocs = getAllDocs()

  const categoryMap = new Map<string, SidebarItem[]>()
  const categoryLabels = new Map<string, string>()
  const categoryOrder = new Map<string, number>()

  for (const doc of allDocs) {
    if (doc.slug.length === 0) continue // Skip root index

    const section = doc.metadata.section || doc.slug[0] || 'docs'
    const categorySlug = doc.metadata.category || doc.slug[1] || 'general'
    const categoryKey = `${section}::${categorySlug}`
    const sectionLabel = formatSectionLabel(section)
    const categoryLabel = doc.metadata.categoryLabel || formatCategoryLabel(categorySlug)
    const labelPrefix = sectionLabel ? `${sectionLabel} · ` : ''
    const categoryDisplay = `${labelPrefix}${categoryLabel}`
    const href = `/docs/${doc.slug.join('/')}`
    const label = doc.metadata.title || doc.slug[doc.slug.length - 1]

    if (!categoryMap.has(categoryKey)) {
      categoryMap.set(categoryKey, [])
    }

    categoryLabels.set(categoryKey, categoryDisplay)
    if (typeof doc.metadata.categoryOrder === 'number') {
      const existing = categoryOrder.get(categoryKey)
      if (existing === undefined || doc.metadata.categoryOrder < existing) {
        categoryOrder.set(categoryKey, doc.metadata.categoryOrder)
      }
    }

    categoryMap.get(categoryKey)!.push({
      label,
      href,
      order: doc.metadata.sidebar?.order,
    })
  }

  type SectionEntry = SidebarSection & { categoryKey: string }
  const sections: SectionEntry[] = Array.from(categoryMap.entries()).map(([categoryKey, items]) => {
    return {
      label: categoryLabels.get(categoryKey) || categoryKey,
      items,
      categoryKey,
    }
  })

  // Sort items within each section - prioritize intro/overview
  for (const section of sections) {
    section.items.sort((a, b) => {
      const aOrder = (a as SidebarItem & { order?: number }).order
      const bOrder = (b as SidebarItem & { order?: number }).order
      if (aOrder !== undefined || bOrder !== undefined) {
        const safeA = aOrder ?? 999
        const safeB = bOrder ?? 999
        if (safeA !== safeB) return safeA - safeB
      }
      const aLabel = a.label.toLowerCase()
      const bLabel = b.label.toLowerCase()

      // Intro/Overview pages first
      if (aLabel.includes('intro') || aLabel.includes('overview')) return -1
      if (bLabel.includes('intro') || bLabel.includes('overview')) return 1

      // Then alphabetically
      return a.label.localeCompare(b.label)
    })
  }

  sections.sort((a, b) => {
    const orderA = categoryOrder.get(a.categoryKey)
    const orderB = categoryOrder.get(b.categoryKey)
    if (orderA !== undefined || orderB !== undefined) {
      const safeA = orderA ?? 999
      const safeB = orderB ?? 999
      if (safeA !== safeB) return safeA - safeB
    }
    return a.label.localeCompare(b.label)
  })

  return sections
}

function formatCategoryLabel(value: string): string {
  if (!value) return 'General'
  return value
    .replace(/[-_]+/g, ' ')
    .replace(/\b\w/g, (char) => char.toUpperCase())
}

function formatSectionLabel(value: string): string {
  if (!value) return ''
  const lowered = value.toLowerCase()
  if (lowered === 'phpx') return 'PHPX'
  if (lowered === 'php') return 'PHP'
  if (lowered === 'js') return 'JS'
  return formatCategoryLabel(value)
}

// Get table of contents for a doc (pre-computed at build time)
export function getTableOfContents(doc: DocFile): TableOfContentsItem[] {
  return doc.tableOfContents
}
