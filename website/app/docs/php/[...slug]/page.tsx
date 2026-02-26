import { notFound } from 'next/navigation'
import Link from 'next/link'
import { CLIContent } from '@/components/cli-docs/CLIContent'
import { DocBreadcrumbs } from '@/components/docs/DocBreadcrumbs'
import { DOC_VERSIONS, buildDocsPath, splitDocSlug } from '@/lib/docs-versions'
import { getRuntimeCategoryItems } from '@/lib/runtime-docs'
import { getAllRuntimeDocSlugs, getRuntimeDoc } from '@/lib/runtime-docs'
import { loadRuntimeDocs } from '@/lib/runtime-docs-server'
import { getRequestLang } from '@/lib/i18n-server'

export async function generateStaticParams() {
  const slugs = getAllRuntimeDocSlugs('php')
  const params = slugs.map((slug) => ({ slug }))

  for (const version of DOC_VERSIONS) {
    if (version.id === 'latest') continue
    params.push({ slug: [version.id] })
    for (const slug of slugs) {
      params.push({ slug: [version.id, ...slug] })
    }
  }

  for (const slug of slugs) {
    if (slug.length === 2 && slug[1] === 'index') {
      params.push({ slug: [slug[0]] })
      for (const version of DOC_VERSIONS) {
        if (version.id === 'latest') continue
        params.push({ slug: [version.id, slug[0]] })
      }
    }
  }

  return params
}

export async function generateMetadata({ params }: { params: Promise<{ slug: string[] }> }) {
  const { slug } = await params
  const { slug: docSlug } = splitDocSlug(slug)
  const resolvedSlug = docSlug.length ? docSlug : ['overview']
  const docs = loadRuntimeDocs(await getRequestLang())
  const doc = getRuntimeDoc('php', resolvedSlug, docs) || (resolvedSlug.length === 1
    ? getRuntimeDoc('php', [resolvedSlug[0], 'index'], docs)
    : null)

  if (!doc) {
    return {
      title: 'Page Not Found',
    }
  }

  return {
    title: `${doc.metadata.title} | Deka PHP Runtime`,
    description: doc.metadata.description,
  }
}

export default async function PHPRuntimeDocPage({ params }: { params: Promise<{ slug: string[] }> }) {
  const { slug } = await params
  const { slug: docSlug, version } = splitDocSlug(slug)
  const resolvedSlug = docSlug.length ? docSlug : ['overview']
  const docs = loadRuntimeDocs(await getRequestLang())
  const doc = getRuntimeDoc('php', resolvedSlug, docs) || (resolvedSlug.length === 1
    ? getRuntimeDoc('php', [resolvedSlug[0], 'index'], docs)
    : null)

  if (!doc) {
    notFound()
  }

  const leafLabel = doc.metadata.docid?.split('/').pop() || doc.metadata.title
  const breadcrumbItems = [
    { label: 'docs', href: '/docs' },
    { label: 'php', href: buildDocsPath('php', version) },
  ]

  if (docSlug.length > 1) {
    breadcrumbItems.push({ label: docSlug[0], href: buildDocsPath('php', version, [docSlug[0]]) })
  } else if (docSlug.length === 1) {
    breadcrumbItems.push({
      label: docSlug[0],
      href: buildDocsPath('php', version, [docSlug[0]]),
      linkCurrent: true,
    })
  }

  if (docSlug.length > 1) {
    breadcrumbItems.push({ label: leafLabel })
  }

  const isCategoryPage = doc.slug.length === 2 && doc.slug[1] === 'index'
  const categoryItems = isCategoryPage
    ? getRuntimeCategoryItems('php', docSlug[0], docs)
    : []

  return (
    <div className="max-w-5xl mx-auto px-8 py-12">
      <article className="max-w-none">
        <DocBreadcrumbs items={breadcrumbItems} />
        <h1 className="text-4xl font-bold text-foreground mb-2">
          {doc.metadata.title}
        </h1>

        <CLIContent html={doc.html} codeBlocks={doc.codeBlocks} />

        {categoryItems.length > 0 && (
          <div className="mt-12 border-t border-border/40 pt-8">
            <h2 className="text-2xl font-semibold text-foreground mb-4">In this section</h2>
            <div className="space-y-3">
              {categoryItems.map((item) => (
                <Link
                  key={item.slug}
                  href={buildDocsPath('php', version, item.slug.split('/'))}
                  className="block rounded-lg border border-border/60 bg-background/70 px-4 py-3 hover:border-primary/60 hover:text-foreground"
                >
                  <div className="font-medium text-foreground">{item.title}</div>
                </Link>
              ))}
            </div>
          </div>
        )}
      </article>
    </div>
  )
}
