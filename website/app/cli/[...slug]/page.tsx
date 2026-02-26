import { notFound } from 'next/navigation'
import { getCLIDoc, getAllCLIDocs } from '@/lib/cli'
import { CLIContent } from '@/components/cli-docs/CLIContent'
import { CLIOverview } from '@/components/cli-docs/CLIOverview'

export async function generateStaticParams() {
  const slugs = getAllCLIDocs()
  return slugs.map((slug) => ({ slug }))
}

export async function generateMetadata({ params }: { params: Promise<{ slug: string[] }> }) {
  const { slug } = await params

  // Special case for overview
  if (slug.length === 1 && slug[0] === 'overview') {
    return {
      title: 'Deka CLI',
      description: 'Command-line tools to deploy, manage, and scale your own infrastructure.',
    }
  }

  const doc = await getCLIDoc(slug)

  if (!doc) {
    return {
      title: 'Page Not Found',
    }
  }

  return {
    title: `${doc.metadata.title} | deka CLI`,
    description: doc.metadata.description || `CLI documentation for ${doc.metadata.title}`,
  }
}

export default async function CLIDocPage({ params }: { params: Promise<{ slug: string[] }> }) {
  const { slug } = await params

  // Special case for overview - render custom component
  if (slug.length === 1 && slug[0] === 'overview') {
    return <CLIOverview />
  }

  const doc = await getCLIDoc(slug)

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

        <CLIContent html={doc.html} codeBlocks={doc.codeBlocks} />
      </article>
    </div>
  )
}
