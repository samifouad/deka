'use client'

import { useEffect, useState } from 'react'
import { TableOfContentsItem } from '@/lib/docs'

interface TableOfContentsProps {
  headings: TableOfContentsItem[]
}

export function TableOfContents({ headings }: TableOfContentsProps) {
  const [activeId, setActiveId] = useState<string>('')

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            setActiveId(entry.target.id)
          }
        })
      },
      { rootMargin: '-100px 0px -66% 0px' }
    )

    headings.forEach(({ id }) => {
      const element = document.getElementById(id)
      if (element) observer.observe(element)
    })

    return () => observer.disconnect()
  }, [headings])

  if (headings.length === 0) return null

  return (
    <aside className="hidden xl:block w-64 overflow-y-auto">
      <div className="sticky top-20 p-4">
        <h4 className="text-sm font-semibold text-foreground mb-3 uppercase tracking-wide">
          On This Page
        </h4>
        <nav>
          <ul className="space-y-2">
            {headings.map(({ id, text, level }) => (
              <li
                key={id}
                className={level === 3 ? 'ml-4' : ''}
              >
                <a
                  href={`#${id}`}
                  className={`block text-sm transition-colors ${
                    activeId === id
                      ? 'text-primary font-semibold'
                      : 'text-muted-foreground hover:text-foreground'
                  }`}
                  onClick={(e) => {
                    e.preventDefault()
                    document.getElementById(id)?.scrollIntoView({
                      behavior: 'smooth',
                      block: 'start'
                    })
                  }}
                >
                  {text}
                </a>
              </li>
            ))}
          </ul>
        </nav>
      </div>
    </aside>
  )
}
