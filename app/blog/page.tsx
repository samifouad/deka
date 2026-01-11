import Link from 'next/link'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { BlueprintGrid } from '@/components/landing/BlueprintGrid'
import { Navbar } from '@/components/landing/Navbar'
import { getBlogPosts } from '@/lib/blog-posts'
import { Calendar, Clock, User, ArrowRight } from 'lucide-react'

// Generate metadata for SEO
export const metadata = {
  title: 'Blog | tana',
  description: 'Updates, insights, and stories from the tana team. Learn about the future of money, identity, and decentralized commerce.',
}

export default function BlogPage() {
  const posts = getBlogPosts()

  return (
    <div className="min-h-screen bg-background text-foreground overflow-hidden">
      <BlueprintGrid />
      <Navbar />

      {/* Hero Section */}
      <section className="relative px-4 py-24 lg:px-8">
        <div className="absolute inset-0 bg-gradient-to-br from-primary/10 via-background/20 to-primary/5 backdrop-blur-sm"></div>

        <div className="relative mx-auto max-w-7xl text-center">
          <h1 className="text-5xl lg:text-6xl font-bold text-foreground mb-6">
            tana Blog
          </h1>
          <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
            Updates, insights, and stories from the tana team. Learn about the future of money, identity, and decentralized commerce.
          </p>
        </div>
      </section>

      {/* Featured Post */}
      {posts[0] && (
        <section className="relative px-4 pb-8 lg:px-8">
          <div className="mx-auto max-w-7xl">
            <Card className="bg-card border-border backdrop-blur-sm overflow-hidden">
              <div className="grid lg:grid-cols-2">
                {/* Image placeholder */}
                <div className="bg-gradient-to-br from-primary/20 to-primary/5 p-12 flex items-center justify-center">
                  <div className="text-center">
                    <div className="w-24 h-24 bg-primary/20 rounded-full flex items-center justify-center mx-auto mb-4">
                      <span className="text-4xl">📢</span>
                    </div>
                    <p className="text-muted-foreground">Featured Article</p>
                  </div>
                </div>

                {/* Content */}
                <div className="p-8 lg:p-12">
                  <div className="inline-block bg-primary/10 rounded-full px-3 py-1 mb-4">
                    <span className="text-primary font-semibold text-xs uppercase">Featured</span>
                  </div>

                  <h2 className="text-3xl font-bold text-foreground mb-4">
                    {posts[0].title}
                  </h2>

                  <p className="text-muted-foreground mb-6">
                    {posts[0].excerpt}
                  </p>

                  <div className="flex items-center gap-6 text-sm text-muted-foreground mb-6">
                    <div className="flex items-center gap-2">
                      <User className="w-4 h-4" />
                      <span>{posts[0].author.name}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <Calendar className="w-4 h-4" />
                      <span>{new Date(posts[0].date).toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' })}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <Clock className="w-4 h-4" />
                      <span>{posts[0].readTime} read</span>
                    </div>
                  </div>

                  <Link href={`/blog/${posts[0].slug}`}>
                    <Button className="bg-primary hover:bg-primary/90 text-primary-foreground">
                      Read Article
                      <ArrowRight className="w-4 h-4 ml-2" />
                    </Button>
                  </Link>
                </div>
              </div>
            </Card>
          </div>
        </section>
      )}

      {/* All Posts */}
      <section className="relative px-4 py-16 lg:px-8">
        <div className="mx-auto max-w-7xl">
          <h2 className="text-3xl font-bold text-foreground mb-8">Latest Articles</h2>

          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
            {posts.map((post) => (
              <Card key={post.slug} className="bg-card border-border backdrop-blur-sm hover:border-primary/50 transition-all">
                <CardHeader>
                  <div className="flex flex-wrap gap-2 mb-3">
                    {post.tags.map((tag) => (
                      <span key={tag} className="text-xs px-2 py-1 bg-primary/10 text-primary rounded-full">
                        {tag}
                      </span>
                    ))}
                  </div>

                  <CardTitle className="text-foreground hover:text-primary transition-colors">
                    <Link href={`/blog/${post.slug}`}>
                      {post.title}
                    </Link>
                  </CardTitle>

                  <CardDescription className="text-muted-foreground">
                    {post.excerpt}
                  </CardDescription>
                </CardHeader>

                <CardContent>
                  <div className="flex items-center justify-between text-sm text-muted-foreground mb-4">
                    <div className="flex items-center gap-2">
                      <Calendar className="w-4 h-4" />
                      <span>{new Date(post.date).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <Clock className="w-4 h-4" />
                      <span>{post.readTime}</span>
                    </div>
                  </div>

                  <div className="flex items-center gap-3 mb-4">
                    <div className="w-8 h-8 bg-primary/10 rounded-full flex items-center justify-center">
                      <User className="w-4 h-4 text-primary" />
                    </div>
                    <div className="text-sm">
                      <p className="font-semibold text-foreground">{post.author.name}</p>
                      <p className="text-muted-foreground text-xs">{post.author.role}</p>
                    </div>
                  </div>

                  <Link href={`/blog/${post.slug}`}>
                    <Button variant="outline" className="w-full border-border hover:bg-accent">
                      Read More
                      <ArrowRight className="w-4 h-4 ml-2" />
                    </Button>
                  </Link>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      </section>

      {/* Newsletter CTA */}
      <section className="relative px-4 py-16 lg:px-8 bg-secondary/30">
        <div className="mx-auto max-w-4xl text-center">
          <h2 className="text-3xl font-bold text-foreground mb-4">
            Stay in the loop
          </h2>
          <p className="text-lg text-muted-foreground mb-8">
            Get the latest updates, articles, and announcements delivered to your inbox.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center max-w-md mx-auto">
            <input
              type="email"
              placeholder="Enter your email"
              className="flex-1 px-4 py-2 bg-background border border-border rounded-lg focus:outline-none focus:border-primary text-foreground"
            />
            <Button className="bg-primary hover:bg-primary/90 text-primary-foreground">
              Subscribe
            </Button>
          </div>
        </div>
      </section>

    </div>
  )
}
