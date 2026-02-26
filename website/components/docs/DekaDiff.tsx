import type { ReactNode } from 'react'

interface DekaDiffProps {
  title?: string
  children: ReactNode
}

export function DekaDiff({ title = 'Deka differences', children }: DekaDiffProps) {
  return (
    <div className="my-6 rounded-xl border border-amber-300/60 bg-amber-50/70 p-4 text-sm text-amber-950 shadow-sm dark:border-amber-400/30 dark:bg-amber-500/10 dark:text-amber-100">
      <div className="text-xs font-semibold uppercase tracking-wide text-amber-700 dark:text-amber-200">
        {title}
      </div>
      <div className="mt-2 text-sm text-amber-900 dark:text-amber-100">{children}</div>
    </div>
  )
}
