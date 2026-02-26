'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'

interface APISidebarProps {
  isOpen: boolean
}

const apiSections = [
  {
    category: 'Getting Started',
    items: [
      { name: 'Overview', slug: 'overview' },
      { name: 'Authentication', slug: 'authentication' },
    ]
  },
  {
    category: 'Identity API',
    items: [
      { name: 'Sessions', slug: 'identity/sessions' },
      { name: 'QR Authentication', slug: 'identity/qr-auth' },
    ]
  },
  {
    category: 'Ledger API',
    items: [
      { name: 'Users', slug: 'users' },
      { name: 'Balances', slug: 'balances' },
      { name: 'Transactions', slug: 'transactions' },
      { name: 'Blocks', slug: 'blocks' },
      { name: 'Contracts', slug: 'contracts' },
    ]
  }
]

export function APISidebar({ isOpen }: APISidebarProps) {
  const pathname = usePathname()

  return (
    <aside
      className={`${
        isOpen ? 'block' : 'hidden'
      } md:block w-64 overflow-y-auto scrollbar-hide border-r border-border/30 mx-auto md:mx-0`}
    >
      <nav className="p-4 space-y-6">
        {apiSections.map((section, sectionIndex) => (
          <div key={sectionIndex}>
            <h3 className="text-sm font-semibold text-foreground mb-2 uppercase tracking-wide">
              {section.category}
            </h3>
            <ul className="space-y-1">
              {section.items.map((item) => {
                const href = `/api/${item.slug}`
                const isActive = pathname === href
                return (
                  <li key={item.slug}>
                    <Link
                      href={href}
                      prefetch={true}
                      scroll={false}
                      className={`block px-3 py-2 rounded-lg text-sm transition-colors ${
                        isActive
                          ? 'bg-primary/10 text-primary font-medium'
                          : 'text-muted-foreground hover:bg-secondary/50 hover:text-foreground'
                      }`}
                    >
                      {item.name}
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
