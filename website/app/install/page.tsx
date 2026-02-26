'use client'

import Link from 'next/link'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { BlueprintGrid } from '@/components/landing/BlueprintGrid'
import { Navbar } from '@/components/landing/Navbar'
import { Server, Terminal, Layers, Shield, CheckCircle2 } from 'lucide-react'

const installOptions = [
  {
    title: 'Containerized runtime',
    description: 'Run Deka with container orchestration and predictable upgrades.',
    icon: <Layers className="w-8 h-8 text-primary" />,
    bullets: [
      'Best for production clusters and CI deployments',
      'Pair with your existing observability stack',
      'Standard health checks and rolling restarts'
    ]
  },
  {
    title: 'Bare-metal binary',
    description: 'Install a single runtime binary on a Linux host.',
    icon: <Server className="w-8 h-8 text-primary" />,
    bullets: [
      'Ideal for dedicated hosts and edge servers',
      'Keep dependencies minimal and explicit',
      'Systemd-friendly service layout'
    ]
  },
  {
    title: 'Source build',
    description: 'Build the runtime and services directly from source.',
    icon: <Terminal className="w-8 h-8 text-primary" />,
    bullets: [
      'Great for customizing services or extending modules',
      'Use the same workflow as core contributors',
      'Works well with custom deployment pipelines'
    ]
  }
]

const requirements = [
  'Linux host or container runtime',
  'Persistent storage for service data',
  'Outbound network access for dependencies',
  'Secrets management for service credentials'
]

export default function InstallPage() {
  return (
    <div className="min-h-screen bg-background text-foreground overflow-hidden">
      <BlueprintGrid />
      <Navbar />

      <section className="relative px-4 py-24 lg:px-8">
        <div className="absolute inset-0 bg-gradient-to-br from-primary/10 via-background/30 to-primary/5 backdrop-blur-sm"></div>

        <div className="relative mx-auto max-w-6xl">
          <div className="text-center mb-16">
            <div className="inline-flex items-center gap-2 rounded-full border border-primary/30 bg-primary/10 px-4 py-2 text-xs font-semibold tracking-[0.2em] text-primary">
              INSTALL
            </div>
            <h1 className="text-4xl lg:text-6xl font-bold text-foreground mt-6 mb-4">
              Install Deka on your server
            </h1>
            <p className="text-lg text-muted-foreground max-w-3xl mx-auto">
              Pick the deployment path that matches your infrastructure. Each option keeps the Deka runtime close to your data plane, with clear operational boundaries.
            </p>
          </div>

          <div className="grid lg:grid-cols-3 gap-6">
            {installOptions.map((option) => (
              <Card key={option.title} className="bg-card border-border backdrop-blur-sm">
                <CardContent className="pt-6">
                  <div className="mb-4">{option.icon}</div>
                  <h3 className="text-xl font-bold text-foreground mb-2">{option.title}</h3>
                  <p className="text-muted-foreground mb-4">{option.description}</p>
                  <ul className="space-y-2 text-sm text-muted-foreground">
                    {option.bullets.map((bullet) => (
                      <li key={bullet} className="flex items-start gap-2">
                        <CheckCircle2 className="w-4 h-4 text-primary mt-0.5" />
                        <span>{bullet}</span>
                      </li>
                    ))}
                  </ul>
                </CardContent>
              </Card>
            ))}
          </div>
        </div>
      </section>

      <section className="relative px-4 py-16 lg:px-8 bg-secondary/30">
        <div className="mx-auto max-w-5xl">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <div>
              <div className="inline-block bg-primary/10 rounded-full px-4 py-2 mb-4">
                <span className="text-primary font-semibold text-sm">REQUIREMENTS</span>
              </div>
              <h2 className="text-3xl font-bold text-foreground mb-4">
                What you need before installing
              </h2>
              <p className="text-muted-foreground mb-6">
                Deka runs best when you plan for data durability, secrets, and a stable network boundary.
              </p>
              <ul className="space-y-3">
                {requirements.map((item) => (
                  <li key={item} className="flex items-start gap-3">
                    <Shield className="w-5 h-5 text-primary mt-0.5" />
                    <span className="text-muted-foreground">{item}</span>
                  </li>
                ))}
              </ul>
            </div>

            <div className="bg-card border border-border rounded-2xl p-8 shadow-lg">
              <h3 className="text-xl font-bold text-foreground mb-4">Need a guided setup?</h3>
              <p className="text-muted-foreground mb-6">
                Jump into the developer docs for a step-by-step path that matches your infrastructure.
              </p>
              <Button asChild className="bg-primary hover:bg-primary/90 text-primary-foreground font-bold">
                <Link href="/developers">Open the docs</Link>
              </Button>
            </div>
          </div>
        </div>
      </section>
    </div>
  )
}
