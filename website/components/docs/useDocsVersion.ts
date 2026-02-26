'use client'

import { useEffect, useMemo, useState } from 'react'
import { DEFAULT_DOC_VERSION, isDocsVersion, parseDocsPath } from '@/lib/docs-versions'
import { usePathname } from 'next/navigation'

const STORAGE_KEY = 'docs-version'

export function useDocsVersion() {
  const pathname = usePathname() || ''
  const parsed = useMemo(() => parseDocsPath(pathname), [pathname])
  const [version, setVersion] = useState(DEFAULT_DOC_VERSION)

  useEffect(() => {
    if (typeof window === 'undefined') return

    if (parsed?.version) {
      setVersion(parsed.version)
      localStorage.setItem(STORAGE_KEY, parsed.version)
      return
    }

    const stored = localStorage.getItem(STORAGE_KEY)
    if (isDocsVersion(stored)) {
      setVersion(stored)
    }
  }, [parsed?.version, pathname])

  const updateVersion = (next: string) => {
    setVersion(next)
    if (typeof window !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, next)
    }
  }

  return { version, parsed, updateVersion }
}
