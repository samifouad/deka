'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useMemo, useState } from 'react'
import { getRuntimeSidebar, type RuntimeLanguage } from '@/lib/runtime-docs'
import { VersionSelect } from '@/components/docs/VersionSelect'
import { useDocsVersion } from '@/components/docs/useDocsVersion'
import { buildDocsPath } from '@/lib/docs-versions'
import { LanguageSelect } from '@/components/docs/LanguageSelect'

interface RuntimeSidebarProps {
  isOpen: boolean
  language: RuntimeLanguage
  basePath?: string
  sections?: ReturnType<typeof getRuntimeSidebar>
}

export function RuntimeSidebar({ isOpen, language, basePath, sections }: RuntimeSidebarProps) {
  const pathname = usePathname()
  const resolvedSections = sections ?? getRuntimeSidebar(language)
  const { version } = useDocsVersion()
  const fallbackBase = buildDocsPath(language, version)
  const resolvedBasePath = basePath
    ? (version && version !== 'latest' ? `${basePath}/${version}` : basePath)
    : fallbackBase
  const defaultOpen = useMemo(() => {
    const open = new Set<string>()
    for (const section of resolvedSections) {
      for (const item of section.items) {
        const href = `${resolvedBasePath}/${item.slug}`
        if (pathname === href) {
          open.add(section.categorySlug)
          break
        }
      }
    }
    if (open.size === 0 && resolvedSections[0]) {
      open.add(resolvedSections[0].categorySlug)
    }
    return open
  }, [pathname, resolvedSections, resolvedBasePath])
  const [openSections, setOpenSections] = useState<Set<string>>(() => defaultOpen)

  const toggleSection = (slug: string) => {
    setOpenSections((prev) => {
      const next = new Set(prev)
      if (next.has(slug)) {
        next.delete(slug)
      } else {
        next.add(slug)
      }
      return next
    })
  }

  return (
    <aside
      className={`${
        isOpen ? 'block' : 'hidden'
      } md:block w-64 overflow-y-auto scrollbar-hide border-r border-border/30 mx-auto md:mx-0 relative z-30`}
    >
      <nav className="p-4 space-y-6">
        <VersionSelect />
        {resolvedSections.map((section) => {
          const isSectionOpen = openSections.has(section.categorySlug)
          return (
          <div key={section.categorySlug}>
            <button
              type="button"
              onClick={() => toggleSection(section.categorySlug)}
              className="w-full text-sm font-semibold text-foreground mb-2 uppercase tracking-wide flex items-center justify-between hover:text-primary"
              aria-expanded={isSectionOpen}
              aria-controls={`runtime-section-${section.categorySlug}`}
            >
              <span>{section.category}</span>
              <span className={`transition-transform ${isSectionOpen ? 'rotate-90' : ''}`}>›</span>
            </button>
            {isSectionOpen ? (
              <ul id={`runtime-section-${section.categorySlug}`} className="space-y-1">
                {section.items.map((item) => {
                  const href = `${resolvedBasePath}/${item.slug}`
                  const isActive = pathname === href
                  return (
                    <li key={item.slug}>
                      <Link
                        href={href}
                        prefetch={true}
                        className={`block px-3 py-1 rounded-lg text-sm transition-colors truncate ${
                          isActive
                            ? 'bg-primary/10 text-primary font-medium'
                            : 'text-muted-foreground hover:bg-secondary/50 hover:text-foreground'
                        }`}
                      >
                        <span className="block truncate">{item.name}</span>
                      </Link>
                    </li>
                  )
                })}
              </ul>
            ) : null}
          </div>
          )
        })}
        <div className="pt-4 border-t border-border/30">
          <LanguageSelect />
        </div>
      </nav>
    </aside>
  )
}
