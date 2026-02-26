import { notFound } from 'next/navigation'
import { getAPIDoc, getAllAPIDocs } from '@/lib/api'
import { APIContent } from '@/components/api-docs/APIContent'
import { DocBreadcrumbs } from '@/components/docs/DocBreadcrumbs'
import { DOC_VERSIONS, buildDocsPath, splitDocSlug } from '@/lib/docs-versions'
import { loadAPIDocs } from '@/lib/api-server'
import { getRequestLang } from '@/lib/i18n-server'

export async function generateStaticParams() {
  const slugs = getAllAPIDocs()
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
  const resolvedSlug = docSlug.length ? docSlug : ['intro']
  const docs = loadAPIDocs(await getRequestLang())
  const doc = await getAPIDoc(resolvedSlug, docs)

  if (!doc) {
    return {
      title: 'Page Not Found',
    }
  }

  return {
    title: `${doc.metadata.title} | Deka Deploy`,
    description: doc.metadata.description || `Deploy server documentation for ${doc.metadata.title}`,
  }
}

export default async function DeployDocPage({ params }: { params: Promise<{ slug: string[] }> }) {
  const { slug } = await params
  const { slug: docSlug, version } = splitDocSlug(slug)
  const resolvedSlug = docSlug.length ? docSlug : ['intro']
  const docs = loadAPIDocs(await getRequestLang())
  const doc = await getAPIDoc(resolvedSlug, docs)

  if (!doc) {
    notFound()
  }

  const breadcrumbItems = [
    { label: 'docs', href: '/docs' },
    { label: 'deploy', href: buildDocsPath('deploy', version) },
    ...docSlug.map((part, index) => ({
      label: part,
      href: index === docSlug.length - 1
        ? undefined
        : buildDocsPath('deploy', version, docSlug.slice(0, index + 1)),
    })),
  ]

  return (
    <div className="max-w-5xl mx-auto px-8 py-12">
      <article className="max-w-none">
        <DocBreadcrumbs items={breadcrumbItems} />
        <h1 className="text-4xl font-bold text-foreground mb-2">
          {doc.metadata.title}
        </h1>

        <APIContent html={doc.html} codeBlocks={doc.codeBlocks} />
      </article>
    </div>
  )
}
