import bundledAPIDocs from './bundled-api.json'

export interface APIDoc {
  slug: string[]
  metadata: {
    title: string
    description?: string
  }
  content: string
  html: string
  codeBlocks: Array<{lang: string, code: string}>
}

const apiDocs = bundledAPIDocs as APIDoc[]

export async function getAPIDoc(slug: string[]): Promise<APIDoc | null> {
  const doc = apiDocs.find((d) =>
    d.slug.length === slug.length &&
    d.slug.every((s, i) => s === slug[i])
  )
  return doc || null
}

export function getAllAPIDocs(): string[][] {
  return apiDocs.map((doc) => doc.slug)
}
