import { notFound } from 'next/navigation'
import { getDoc } from '@/lib/docs'
import { DocsContent } from '@/components/docs/DocsContent'

// Force dynamic rendering for client-side features
export const dynamic = 'force-dynamic'

// Generate metadata for SEO
export async function generateMetadata({ params }: { params: Promise<{ slug: string[] }> }) {
  const { slug } = await params
  const doc = getDoc(slug)

  if (!doc) {
    return {
      title: 'Page Not Found',
    }
  }

  return {
    title: `${doc.metadata.title} | tana Docs`,
    description: doc.metadata.description || `Documentation for ${doc.metadata.title}`,
  }
}

export default async function DocPage({ params }: { params: Promise<{ slug: string[] }> }) {
  const { slug } = await params
  const doc = getDoc(slug)

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

        <DocsContent html={doc.html} codeBlocks={doc.codeBlocks} />
      </article>
    </div>
  )
}
