'use client'

import { AnimatedTerminal } from './AnimatedTerminal'
import { AnimatedTerminalWithScreenshot } from './AnimatedTerminalWithScreenshot'
import { Button } from '@/components/ui/button'

export function HeroSection() {
  return (
    <section className="relative px-4 py-24 lg:px-8">
      {/* Enhanced background for hero */}
      <div className="absolute inset-0 bg-gradient-to-br from-primary/10 via-background/20 to-primary/5 backdrop-blur-sm"></div>

      <div className="relative mx-auto max-w-7xl">
        <div className="grid lg:grid-cols-2 gap-12 items-center">
          {/* Left side - Text content */}
          <div className="text-center lg:text-left">
            <div className="mb-8">
              <h1 className="text-5xl lg:text-7xl font-bold text-foreground mb-6 leading-tight">
                tana
              </h1>
              <p className="text-xl lg:text-2xl text-foreground mb-4 max-w-2xl">
                User-owned blockchain with TypeScript smart contracts
              </p>
              <p className="text-lg text-muted-foreground max-w-xl mb-8">
                No native token required. Multi-currency support. CLI-first design. Build decentralized applications with tools you already know.
              </p>
            </div>

            <div className="flex flex-col sm:flex-row gap-4 justify-center lg:justify-start mb-12">
              <Button size="lg" className="bg-primary hover:bg-primary/90 text-primary-foreground font-bold border border-primary/30 shadow-lg shadow-primary/20">
                Get Started
              </Button>
              <Button variant="outline" size="lg" className="border-border text-foreground hover:bg-accent">
                View Documentation
              </Button>
            </div>

            {/* Key features pills */}
            <div className="flex flex-wrap gap-2 justify-center lg:justify-start">
              <span className="px-3 py-1 bg-secondary border border-border rounded-full text-secondary-foreground text-sm backdrop-blur-sm">
                TypeScript
              </span>
              <span className="px-3 py-1 bg-secondary border border-border rounded-full text-secondary-foreground text-sm backdrop-blur-sm">
                Multi-Currency
              </span>
              <span className="px-3 py-1 bg-secondary border border-border rounded-full text-secondary-foreground text-sm backdrop-blur-sm">
                CLI First
              </span>
              <span className="px-3 py-1 bg-secondary border border-border rounded-full text-secondary-foreground text-sm backdrop-blur-sm">
                User-Owned
              </span>
            </div>
          </div>

          {/* Right side - Animated terminal */}
          <div className="lg:pl-8">
            <AnimatedTerminal />
          </div>
        </div>

        {/* Additional terminal examples */}
        <div className="mt-24 space-y-16">
          {/* TUI */}
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <div className="order-1">
              <AnimatedTerminalWithScreenshot
                command="tana start tui"
                screenshotAlt="Tana TUI Interface"
                align="left"
                // Optional timing controls (all in milliseconds):
                // typingSpeed={50}              // Time per character (default: 50ms)
                // pauseBeforeScreenshot={3000}  // Pause after typing (default: 3000ms)
                // screenshotDuration={5000}     // How long to show screenshot (default: 5000ms)
              />
            </div>
            <div className="text-center lg:text-right order-2">
              <h3 className="text-2xl lg:text-3xl font-bold text-foreground mb-4">
                Terminal User Interface
              </h3>
              <p className="text-lg text-muted-foreground">
                Interactive TUI for monitoring your blockchain with real-time updates and controls.
              </p>
            </div>
          </div>

          {/* WebUI */}
          <div className="grid lg:grid-cols-2 gap-12 items-center">
            <div className="text-center lg:text-left order-2 lg:order-1">
              <h3 className="text-2xl lg:text-3xl font-bold text-foreground mb-4">
                Web User Interface
              </h3>
              <p className="text-lg text-muted-foreground">
                Launch the web-based dashboard to manage your blockchain from your browser.
              </p>
            </div>
            <div className="order-1 lg:order-2">
              <AnimatedTerminalWithScreenshot
                command="tana start webui"
                screenshotAlt="Tana Web Interface"
                align="right"
              />
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}