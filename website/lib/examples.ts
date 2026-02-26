// Import pre-bundled examples for Cloudflare Workers compatibility
import bundledExamples from './bundled-examples.json'

interface CodeExample {
  id: string
  title: string
  description: string
  code: string
  result: any
  category: string
}

export async function loadCodeExamples(): Promise<CodeExample[]> {
  return bundledExamples as CodeExample[]
}