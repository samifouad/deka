import Link from 'next/link'
import { Button } from '@/components/ui/button'
import { BlueprintGrid } from '@/components/landing/BlueprintGrid'
import { Navbar } from '@/components/landing/Navbar'
import { getBlogPost, getBlogPosts } from '@/lib/blog-posts'
import { Calendar, Clock, User, ArrowLeft, ArrowRight } from 'lucide-react'
import { notFound } from 'next/navigation'

// Tell Next.js to statically generate all blog post routes at build time
export async function generateStaticParams() {
  const posts = getBlogPosts()

  return posts.map((post) => ({
    slug: post.slug,
  }))
}

// Generate metadata for SEO
export async function generateMetadata({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params
  const post = getBlogPost(slug)

  if (!post) {
    return {
      title: 'Post Not Found',
    }
  }

  return {
    title: `${post.title} | tana Blog`,
    description: post.excerpt,
  }
}

export default async function BlogPostPage({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params
  const post = getBlogPost(slug)
  const allPosts = getBlogPosts()

  if (!post) {
    notFound()
  }

  // Find next and previous posts
  const currentIndex = allPosts.findIndex(p => p.slug === slug)
  const previousPost = currentIndex < allPosts.length - 1 ? allPosts[currentIndex + 1] : null
  const nextPost = currentIndex > 0 ? allPosts[currentIndex - 1] : null

  return (
    <div className="min-h-screen bg-background text-foreground overflow-hidden">
      <BlueprintGrid />
      <Navbar />

      {/* Back to Blog */}
      <section className="relative px-4 pt-8 lg:px-8">
        <div className="mx-auto max-w-4xl">
          <Link href="/blog">
            <Button variant="ghost" className="text-muted-foreground hover:text-foreground">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Blog
            </Button>
          </Link>
        </div>
      </section>

      {/* Article Header */}
      <article className="relative px-4 py-12 lg:px-8">
        <div className="mx-auto max-w-4xl">
          {/* Tags */}
          <div className="flex flex-wrap gap-2 mb-6">
            {post.tags.map((tag) => (
              <span key={tag} className="text-xs px-3 py-1 bg-primary/10 text-primary rounded-full">
                {tag}
              </span>
            ))}
          </div>

          {/* Title */}
          <h1 className="text-4xl lg:text-5xl font-bold text-foreground mb-6 leading-tight">
            {post.title}
          </h1>

          {/* Excerpt */}
          <p className="text-xl text-muted-foreground mb-8">
            {post.excerpt}
          </p>

          {/* Meta info */}
          <div className="flex flex-wrap items-center gap-6 text-sm text-muted-foreground pb-8 border-b border-border mb-12">
            <div className="flex items-center gap-3">
              <div className="w-12 h-12 bg-primary/10 rounded-full flex items-center justify-center">
                <User className="w-6 h-6 text-primary" />
              </div>
              <div>
                <p className="font-semibold text-foreground">{post.author.name}</p>
                <p className="text-xs">{post.author.role}</p>
              </div>
            </div>

            <div className="flex items-center gap-2">
              <Calendar className="w-4 h-4" />
              <span>
                {new Date(post.date).toLocaleDateString('en-US', {
                  month: 'long',
                  day: 'numeric',
                  year: 'numeric'
                })}
              </span>
            </div>

            <div className="flex items-center gap-2">
              <Clock className="w-4 h-4" />
              <span>{post.readTime} read</span>
            </div>
          </div>

          {/* Content */}
          <div className="prose prose-lg prose-invert max-w-none">
            <div
              className="text-muted-foreground leading-relaxed"
              dangerouslySetInnerHTML={{
                __html: post.content
                  .replace(/^# (.*$)/gim, '<h1 class="text-3xl font-bold text-foreground mt-8 mb-4">$1</h1>')
                  .replace(/^## (.*$)/gim, '<h2 class="text-2xl font-bold text-foreground mt-6 mb-3">$2</h2>')
                  .replace(/^### (.*$)/gim, '<h3 class="text-xl font-bold text-foreground mt-4 mb-2">$3</h3>')
                  .replace(/\*\*(.*?)\*\*/g, '<strong class="font-bold text-foreground">$1</strong>')
                  .replace(/^- (.*$)/gim, '<li class="ml-6 mb-2">$1</li>')
                  .replace(/^(\d+)\. (.*$)/gim, '<li class="ml-6 mb-2">$2</li>')
                  .split('\n\n')
                  .map(para => {
                    if (para.startsWith('<h') || para.startsWith('<li')) {
                      return para
                    }
                    return `<p class="mb-4">${para}</p>`
                  })
                  .join('\n')
              }}
            />
          </div>

          {/* Share section */}
          <div className="mt-12 pt-8 border-t border-border">
            <p className="text-sm text-muted-foreground mb-4">Share this article:</p>
            <div className="flex gap-2">
              <Button variant="outline" size="sm" className="border-border">Twitter</Button>
              <Button variant="outline" size="sm" className="border-border">LinkedIn</Button>
              <Button variant="outline" size="sm" className="border-border">Copy Link</Button>
            </div>
          </div>
        </div>
      </article>

      {/* Navigation to other posts */}
      <section className="relative px-4 py-12 lg:px-8 bg-secondary/30">
        <div className="mx-auto max-w-4xl">
          <div className="grid md:grid-cols-2 gap-6">
            {/* Previous post */}
            {previousPost && (
              <Link href={`/blog/${previousPost.slug}`}>
                <div className="bg-card border border-border rounded-lg p-6 hover:border-primary/50 transition-all h-full">
                  <p className="text-sm text-muted-foreground mb-2 flex items-center gap-2">
                    <ArrowLeft className="w-4 h-4" />
                    Previous Article
                  </p>
                  <h3 className="text-lg font-bold text-foreground hover:text-primary transition-colors">
                    {previousPost.title}
                  </h3>
                </div>
              </Link>
            )}

            {/* Next post */}
            {nextPost && (
              <Link href={`/blog/${nextPost.slug}`}>
                <div className="bg-card border border-border rounded-lg p-6 hover:border-primary/50 transition-all h-full text-right md:ml-auto">
                  <p className="text-sm text-muted-foreground mb-2 flex items-center justify-end gap-2">
                    Next Article
                    <ArrowRight className="w-4 h-4" />
                  </p>
                  <h3 className="text-lg font-bold text-foreground hover:text-primary transition-colors">
                    {nextPost.title}
                  </h3>
                </div>
              </Link>
            )}
          </div>
        </div>
      </section>

    </div>
  )
}
