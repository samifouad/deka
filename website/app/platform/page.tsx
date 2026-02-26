'use client'

import Link from 'next/link'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { BlueprintGrid } from '@/components/landing/BlueprintGrid'
import { Navbar } from '@/components/landing/Navbar'
import {
  Code2,
  Server,
  Layers,
  Shield,
  Zap,
  Globe2,
  Network,
  Database,
  Smartphone
} from 'lucide-react'

export default function PlatformPage() {
  return (
    <div className="min-h-screen bg-background text-foreground overflow-hidden">
      <BlueprintGrid />
      <Navbar />

      {/* Hero Section */}
      <section className="relative px-4 py-24 lg:px-8">
        <div className="absolute inset-0 bg-gradient-to-br from-primary/12 via-background/30 to-primary/5 backdrop-blur-sm"></div>

        <div className="relative mx-auto max-w-7xl">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <div className="text-center lg:text-left">
              <div className="inline-flex items-center gap-2 rounded-full border border-primary/30 bg-primary/10 px-4 py-2 text-xs font-semibold tracking-[0.2em] text-primary">
                DEKA PLATFORM
              </div>
              <h1 className="text-5xl lg:text-7xl font-bold text-foreground mb-6 leading-tight">
                Build sovereign apps with a platform that ships.
              </h1>
              <p className="text-xl lg:text-2xl text-muted-foreground mb-8 max-w-2xl">
                Deka is the programmable platform for on-chain products: runtime services, a TypeScript framework, and deploy tooling that stays close to your infrastructure.
              </p>

              <div className="flex flex-col sm:flex-row gap-4 justify-center lg:justify-start mb-12">
                <Button
                  size="lg"
                  className="bg-primary hover:bg-primary/90 text-primary-foreground font-bold"
                  asChild
                >
                  <Link href="/install">Install</Link>
                </Button>
                <Button
                  variant="outline"
                  size="lg"
                  className="border-border text-foreground hover:bg-accent"
                  asChild
                >
                  <Link href="/developers">Read the docs</Link>
                </Button>
              </div>

              <div className="flex flex-wrap gap-4 justify-center lg:justify-start text-sm text-muted-foreground">
                <div className="flex items-center gap-2">
                  <Zap className="w-4 h-4 text-primary" />
                  <span>Low-latency runtime services</span>
                </div>
                <div className="flex items-center gap-2">
                  <Shield className="w-4 h-4 text-primary" />
                  <span>Deterministic, verifiable execution</span>
                </div>
                <div className="flex items-center gap-2">
                  <Globe2 className="w-4 h-4 text-primary" />
                  <span>Deploy across regions</span>
                </div>
              </div>
            </div>

            <div className="flex justify-center">
              <div className="relative">
                <div className="w-[520px] max-w-full">
                  <div className="h-[320px] bg-gradient-to-br from-primary to-primary/60 border-4 border-border rounded-3xl shadow-2xl flex items-center justify-center">
                    <div className="text-center p-8">
                      <Layers className="w-24 h-24 mx-auto mb-4 text-muted-foreground" />
                      <p className="text-muted-foreground">Platform architecture preview</p>
                    </div>
                  </div>
                  <div className="mx-auto mt-2 h-4 w-[85%] rounded-b-3xl bg-border/70 shadow-lg"></div>
                </div>
                <div className="absolute -left-6 top-20 bg-primary/10 border border-primary/30 rounded-lg p-3 backdrop-blur-sm">
                  <Server className="w-6 h-6 text-primary" />
                </div>
                <div className="absolute -right-6 bottom-28 bg-primary/10 border border-primary/30 rounded-lg p-3 backdrop-blur-sm">
                  <Code2 className="w-6 h-6 text-primary" />
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Platform Services Section */}
      <section className="relative px-4 py-16 lg:px-8 bg-white dark:bg-background">
        <div className="mx-auto max-w-7xl">
          <div className="text-center mb-16">
            <h2 className="text-4xl font-bold text-foreground mb-4">
              Platform services that move at product speed
            </h2>
            <p className="text-lg text-muted-foreground max-w-3xl mx-auto">
              Deka ships with a runtime stack of composable services: identity, ledger, messaging, and workflow primitives designed for production-grade apps.
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
            <Card className="bg-card border-border backdrop-blur-sm">
              <CardContent className="pt-6">
                <Network className="w-8 h-8 text-primary mb-3" />
                <h3 className="text-lg font-bold text-foreground mb-2">Composable services</h3>
                <p className="text-muted-foreground text-sm">
                  Wire identity, ledger, queues, and notifications without stitching together third-party vendors.
                </p>
              </CardContent>
            </Card>

            <Card className="bg-card border-border backdrop-blur-sm">
              <CardContent className="pt-6">
                <Database className="w-8 h-8 text-primary mb-3" />
                <h3 className="text-lg font-bold text-foreground mb-2">Operational primitives</h3>
                <p className="text-muted-foreground text-sm">
                  Ship stateful services with consensus-ready storage, monitoring hooks, and health endpoints baked in.
                </p>
              </CardContent>
            </Card>

            <Card className="bg-card border-border backdrop-blur-sm">
              <CardContent className="pt-6">
                <Shield className="w-8 h-8 text-primary mb-3" />
                <h3 className="text-lg font-bold text-foreground mb-2">Security by default</h3>
                <p className="text-muted-foreground text-sm">
                  Deterministic execution, signed operations, and sealed secrets keep infrastructure predictable.
                </p>
              </CardContent>
            </Card>
          </div>
        </div>
      </section>

      {/* Remote Management Section */}
      <section className="relative px-4 py-16 lg:px-8 bg-white dark:bg-background">
        <div className="mx-auto max-w-7xl">
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <div>
              <div className="inline-block bg-primary/10 rounded-full px-4 py-2 mb-4">
                <span className="text-primary font-semibold text-sm">REMOTE ACCESS</span>
              </div>
              <h2 className="text-4xl font-bold text-foreground mb-4">
                Manage your Deka instance on the go
              </h2>
              <p className="text-lg text-muted-foreground mb-6">
                Stay close to deployments, status, and alerts from any device. Keep your runtime healthy while you are away from the terminal.
              </p>
              <ul className="space-y-3">
                <li className="flex items-start gap-3">
                  <Smartphone className="w-5 h-5 text-primary mt-0.5" />
                  <span className="text-muted-foreground">Live service status and health signals</span>
                </li>
                <li className="flex items-start gap-3">
                  <Shield className="w-5 h-5 text-primary mt-0.5" />
                  <span className="text-muted-foreground">Secure access tied to instance roles</span>
                </li>
                <li className="flex items-start gap-3">
                  <Zap className="w-5 h-5 text-primary mt-0.5" />
                  <span className="text-muted-foreground">Respond quickly to deploy events and alerts</span>
                </li>
              </ul>
            </div>

            <div className="flex justify-center">
              <div className="relative">
                <div className="w-72 h-[520px] bg-gradient-to-br from-primary to-primary/60 border-4 border-border rounded-[2.5rem] shadow-2xl flex items-center justify-center">
                  <div className="text-center p-8">
                    <Smartphone className="w-24 h-24 mx-auto mb-4 text-muted-foreground" />
                    <p className="text-muted-foreground">Remote management preview</p>
                  </div>
                </div>
                <div className="absolute -left-5 top-20 bg-primary/10 border border-primary/30 rounded-lg p-3 backdrop-blur-sm">
                  <Shield className="w-6 h-6 text-primary" />
                </div>
                <div className="absolute -right-5 bottom-28 bg-primary/10 border border-primary/30 rounded-lg p-3 backdrop-blur-sm">
                  <Zap className="w-6 h-6 text-primary" />
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="relative px-4 py-24 lg:px-8 bg-white dark:bg-background">
        <div className="mx-auto max-w-4xl text-center">
          <h2 className="text-4xl lg:text-5xl font-bold text-foreground mb-6">
            Coming Soon: The platform for sovereign applications.
          </h2>
          <p className="text-lg text-muted-foreground mb-8 max-w-2xl mx-auto">
            Deka is in active development. Follow our progress and be the first to know when it's ready for production.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Button variant="outline" size="lg" className="border-border text-foreground hover:bg-accent" asChild>
              <Link href="/developers">View Documentation</Link>
            </Button>
          </div>
          <p className="text-sm text-muted-foreground mt-6">
            <strong className="text-foreground">Early Prototype:</strong> The platform is under active development. Public release coming soon.
          </p>
        </div>
      </section>
    </div>
  )
}
