import bundledCLIDocs from './bundled-cli.json'

export interface CLIDoc {
  slug: string[]
  metadata: {
    title: string
    description?: string
  }
  content: string
  html: string
  codeBlocks: Array<{lang: string, code: string}>
}

const cliDocs = bundledCLIDocs as CLIDoc[]

export async function getCLIDoc(slug: string[], docs: CLIDoc[] = cliDocs): Promise<CLIDoc | null> {
  const doc = docs.find((d) =>
    d.slug.length === slug.length &&
    d.slug.every((s, i) => s === slug[i])
  )
  return doc || null
}

export function getAllCLIDocs(docs: CLIDoc[] = cliDocs): string[][] {
  return docs.map((doc) => doc.slug)
}
