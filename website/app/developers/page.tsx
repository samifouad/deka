'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { BlueprintGrid } from '@/components/landing/BlueprintGrid'
import { CodeExample } from '@/components/landing/CodeExample'
import { ExampleTabs } from '@/components/landing/ExampleTabs'
import { FeatureCard } from '@/components/landing/FeatureCard'
import { HeroSection } from '@/components/landing/HeroSection'
import { Navbar } from '@/components/landing/Navbar'
import { Terminal, Zap, Shield, Network, Code2, Globe } from 'lucide-react'

const features = [
  {
    icon: <Code2 className="w-6 h-6" />,
    title: "TypeScript Smart Contracts",
    description: "Write contracts in familiar TypeScript with full type safety and IDE support."
  },
  {
    icon: <Shield className="w-6 h-6" />,
    title: "Sandboxed Execution",
    description: "Secure V8 isolation ensures contracts can't access system resources."
  },
  {
    icon: <Network className="w-6 h-6" />,
    title: "Multi-Currency Support",
    description: "No native token required. Create and manage any currency in your contracts."
  },
  {
    icon: <Terminal className="w-6 h-6" />,
    title: "CLI-First Design",
    description: "Everything controllable from the terminal with powerful command-line tools."
  },
  {
    icon: <Zap className="w-6 h-6" />,
    title: "High Performance",
    description: "100,000+ transactions per second with millisecond block times."
  },
  {
    icon: <Globe className="w-6 h-6" />,
    title: "User-Owned Networks",
    description: "Anyone can start their own blockchain or join existing networks."
  }
]

export default function LandingPage() {
  const [selectedExample, setSelectedExample] = useState<any>(null)
  const router = useRouter()

  return (
    <div className="min-h-screen bg-background text-foreground overflow-hidden">
      {/* Blueprint grid background */}
      <BlueprintGrid />
      
      {/* Navigation */}
      <Navbar />

      {/* Hero Section */}
      <HeroSection />

      {/* What is Tana Section */}
      <section className="relative px-4 py-16 lg:px-8 bg-secondary/30">
        <div className="mx-auto max-w-7xl">
          <div className="text-center mb-16">
            <h2 className="text-3xl lg:text-4xl font-bold text-foreground mb-4">
              What is tana?
            </h2>
            <p className="text-lg text-muted-foreground max-w-3xl mx-auto">
              Tana is a blockchain platform designed to be user-owned and operated.
              Anyone can start their own blockchain or join existing networks as a validator node.
              Smart contracts are written in familiar TypeScript and executed in a sandboxed V8 environment.
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
            {features.map((feature, index) => (
              <FeatureCard key={index} {...feature} />
            ))}
          </div>
        </div>
      </section>

      {/* Code Examples Section */}
      <section className="relative px-4 py-16 lg:px-8">
        <div className="mx-auto max-w-7xl">
          <div className="text-center mb-16">
            <h2 className="text-3xl lg:text-4xl font-bold text-foreground mb-4">
              Smart Contract Examples
            </h2>
            <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
              Explore the versatility of TypeScript smart contracts. From simple counters to complex DAOs,
              see what you can build with tana.
            </p>
          </div>

          {/* Example tabs */}
          <ExampleTabs 
            onExampleSelect={setSelectedExample}
            selectedExample={selectedExample}
          />

          {/* Selected example display */}
          {selectedExample ? (
            <div className="grid lg:grid-cols-2 gap-8 mt-12">
              <CodeExample
                title={selectedExample.title}
                code={selectedExample.code}
                result={selectedExample.result}
              />
              <Card className="bg-card border-border backdrop-blur-sm">
                <CardHeader>
                  <CardTitle className="text-foreground">Execution Result</CardTitle>
                  <CardDescription className="text-muted-foreground">
                    Pretty-printed JSON output from contract execution
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <pre className="bg-secondary/30 border border-border rounded-lg p-4 overflow-x-auto text-sm max-h-96 overflow-y-auto">
                    <code className="text-foreground">
                      {JSON.stringify(selectedExample.result, null, 2)}
                    </code>
                  </pre>
                </CardContent>
              </Card>
            </div>
          ) : (
            <div className="text-center text-muted-foreground mt-12">
              Select an example to view the code and execution result
            </div>
          )}
        </div>
      </section>

      {/* CTA Section */}
      <section className="relative px-4 py-24 lg:px-8 bg-secondary/30">
        <div className="mx-auto max-w-4xl text-center">
          <h2 className="text-3xl lg:text-4xl font-bold text-foreground mb-6">
            Ready to Build Your Blockchain?
          </h2>
          <p className="text-lg text-muted-foreground mb-8 max-w-2xl mx-auto">
            Get started with tana today and deploy your first TypeScript smart contract in minutes.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Button
              onClick={() => router.push('/signin')}
              size="lg"
              className="bg-primary hover:bg-primary/90 text-primary-foreground border border-primary/30"
            >
              Sign In to Get Started
            </Button>
            <Button
              variant="outline"
              size="lg"
              className="border-border text-foreground hover:bg-accent"
              onClick={() => window.open('https://github.com/conoda/tana', '_blank')}
            >
              View on GitHub
            </Button>
          </div>
        </div>
      </section>

    </div>
  )
}