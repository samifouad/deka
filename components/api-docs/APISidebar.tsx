'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'

interface APISidebarProps {
  isOpen: boolean
}

const apiSections = [
  {
    category: 'Overview',
    items: [
      { name: 'Introduction', slug: 'intro', description: 'API overview and authentication' },
      { name: 'Rate Limiting', slug: 'rate-limit', description: 'API usage limits' },
    ]
  },
  {
    category: 'Authentication',
    items: [
      { name: 'Identity Service', slug: 'identity/index', description: 'QR code auth API' },
    ]
  },
  {
    category: 'Blockchain API',
    items: [
      { name: 'Users', slug: 'users/create', description: 'User management' },
      { name: 'Balances', slug: 'balances/index', description: 'Balance queries' },
      { name: 'Transactions', slug: 'transactions/index', description: 'Transaction management' },
      { name: 'Blocks', slug: 'blocks/index', description: 'Block queries' },
      { name: 'Contracts', slug: 'contracts/index', description: 'Smart contracts' },
    ]
  },
  {
    category: 'Developer Tools',
    items: [
      { name: 'Keys', slug: 'keys/index', description: 'Cryptographic keys' },
      { name: 'Modules', slug: 'modules/kv', description: 'Storage modules' },
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
                      <div>{item.name}</div>
                      <div className="text-xs text-muted-foreground mt-0.5">{item.description}</div>
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
