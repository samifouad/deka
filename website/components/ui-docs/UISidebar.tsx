'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'

interface UISidebarProps {
  isOpen: boolean
}

const components = [
  { name: 'Avatar', slug: 'avatar' },
  { name: 'Button', slug: 'button' },
  { name: 'Card', slug: 'card' },
  { name: 'Checkbox', slug: 'checkbox' },
  { name: 'Command', slug: 'command' },
  { name: 'Dialog', slug: 'dialog' },
  { name: 'Input', slug: 'input' },
  { name: 'Label', slug: 'label' },
  { name: 'Popover', slug: 'popover' },
  { name: 'Separator', slug: 'separator' },
  { name: 'Sheet', slug: 'sheet' },
  { name: 'Skeleton', slug: 'skeleton' },
  { name: 'Switch', slug: 'switch' },
]

export function UISidebar({ isOpen }: UISidebarProps) {
  const pathname = usePathname()

  return (
    <aside
      className={`${
        isOpen ? 'block' : 'hidden'
      } md:block w-64 overflow-y-auto scrollbar-hide border-r border-border/30 mx-auto md:mx-0`}
    >
      <nav className="p-4 space-y-6">
        <div>
          <h3 className="text-sm font-semibold text-foreground mb-2 uppercase tracking-wide">
            Components
          </h3>
          <ul className="space-y-1">
            {components.map((component) => {
              const href = `/ui/${component.slug}`
              const isActive = pathname === href
              return (
                <li key={component.slug}>
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
                    {component.name}
                  </Link>
                </li>
              )
            })}
          </ul>
        </div>
      </nav>
    </aside>
  )
}
