import { notFound } from 'next/navigation'
import { getCLIDoc, getAllCLIDocs } from '@/lib/cli'
import { CLIContent } from '@/components/cli-docs/CLIContent'
import { CLIOverview } from '@/components/cli-docs/CLIOverview'
import { DocBreadcrumbs } from '@/components/docs/DocBreadcrumbs'
import { DOC_VERSIONS, buildDocsPath, splitDocSlug } from '@/lib/docs-versions'
import { loadCLIDocs } from '@/lib/cli-server'
import { getRequestLang } from '@/lib/i18n-server'

export async function generateStaticParams() {
  const slugs = getAllCLIDocs()
  const params = slugs.map((slug) => ({ slug }))

  for (const version of DOC_VERSIONS) {
    if (version.id === 'latest') continue
    params.push({ slug: [version.id] })
    for (const slug of slugs) {
      params.push({ slug: [version.id, ...slug] })
    }
  }

  return params
}

export async function generateMetadata({ params }: { params: Promise<{ slug: string[] }> }) {
  const { slug } = await params
  const { slug: docSlug } = splitDocSlug(slug)

  if (docSlug.length === 0 || (docSlug.length === 1 && docSlug[0] === 'overview')) {
    return {
      title: 'Deka CLI',
      description: 'Command-line tools to deploy, manage, and scale your own infrastructure.',
    }
  }

  const docs = loadCLIDocs(await getRequestLang())
  const doc = await getCLIDoc(docSlug, docs)

  if (!doc) {
    return {
      title: 'Page Not Found',
    }
  }

  return {
    title: `${doc.metadata.title} | Deka CLI`,
    description: doc.metadata.description || `CLI documentation for ${doc.metadata.title}`,
  }
}

export default async function DocsCLIDocPage({ params }: { params: Promise<{ slug: string[] }> }) {
  const { slug } = await params
  const { slug: docSlug, version } = splitDocSlug(slug)

  if (docSlug.length === 0 || (docSlug.length === 1 && docSlug[0] === 'overview')) {
    return <CLIOverview />
  }

  const docs = loadCLIDocs(await getRequestLang())
  const doc = await getCLIDoc(docSlug, docs)

  if (!doc) {
    notFound()
  }

  const breadcrumbItems = [
    { label: 'docs', href: '/docs' },
    { label: 'cli', href: buildDocsPath('cli', version) },
    ...docSlug.map((part, index) => ({
      label: part,
      href: index === docSlug.length - 1
        ? undefined
        : buildDocsPath('cli', version, docSlug.slice(0, index + 1)),
    })),
  ]

  return (
    <div className="max-w-5xl mx-auto px-8 py-12">
      <article className="max-w-none">
        <DocBreadcrumbs items={breadcrumbItems} />
        <h1 className="text-4xl font-bold text-foreground mb-2">
          {doc.metadata.title}
        </h1>

        <CLIContent html={doc.html} codeBlocks={doc.codeBlocks} />
      </article>
    </div>
  )
}
