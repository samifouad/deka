import Link from 'next/link'
import { Navbar } from '@/components/landing/Navbar'
import { getBlogPost, getBlogPosts } from '@/lib/blog-posts'
import { Calendar, Clock, User, ArrowLeft, ArrowRight } from 'lucide-react'
import { notFound } from 'next/navigation'
import styles from '../blog.module.css'

export async function generateStaticParams() {
  const posts = getBlogPosts()

  return posts.map((post) => ({
    slug: post.slug,
  }))
}

export async function generateMetadata({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params
  const post = getBlogPost(slug)

  if (!post) {
    return {
      title: 'Post Not Found',
    }
  }

  return {
    title: `${post.title} | deka Blog`,
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

  const currentIndex = allPosts.findIndex((p) => p.slug === slug)
  const previousPost = currentIndex < allPosts.length - 1 ? allPosts[currentIndex + 1] : null
  const nextPost = currentIndex > 0 ? allPosts[currentIndex - 1] : null

  const contentHtml = post.content
    .replace(/^# (.*$)/gim, `<h1 class="${styles.articleH1}">$1</h1>`)
    .replace(/^## (.*$)/gim, `<h2 class="${styles.articleH2}">$1</h2>`)
    .replace(/^### (.*$)/gim, `<h3 class="${styles.articleH3}">$1</h3>`)
    .replace(/\*\*(.*?)\*\*/g, `<strong class="${styles.articleStrong}">$1</strong>`)
    .replace(/^- (.*$)/gim, `<li class="${styles.articleListItem}">$1</li>`)
    .replace(/^(\d+)\. (.*$)/gim, `<li class="${styles.articleListItem}">$2</li>`)
    .split('\n\n')
    .map((para) => {
      if (para.startsWith('<h') || para.startsWith('<li')) {
        return para
      }
      return `<p class="${styles.articleParagraph}">${para}</p>`
    })
    .join('\n')

  return (
    <div className={styles.page}>
      <Navbar />
      <article className={styles.article}>
        <Link href="/blog" className={styles.backLink}>
          <ArrowLeft className="h-4 w-4" />
          Back to Blog
        </Link>

        <header className={styles.articleHeader}>
          <div className={styles.tagList}>
            {post.tags.map((tag) => (
              <span key={tag} className={styles.tag}>
                {tag}
              </span>
            ))}
          </div>

          <h1 className={styles.articleTitle}>{post.title}</h1>
          <p className={styles.articleExcerpt}>{post.excerpt}</p>

          <div className={styles.articleMeta}>
            <span className={styles.metaItem}>
              <User className="h-4 w-4" />
              {post.author.name}
            </span>
            <span className={styles.metaItem}>
              <Calendar className="h-4 w-4" />
              {new Date(post.date).toLocaleDateString('en-US', {
                month: 'long',
                day: 'numeric',
                year: 'numeric'
              })}
            </span>
            <span className={styles.metaItem}>
              <Clock className="h-4 w-4" />
              {post.readTime} read
            </span>
          </div>
        </header>

        <div
          className={styles.articleBody}
          dangerouslySetInnerHTML={{
            __html: contentHtml
          }}
        />

        <div className={styles.shareRow}>
          <p className={styles.subtitle}>Share this article:</p>
          <div className={styles.shareButtons}>
            <button className={styles.secondaryButton}>Twitter</button>
            <button className={styles.secondaryButton}>LinkedIn</button>
            <button className={styles.secondaryButton}>Copy Link</button>
          </div>
        </div>
      </article>

      <section className={styles.postNav}>
        <div className={styles.postNavGrid}>
          {previousPost && (
            <Link href={`/blog/${previousPost.slug}`} className={styles.postNavCard}>
              <p className={styles.subtitle}>
                <ArrowLeft className="h-4 w-4" /> Previous Article
              </p>
              <h3 className={styles.postTitle}>{previousPost.title}</h3>
            </Link>
          )}

          {nextPost && (
            <Link href={`/blog/${nextPost.slug}`} className={styles.postNavCard}>
              <p className={styles.subtitle}>
                Next Article <ArrowRight className="h-4 w-4" />
              </p>
              <h3 className={styles.postTitle}>{nextPost.title}</h3>
            </Link>
          )}
        </div>
      </section>
    </div>
  )
}
