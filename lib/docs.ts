// Import pre-bundled docs for Cloudflare Workers compatibility
import bundledDocsData from './bundled-docs.json'

export interface DocMetadata {
  title: string
  description?: string
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

  // Create a flat structure organized by top-level category
  const categoryMap = new Map<string, SidebarItem[]>()

  for (const doc of allDocs) {
    if (doc.slug.length === 0) continue // Skip root index

    const category = doc.slug[0]
    const href = `/docs/${doc.slug.join('/')}`
    const label = doc.metadata.title || doc.slug[doc.slug.length - 1]

    if (!categoryMap.has(category)) {
      categoryMap.set(category, [])
    }

    categoryMap.get(category)!.push({ label, href })
  }

  // Map categories to sections with proper labels
  const sections: SidebarSection[] = []

  // ============================================
  // USER SECTION - Using Tana as an end user
  // ============================================
  if (categoryMap.has('guides')) {
    sections.push({
      label: 'Guides',
      items: categoryMap.get('guides')!
    })
  }

  if (categoryMap.has('tana-app')) {
    sections.push({
      label: 'Mobile App',
      items: categoryMap.get('tana-app')!
    })
  }

  // ============================================
  // SOVEREIGN SECTION - Running your own network
  // ============================================
  if (categoryMap.has('sovereign')) {
    sections.push({
      label: 'Sovereign',
      items: categoryMap.get('sovereign')!
    })
  }

  if (categoryMap.has('tana-edge')) {
    sections.push({
      label: 'Edge Server',
      items: categoryMap.get('tana-edge')!
    })
  }

  // ============================================
  // DEVELOPER SECTION - Contributing to Tana
  // ============================================
  if (categoryMap.has('contributing')) {
    sections.push({
      label: 'Developer',
      items: categoryMap.get('contributing')!
    })
  }

  // Sort items within each section - prioritize intro/overview
  for (const section of sections) {
    section.items.sort((a, b) => {
      const aLabel = a.label.toLowerCase()
      const bLabel = b.label.toLowerCase()

      // Intro/Overview pages first
      if (aLabel.includes('intro') || aLabel.includes('overview')) return -1
      if (bLabel.includes('intro') || bLabel.includes('overview')) return 1

      // Then alphabetically
      return a.label.localeCompare(b.label)
    })
  }

  return sections
}

// Get table of contents for a doc (pre-computed at build time)
export function getTableOfContents(doc: DocFile): TableOfContentsItem[] {
  return doc.tableOfContents
}
