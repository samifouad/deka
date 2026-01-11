'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { SidebarSection } from '@/lib/docs'

interface DocsSidebarProps {
  sections: SidebarSection[]
  isOpen: boolean
}

export function DocsSidebar({ sections, isOpen }: DocsSidebarProps) {
  const pathname = usePathname()

  return (
    <aside
      className={`${
        isOpen ? 'block' : 'hidden'
      } md:block w-64 overflow-y-auto scrollbar-hide border-r border-border/30 mx-auto md:mx-0`}
    >
      <nav className="p-4 space-y-6">
        {sections.map((section, index) => (
          <div key={index}>
            <h3 className="text-sm font-semibold text-foreground mb-2 uppercase tracking-wide">
              {section.label}
            </h3>
            <ul className="space-y-1">
              {section.items.map((item, itemIndex) => {
                const isActive = pathname === item.href
                return (
                  <li key={itemIndex}>
                    <Link
                      href={item.href}
                      prefetch={true}
                      scroll={false}
                      className={`block px-3 py-2 rounded-lg text-sm transition-colors ${
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
          </div>
        ))}
      </nav>
    </aside>
  )
}
