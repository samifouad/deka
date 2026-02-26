import Link from 'next/link'
import { Navbar } from '@/components/landing/Navbar'
import { getBlogPosts } from '@/lib/blog-posts'
import { Calendar, Clock, User, ArrowRight } from 'lucide-react'
import styles from './blog.module.css'

// Generate metadata for SEO
export const metadata = {
  title: 'Blog | deka',
  description: 'Updates, insights, and stories from the deka team. Learn about the runtime, platform, and developer experience.',
}

export default function BlogPage() {
  const posts = getBlogPosts()
  const featuredImage =
    'https://images.pexels.com/photos/1181675/pexels-photo-1181675.jpeg?auto=compress&cs=tinysrgb&w=1200&h=800&dpr=2'

  return (
    <div className={styles.page}>
      <Navbar />
      <main className={styles.main}>
        <section className={styles.hero}>
          <h1 className={styles.heroTitle}>Deka Blog</h1>
          <p className={styles.heroSubtitle}>
            Updates, insights, and stories from the Deka team. Learn about the runtime, platform, and developer experience.
          </p>
        </section>

      {/* Featured Post */}
        {posts[0] && (
          <section>
            <div className={styles.featuredCard}>
              <div
                className={styles.featuredMedia}
                style={{ backgroundImage: `url(${featuredImage})` }}
              >
                <div className={styles.featuredOverlay} />
              </div>
              <div className={styles.featuredContent}>
                <span className={styles.featuredBadge}>Featured</span>
                <h2 className={styles.featuredTitle}>{posts[0].title}</h2>
                <p className={styles.featuredExcerpt}>{posts[0].excerpt}</p>
                <div className={styles.metaRow}>
                  <span className={styles.metaItem}>
                    <User className="h-4 w-4" />
                    {posts[0].author.name}
                  </span>
                  <span className={styles.metaItem}>
                    <Calendar className="h-4 w-4" />
                    {new Date(posts[0].date).toLocaleDateString('en-US', {
                      month: 'long',
                      day: 'numeric',
                      year: 'numeric'
                    })}
                  </span>
                  <span className={styles.metaItem}>
                    <Clock className="h-4 w-4" />
                    {posts[0].readTime} read
                  </span>
                </div>
                <Link href={`/blog/${posts[0].slug}`} className={styles.primaryButton}>
                  Read Article
                  <ArrowRight className="h-4 w-4" />
                </Link>
              </div>
            </div>
          </section>
        )}

        <section>
          <h2 className={styles.sectionTitle}>Latest Articles</h2>
          <div className={styles.grid}>
            {posts.map((post) => (
              <article key={post.slug} className={styles.postCard}>
                <div className={styles.tagList}>
                  {post.tags.map((tag) => (
                    <span key={tag} className={styles.tag}>
                      {tag}
                    </span>
                  ))}
                </div>
                <Link href={`/blog/${post.slug}`}>
                  <h3 className={styles.postTitle}>{post.title}</h3>
                </Link>
                <p className={styles.postExcerpt}>{post.excerpt}</p>
                <div className={styles.postMeta}>
                  <span className={styles.metaItem}>
                    <Calendar className="h-4 w-4" />
                    {new Date(post.date).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}
                  </span>
                  <span className={styles.metaItem}>
                    <Clock className="h-4 w-4" />
                    {post.readTime}
                  </span>
                </div>
                <div className={styles.postAuthor}>
                  <span className={styles.authorBadge}>
                    <User className="h-4 w-4" />
                  </span>
                  <div>
                    <div className={styles.postTitle}>{post.author.name}</div>
                    <div className={styles.postExcerpt}>{post.author.role}</div>
                  </div>
                </div>
                <Link href={`/blog/${post.slug}`} className={styles.secondaryButton}>
                  Read More
                  <ArrowRight className="h-4 w-4" />
                </Link>
              </article>
            ))}
          </div>
        </section>

        <section className={styles.newsletter}>
          <h2 className={styles.newsletterTitle}>Stay in the loop</h2>
          <p className={styles.newsletterText}>
            Get the latest updates, articles, and announcements delivered to your inbox.
          </p>
          <div className={styles.newsletterForm}>
            <input type="email" placeholder="Enter your email" className={styles.newsletterInput} />
            <button className={styles.primaryButton}>Subscribe</button>
          </div>
        </section>
      </main>
    </div>
  )
}
