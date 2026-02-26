'use client'

import { useRouter } from 'next/navigation'
import { DOC_VERSIONS, buildDocsPath } from '@/lib/docs-versions'
import { useDocsVersion } from '@/components/docs/useDocsVersion'

interface VersionSelectProps {
  className?: string
}

export function VersionSelect({ className }: VersionSelectProps) {
  const router = useRouter()
  const { version, parsed, updateVersion } = useDocsVersion()

  const handleChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    const next = event.target.value
    updateVersion(next)

    if (!parsed) {
      return
    }

    const nextPath = buildDocsPath(parsed.section, next, parsed.rest)
    router.push(nextPath)
  }

  return (
    <div className={className}>
      <div className="text-xs uppercase tracking-wide text-muted-foreground mb-2">version</div>
      <div className="relative">
        <select
          aria-label="Select documentation version"
          value={version}
          onChange={handleChange}
          className="w-full appearance-none rounded-lg border border-border/60 bg-background/80 px-3 py-2 text-sm text-foreground shadow-sm focus:border-primary focus:outline-none"
        >
          {DOC_VERSIONS.map((item) => (
            <option key={item.id} value={item.id}>
              {item.label}
            </option>
          ))}
        </select>
        <span className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-xs text-muted-foreground">▼</span>
      </div>
    </div>
  )
}
