'use client'

import Link from 'next/link'

interface BreadcrumbItem {
  label: string
  href?: string
  linkCurrent?: boolean
}

export function DocBreadcrumbs({ items }: { items: BreadcrumbItem[] }) {
  if (!items.length) return null

  return (
    <nav aria-label="Breadcrumb" className="mb-6 text-sm text-muted-foreground">
      <ol className="flex flex-wrap items-center gap-2">
        {items.map((item, index) => {
          const isLast = index === items.length - 1
          return (
            <li key={`${item.label}-${index}`} className="flex items-center gap-2">
              {item.href && (!isLast || item.linkCurrent) ? (
                <Link href={item.href} className="hover:text-foreground">
                  {item.label}
                </Link>
              ) : (
                <span className={isLast ? 'text-foreground' : undefined}>{item.label}</span>
              )}
              {!isLast && <span className="text-muted-foreground">{'>'}</span>}
            </li>
          )
        })}
      </ol>
    </nav>
  )
}
