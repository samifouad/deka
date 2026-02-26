'use client'

import Link from 'next/link'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { BlueprintGrid } from '@/components/landing/BlueprintGrid'
import { Navbar } from '@/components/landing/Navbar'
import { getRFDs, getStatusColor } from '@/lib/rfd-data'
import { Calendar, Users, MessageSquare, FileText, ExternalLink } from 'lucide-react'

export default function RFDPage() {
  const rfds = getRFDs()

  // Group by status
  const activeRFDs = rfds.filter(r => ['ideation', 'discussion', 'published'].includes(r.status))
  const committedRFDs = rfds.filter(r => r.status === 'committed')
  const otherRFDs = rfds.filter(r => ['prediscussion', 'abandoned'].includes(r.status))

  return (
    <div className="min-h-screen bg-background text-foreground overflow-hidden">
      <BlueprintGrid />
      <Navbar />

      {/* Hero Section */}
      <section className="relative px-4 py-24 lg:px-8">
        <div className="absolute inset-0 bg-gradient-to-br from-primary/10 via-background/20 to-primary/5 backdrop-blur-sm"></div>

        <div className="relative mx-auto max-w-7xl">
          <div className="text-center mb-12">
            <h1 className="text-5xl lg:text-6xl font-bold text-foreground mb-6">
              Requests for Discussion
            </h1>
            <p className="text-xl text-muted-foreground max-w-3xl mx-auto mb-8">
              RFDs are our process for proposing, discussing, and documenting significant changes to tana.
              Anyone can submit an RFD. All discussions are public.
            </p>
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <Link href="/rfd/1">
                <Button variant="outline" className="border-border hover:bg-accent">
                  <FileText className="w-4 h-4 mr-2" />
                  Learn About RFDs
                </Button>
              </Link>
              <Button className="bg-primary hover:bg-primary/90 text-primary-foreground">
                <ExternalLink className="w-4 h-4 mr-2" />
                Submit an RFD
              </Button>
            </div>
          </div>

          {/* Stats */}
          <div className="grid md:grid-cols-4 gap-4 max-w-4xl mx-auto">
            <Card className="bg-card border-border backdrop-blur-sm text-center">
              <CardContent className="pt-6">
                <div className="text-3xl font-bold text-primary mb-1">{rfds.length}</div>
                <p className="text-sm text-muted-foreground">Total RFDs</p>
              </CardContent>
            </Card>
            <Card className="bg-card border-border backdrop-blur-sm text-center">
              <CardContent className="pt-6">
                <div className="text-3xl font-bold text-yellow-500 mb-1">{activeRFDs.length}</div>
                <p className="text-sm text-muted-foreground">Active</p>
              </CardContent>
            </Card>
            <Card className="bg-card border-border backdrop-blur-sm text-center">
              <CardContent className="pt-6">
                <div className="text-3xl font-bold text-green-500 mb-1">{committedRFDs.length}</div>
                <p className="text-sm text-muted-foreground">Committed</p>
              </CardContent>
            </Card>
            <Card className="bg-card border-border backdrop-blur-sm text-center">
              <CardContent className="pt-6">
                <div className="text-3xl font-bold text-muted-foreground mb-1">
                  {new Set(rfds.flatMap(r => r.authors)).size}
                </div>
                <p className="text-sm text-muted-foreground">Contributors</p>
              </CardContent>
            </Card>
          </div>
        </div>
      </section>

      {/* Active RFDs */}
      {activeRFDs.length > 0 && (
        <section className="relative px-4 py-16 lg:px-8">
          <div className="mx-auto max-w-7xl">
            <h2 className="text-3xl font-bold text-foreground mb-8">Active Discussions</h2>
            <div className="grid md:grid-cols-2 gap-6">
              {activeRFDs.map((rfd) => (
                <Card key={rfd.number} className="bg-card border-border backdrop-blur-sm hover:border-primary/50 transition-all">
                  <CardHeader>
                    <div className="flex items-start justify-between gap-4 mb-3">
                      <div className="flex items-center gap-3">
                        <span className="text-2xl font-bold text-primary">#{rfd.number}</span>
                        <span className={`text-xs px-2 py-1 rounded-full capitalize ${getStatusColor(rfd.status)}`}>
                          {rfd.status}
                        </span>
                      </div>
                    </div>

                    <CardTitle className="text-foreground hover:text-primary transition-colors">
                      <Link href={`/rfd/${rfd.number}`}>
                        {rfd.title}
                      </Link>
                    </CardTitle>

                    <CardDescription>
                      <div className="flex flex-wrap gap-2 mt-3">
                        {rfd.tags.map((tag) => (
                          <span key={tag} className="text-xs px-2 py-1 bg-secondary rounded-full text-secondary-foreground">
                            {tag}
                          </span>
                        ))}
                      </div>
                    </CardDescription>
                  </CardHeader>

                  <CardContent>
                    <div className="flex items-center justify-between text-sm text-muted-foreground mb-4">
                      <div className="flex items-center gap-2">
                        <Users className="w-4 h-4" />
                        <span>{rfd.authors.join(', ')}</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <Calendar className="w-4 h-4" />
                        <span>{new Date(rfd.updated).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}</span>
                      </div>
                    </div>

                    <div className="flex gap-2">
                      <Link href={`/rfd/${rfd.number}`} className="flex-1">
                        <Button variant="outline" className="w-full border-border hover:bg-accent">
                          Read RFD
                        </Button>
                      </Link>
                      <a href={rfd.discussion} target="_blank" rel="noopener noreferrer">
                        <Button variant="outline" className="border-border hover:bg-accent">
                          <MessageSquare className="w-4 h-4" />
                        </Button>
                      </a>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>
        </section>
      )}

      {/* Committed RFDs */}
      {committedRFDs.length > 0 && (
        <section className="relative px-4 py-16 lg:px-8 bg-secondary/30">
          <div className="mx-auto max-w-7xl">
            <h2 className="text-3xl font-bold text-foreground mb-8">Committed (Implemented)</h2>
            <div className="grid md:grid-cols-3 gap-4">
              {committedRFDs.map((rfd) => (
                <Card key={rfd.number} className="bg-card border-border backdrop-blur-sm hover:border-green-500/50 transition-all">
                  <CardContent className="pt-6">
                    <div className="flex items-center gap-3 mb-3">
                      <span className="text-xl font-bold text-green-500">#{rfd.number}</span>
                      <span className={`text-xs px-2 py-1 rounded-full capitalize ${getStatusColor(rfd.status)}`}>
                        {rfd.status}
                      </span>
                    </div>

                    <h3 className="font-bold text-foreground mb-2 hover:text-green-500 transition-colors">
                      <Link href={`/rfd/${rfd.number}`}>
                        {rfd.title}
                      </Link>
                    </h3>

                    <div className="flex items-center gap-2 text-sm text-muted-foreground mb-3">
                      <Users className="w-3 h-3" />
                      <span className="text-xs">{rfd.authors[0]}</span>
                    </div>

                    <Link href={`/rfd/${rfd.number}`}>
                      <Button variant="ghost" size="sm" className="w-full text-green-500 hover:text-green-400">
                        View Details
                      </Button>
                    </Link>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>
        </section>
      )}

      {/* Other RFDs (Prediscussion, Abandoned) */}
      {otherRFDs.length > 0 && (
        <section className="relative px-4 py-16 lg:px-8">
          <div className="mx-auto max-w-7xl">
            <h2 className="text-3xl font-bold text-foreground mb-8">Archive</h2>
            <div className="space-y-3">
              {otherRFDs.map((rfd) => (
                <Card key={rfd.number} className="bg-card border-border backdrop-blur-sm">
                  <CardContent className="pt-6">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-4 flex-1">
                        <span className="text-xl font-bold text-muted-foreground">#{rfd.number}</span>
                        <span className={`text-xs px-2 py-1 rounded-full capitalize ${getStatusColor(rfd.status)}`}>
                          {rfd.status}
                        </span>
                        <Link href={`/rfd/${rfd.number}`}>
                          <span className="font-semibold text-foreground hover:text-primary transition-colors">
                            {rfd.title}
                          </span>
                        </Link>
                      </div>

                      <div className="flex items-center gap-4 text-sm text-muted-foreground">
                        <span>{rfd.authors.join(', ')}</span>
                        <Link href={`/rfd/${rfd.number}`}>
                          <Button variant="ghost" size="sm">
                            View
                          </Button>
                        </Link>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>
        </section>
      )}

      {/* How to Participate */}
      <section className="relative px-4 py-16 lg:px-8 bg-secondary/30">
        <div className="mx-auto max-w-4xl">
          <h2 className="text-3xl font-bold text-foreground mb-8 text-center">How to Participate</h2>

          <div className="grid md:grid-cols-3 gap-6">
            <Card className="bg-card border-border backdrop-blur-sm text-center">
              <CardContent className="pt-8">
                <div className="w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center mx-auto mb-4">
                  <FileText className="w-8 h-8 text-primary" />
                </div>
                <h3 className="text-lg font-bold text-foreground mb-2">1. Read RFD 1</h3>
                <p className="text-muted-foreground text-sm">
                  Learn about our RFD process and guidelines
                </p>
              </CardContent>
            </Card>

            <Card className="bg-card border-border backdrop-blur-sm text-center">
              <CardContent className="pt-8">
                <div className="w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center mx-auto mb-4">
                  <MessageSquare className="w-8 h-8 text-primary" />
                </div>
                <h3 className="text-lg font-bold text-foreground mb-2">2. Join Discussions</h3>
                <p className="text-muted-foreground text-sm">
                  Comment on GitHub to provide feedback
                </p>
              </CardContent>
            </Card>

            <Card className="bg-card border-border backdrop-blur-sm text-center">
              <CardContent className="pt-8">
                <div className="w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center mx-auto mb-4">
                  <ExternalLink className="w-8 h-8 text-primary" />
                </div>
                <h3 className="text-lg font-bold text-foreground mb-2">3. Submit Your Own</h3>
                <p className="text-muted-foreground text-sm">
                  Create a pull request with your proposal
                </p>
              </CardContent>
            </Card>
          </div>
        </div>
      </section>

    </div>
  )
}
