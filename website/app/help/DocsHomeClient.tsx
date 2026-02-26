import Link from 'next/link'
import { SidebarSection } from '@/lib/docs'

interface DocsHomeClientProps {
  sections: SidebarSection[]
}

export function DocsHomeClient({ sections }: DocsHomeClientProps) {
  return (
    <article className="max-w-4xl mx-auto px-8 py-12">
      <h1 className="text-4xl font-bold text-foreground mb-4">
        Welcome to tana
      </h1>
      <p className="text-xl text-muted-foreground mb-8">
        tana is a blockchain platform with TypeScript smart contracts. Anyone can start their own blockchain or join existing networks as validators.
      </p>

      <div className="grid md:grid-cols-2 gap-6 my-12">
        <Link href="/docs/guides/quickstart">
          <div className="p-6 border border-border rounded-lg hover:border-primary/50 transition-all bg-card">
            <h3 className="text-xl font-bold text-foreground mb-2">Quick Start</h3>
            <p className="text-muted-foreground text-sm">
              Deploy your first smart contract in 5 minutes
            </p>
          </div>
        </Link>

        <Link href="/docs/tana-cli/commands">
          <div className="p-6 border border-border rounded-lg hover:border-primary/50 transition-all bg-card">
            <h3 className="text-xl font-bold text-foreground mb-2">CLI Reference</h3>
            <p className="text-muted-foreground text-sm">
              Learn the command-line interface
            </p>
          </div>
        </Link>

        <Link href="/docs/tana-api/intro">
          <div className="p-6 border border-border rounded-lg hover:border-primary/50 transition-all bg-card">
            <h3 className="text-xl font-bold text-foreground mb-2">API Reference</h3>
            <p className="text-muted-foreground text-sm">
              Explore the REST API endpoints
            </p>
          </div>
        </Link>

        <Link href="/docs/contributing/setup">
          <div className="p-6 border border-border rounded-lg hover:border-primary/50 transition-all bg-card">
            <h3 className="text-xl font-bold text-foreground mb-2">Contributing</h3>
            <p className="text-muted-foreground text-sm">
              Help build tana
            </p>
          </div>
        </Link>
      </div>

      <h2 className="text-3xl font-bold text-foreground mt-12 mb-4">Key Features</h2>

      <h3 className="text-2xl font-bold text-foreground mt-6 mb-3">TypeScript Smart Contracts</h3>
      <p className="text-muted-foreground mb-4">
        Write contracts in familiar TypeScript with full type safety and IDE support. No new language to learn.
      </p>

      <h3 className="text-2xl font-bold text-foreground mt-6 mb-3">Multi-Currency Support</h3>
      <p className="text-muted-foreground mb-4">
        No native token required. Create and manage any currency in your contracts.
      </p>

      <h3 className="text-2xl font-bold text-foreground mt-6 mb-3">CLI-First Design</h3>
      <p className="text-muted-foreground mb-4">
        Everything controllable from the terminal with powerful command-line tools.
      </p>
    </article>
  )
}
