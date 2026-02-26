import { notFound } from 'next/navigation'
import { getAPIDoc, getAllAPIDocs } from '@/lib/api'
import { APIContent } from '@/components/api-docs/APIContent'

export async function generateStaticParams() {
  const slugs = getAllAPIDocs()
  return slugs.map((slug) => ({ slug }))
}

export async function generateMetadata({ params }: { params: Promise<{ slug: string[] }> }) {
  const { slug } = await params
  const doc = await getAPIDoc(slug)

  if (!doc) {
    return {
      title: 'Page Not Found',
    }
  }

  return {
    title: `${doc.metadata.title} | tana API`,
    description: doc.metadata.description || `API documentation for ${doc.metadata.title}`,
  }
}

export default async function APIDocPage({ params }: { params: Promise<{ slug: string[] }> }) {
  const { slug } = await params
  const doc = await getAPIDoc(slug)

  if (!doc) {
    notFound()
  }

  return (
    <div className="max-w-5xl mx-auto px-8 py-12">
      <article className="max-w-none">
        <h1 className="text-4xl font-bold text-foreground mb-2">
          {doc.metadata.title}
        </h1>
        {doc.metadata.description && (
          <p className="text-xl text-muted-foreground mb-8">
            {doc.metadata.description}
          </p>
        )}

        <APIContent html={doc.html} codeBlocks={doc.codeBlocks} />
      </article>
    </div>
  )
}
