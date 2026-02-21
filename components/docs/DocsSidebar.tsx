'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useMemo, useState } from 'react'
import { SidebarSection } from '@/lib/docs'

interface DocsSidebarProps {
  sections: SidebarSection[]
  isOpen: boolean
}

export function DocsSidebar({ sections, isOpen }: DocsSidebarProps) {
  const pathname = usePathname()
  const defaultOpen = useMemo(() => {
    const open = new Set<number>()
    sections.forEach((section, index) => {
      for (const item of section.items) {
        if (pathname === item.href) {
          open.add(index)
          break
        }
      }
    })
    if (open.size === 0 && sections.length > 0) {
      open.add(0)
    }
    return open
  }, [pathname, sections])
  const [openSections, setOpenSections] = useState<Set<number>>(() => defaultOpen)

  const toggleSection = (index: number) => {
    setOpenSections((prev) => {
      const next = new Set(prev)
      if (next.has(index)) {
        next.delete(index)
      } else {
        next.add(index)
      }
      return next
    })
  }

  return (
    <aside
      className={`${
        isOpen ? 'block' : 'hidden'
      } md:block w-64 overflow-y-auto scrollbar-hide border-r border-border/30 mx-auto md:mx-0`}
    >
      <nav className="p-4 space-y-6">
        {sections.map((section, index) => {
          const isSectionOpen = openSections.has(index)
          return (
            <div key={index}>
              <button
                type="button"
                onClick={() => toggleSection(index)}
                className="w-full text-sm font-semibold text-foreground mb-2 uppercase tracking-wide flex items-center justify-between hover:text-primary"
                aria-expanded={isSectionOpen}
                aria-controls={`docs-section-${index}`}
              >
                <span>{section.label}</span>
                <span className={`transition-transform transform ${isSectionOpen ? 'rotate-90' : 'rotate-0'}`}>›</span>
              </button>
              {isSectionOpen ? (
                <ul id={`docs-section-${index}`} className="space-y-1">
                  {section.items.map((item, itemIndex) => {
                    const isActive = pathname === item.href
                    return (
                      <li key={itemIndex}>
                        <Link
                          href={item.href}
                          prefetch={true}
                          scroll={false}
                          className={`block px-3 py-1 rounded-lg text-sm transition-colors ${
                            isActive
                              ? 'bg-primary/10 text-primary font-medium'
                              : 'text-muted-foreground hover:bg-secondary/50 hover:text-foreground'
                          }`}
                        >
                          {item.label}
                        </Link>
                      </li>
                    )
                  })}
                </ul>
              ) : null}
            </div>
          )
        })}
      </nav>
    </aside>
  )
}
