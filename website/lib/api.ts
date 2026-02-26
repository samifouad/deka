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

export async function getAPIDoc(slug: string[], docs: APIDoc[] = apiDocs): Promise<APIDoc | null> {
  const doc = docs.find((d) =>
    d.slug.length === slug.length &&
    d.slug.every((s, i) => s === slug[i])
  )
  return doc || null
}

export function getAllAPIDocs(docs: APIDoc[] = apiDocs): string[][] {
  return docs.map((doc) => doc.slug)
}
