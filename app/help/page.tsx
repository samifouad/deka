import { DocsHomeClient } from './DocsHomeClient'
import { generateSidebar } from '@/lib/docs'

// Force static generation for Cloudflare Workers
export const dynamic = 'force-static'

export default function DocsHome() {
  const sections = generateSidebar()

  return <DocsHomeClient sections={sections} />
}
