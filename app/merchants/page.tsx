'use client'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { BlueprintGrid } from '@/components/landing/BlueprintGrid'
import { Navbar } from '@/components/landing/Navbar'
import {
  Store,
  DollarSign,
  Package,
  Users,
  MessageSquare,
  BarChart3,
  Shield,
  Zap,
  CheckCircle2,
  X,
  Sparkles
} from 'lucide-react'

export default function MerchantsPage() {

  return (
    <div className="min-h-screen bg-background text-foreground overflow-hidden">
      <BlueprintGrid />
      <Navbar />

      {/* Hero Section */}
      <section className="relative px-4 py-24 lg:px-8">
        <div className="absolute inset-0 bg-gradient-to-br from-primary/10 via-background/20 to-primary/5 backdrop-blur-sm"></div>

        <div className="relative mx-auto max-w-7xl text-center">
          <div className="inline-block bg-primary/10 rounded-full px-4 py-2 mb-6">
            <span className="text-primary font-semibold text-sm">FOR BUSINESSES</span>
          </div>

          <h1 className="text-5xl lg:text-7xl font-bold text-foreground mb-6 leading-tight">
            Launch Your Store.<br/>Keep Every Dollar.
          </h1>

          <p className="text-xl lg:text-2xl text-muted-foreground mb-8 max-w-3xl mx-auto">
            Get more features than Shopify's paid plans. Completely free. Zero transaction fees. No per-user billing. Built for businesses that want to grow without limits.
          </p>

          <div className="flex flex-col sm:flex-row gap-4 justify-center mb-12">
            <Button size="lg" className="bg-primary hover:bg-primary/90 text-primary-foreground font-bold">
              Start Free Store
            </Button>
            <Button variant="outline" size="lg" className="border-border text-foreground hover:bg-accent">
              Watch Demo
            </Button>
          </div>

          {/* Trust indicators */}
          <div className="flex flex-wrap gap-6 justify-center text-sm text-muted-foreground">
            <div className="flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4 text-primary" />
              <span>Free forever</span>
            </div>
            <div className="flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4 text-primary" />
              <span>No transaction fees</span>
            </div>
            <div className="flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4 text-primary" />
              <span>Unlimited products</span>
            </div>
            <div className="flex items-center gap-2">
              <CheckCircle2 className="w-4 h-4 text-primary" />
              <span>No credit card required</span>
            </div>
          </div>
        </div>
      </section>

      {/* Comparison Table */}
      <section className="relative px-4 py-16 lg:px-8 bg-secondary/30">
        <div className="mx-auto max-w-6xl">
          <div className="text-center mb-12">
            <h2 className="text-4xl font-bold text-foreground mb-4">
              tana vs. The Competition
            </h2>
            <p className="text-lg text-muted-foreground">
              See why thousands of merchants are switching from expensive platforms
            </p>
          </div>

          <div className="bg-card border border-border rounded-lg overflow-hidden backdrop-blur-sm">
            <table className="w-full">
              <thead>
                <tr className="border-b border-border">
                  <th className="text-left p-4 font-semibold text-foreground">Feature</th>
                  <th className="text-center p-4 font-semibold text-primary bg-primary/5">tana</th>
                  <th className="text-center p-4 font-semibold text-muted-foreground">Shopify</th>
                  <th className="text-center p-4 font-semibold text-muted-foreground">Square</th>
                </tr>
              </thead>
              <tbody>
                <tr className="border-b border-border/50">
                  <td className="p-4 text-muted-foreground">Monthly Fee</td>
                  <td className="text-center p-4 bg-primary/5 font-bold text-primary">$0</td>
                  <td className="text-center p-4 text-muted-foreground">$29-299</td>
                  <td className="text-center p-4 text-muted-foreground">$0-60</td>
                </tr>
                <tr className="border-b border-border/50">
                  <td className="p-4 text-muted-foreground">Transaction Fees</td>
                  <td className="text-center p-4 bg-primary/5 font-bold text-primary">0%</td>
                  <td className="text-center p-4 text-muted-foreground">2.9% + 30¢</td>
                  <td className="text-center p-4 text-muted-foreground">2.6% + 10¢</td>
                </tr>
                <tr className="border-b border-border/50">
                  <td className="p-4 text-muted-foreground">Withdrawal Fee</td>
                  <td className="text-center p-4 bg-primary/5 font-bold text-primary">One-time only</td>
                  <td className="text-center p-4 text-muted-foreground">Per transaction</td>
                  <td className="text-center p-4 text-muted-foreground">Per transaction</td>
                </tr>
                <tr className="border-b border-border/50">
                  <td className="p-4 text-muted-foreground">Unlimited Products</td>
                  <td className="text-center p-4 bg-primary/5"><CheckCircle2 className="w-5 h-5 text-primary mx-auto" /></td>
                  <td className="text-center p-4"><CheckCircle2 className="w-5 h-5 text-muted-foreground mx-auto" /></td>
                  <td className="text-center p-4"><CheckCircle2 className="w-5 h-5 text-muted-foreground mx-auto" /></td>
                </tr>
                <tr className="border-b border-border/50">
                  <td className="p-4 text-muted-foreground">Inventory Management</td>
                  <td className="text-center p-4 bg-primary/5"><CheckCircle2 className="w-5 h-5 text-primary mx-auto" /></td>
                  <td className="text-center p-4"><CheckCircle2 className="w-5 h-5 text-muted-foreground mx-auto" /></td>
                  <td className="text-center p-4"><CheckCircle2 className="w-5 h-5 text-muted-foreground mx-auto" /></td>
                </tr>
                <tr className="border-b border-border/50">
                  <td className="p-4 text-muted-foreground">Team Chat Built-in</td>
                  <td className="text-center p-4 bg-primary/5"><CheckCircle2 className="w-5 h-5 text-primary mx-auto" /></td>
                  <td className="text-center p-4"><X className="w-5 h-5 text-muted-foreground/30 mx-auto" /></td>
                  <td className="text-center p-4"><X className="w-5 h-5 text-muted-foreground/30 mx-auto" /></td>
                </tr>
                <tr className="border-b border-border/50">
                  <td className="p-4 text-muted-foreground">Customer Service Tools</td>
                  <td className="text-center p-4 bg-primary/5"><CheckCircle2 className="w-5 h-5 text-primary mx-auto" /></td>
                  <td className="text-center p-4 text-muted-foreground text-sm">Add-on</td>
                  <td className="text-center p-4 text-muted-foreground text-sm">Add-on</td>
                </tr>
                <tr className="border-b border-border/50">
                  <td className="p-4 text-muted-foreground">Per-User Billing</td>
                  <td className="text-center p-4 bg-primary/5"><X className="w-5 h-5 text-primary mx-auto" /></td>
                  <td className="text-center p-4"><CheckCircle2 className="w-5 h-5 text-muted-foreground/30 mx-auto" /></td>
                  <td className="text-center p-4"><CheckCircle2 className="w-5 h-5 text-muted-foreground/30 mx-auto" /></td>
                </tr>
                <tr>
                  <td className="p-4 text-muted-foreground">Advanced Analytics</td>
                  <td className="text-center p-4 bg-primary/5"><CheckCircle2 className="w-5 h-5 text-primary mx-auto" /></td>
                  <td className="text-center p-4 text-muted-foreground text-sm">$299/mo</td>
                  <td className="text-center p-4 text-muted-foreground text-sm">$60/mo</td>
                </tr>
              </tbody>
            </table>
          </div>

          <div className="text-center mt-8">
            <p className="text-muted-foreground text-sm">
              Save an average of <span className="text-primary font-bold">$4,200/year</span> by switching to tana
            </p>
          </div>
        </div>
      </section>

      {/* Key Features Grid */}
      <section className="relative px-4 py-16 lg:px-8">
        <div className="mx-auto max-w-7xl">
          <div className="text-center mb-16">
            <h2 className="text-4xl font-bold text-foreground mb-4">
              Everything you need. Nothing you don't.
            </h2>
            <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
              Run your entire business from one platform. No expensive add-ons or monthly fees.
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
            <Card className="bg-card border-border backdrop-blur-sm">
              <CardHeader>
                <div className="w-12 h-12 bg-primary/10 rounded-lg flex items-center justify-center mb-3">
                  <Store className="w-6 h-6 text-primary" />
                </div>
                <CardTitle className="text-foreground">Complete Storefront</CardTitle>
                <CardDescription className="text-muted-foreground">
                  Beautiful, customizable store that works on any device. No coding required.
                </CardDescription>
              </CardHeader>
            </Card>

            <Card className="bg-card border-border backdrop-blur-sm">
              <CardHeader>
                <div className="w-12 h-12 bg-primary/10 rounded-lg flex items-center justify-center mb-3">
                  <DollarSign className="w-6 h-6 text-primary" />
                </div>
                <CardTitle className="text-foreground">Zero Transaction Fees</CardTitle>
                <CardDescription className="text-muted-foreground">
                  Keep 100% of every sale. Only pay a small one-time fee when you withdraw funds.
                </CardDescription>
              </CardHeader>
            </Card>

            <Card className="bg-card border-border backdrop-blur-sm">
              <CardHeader>
                <div className="w-12 h-12 bg-primary/10 rounded-lg flex items-center justify-center mb-3">
                  <Package className="w-6 h-6 text-primary" />
                </div>
                <CardTitle className="text-foreground">Inventory Management</CardTitle>
                <CardDescription className="text-muted-foreground">
                  Track stock levels, manage variants, set up automatic alerts. All built-in.
                </CardDescription>
              </CardHeader>
            </Card>

            <Card className="bg-card border-border backdrop-blur-sm">
              <CardHeader>
                <div className="w-12 h-12 bg-primary/10 rounded-lg flex items-center justify-center mb-3">
                  <Users className="w-6 h-6 text-primary" />
                </div>
                <CardTitle className="text-foreground">Customer Service Hub</CardTitle>
                <CardDescription className="text-muted-foreground">
                  Manage customer inquiries, track support tickets, and maintain relationships.
                </CardDescription>
              </CardHeader>
            </Card>

            <Card className="bg-card border-border backdrop-blur-sm">
              <CardHeader>
                <div className="w-12 h-12 bg-primary/10 rounded-lg flex items-center justify-center mb-3">
                  <MessageSquare className="w-6 h-6 text-primary" />
                </div>
                <CardTitle className="text-foreground">Team Chat</CardTitle>
                <CardDescription className="text-muted-foreground">
                  Slack-like team communication built right in. No need for separate tools.
                </CardDescription>
              </CardHeader>
            </Card>

            <Card className="bg-card border-border backdrop-blur-sm">
              <CardHeader>
                <div className="w-12 h-12 bg-primary/10 rounded-lg flex items-center justify-center mb-3">
                  <BarChart3 className="w-6 h-6 text-primary" />
                </div>
                <CardTitle className="text-foreground">Advanced Analytics</CardTitle>
                <CardDescription className="text-muted-foreground">
                  Real-time sales data, customer insights, and revenue forecasting. Included free.
                </CardDescription>
              </CardHeader>
            </Card>

            <Card className="bg-card border-border backdrop-blur-sm">
              <CardHeader>
                <div className="w-12 h-12 bg-primary/10 rounded-lg flex items-center justify-center mb-3">
                  <Shield className="w-6 h-6 text-primary" />
                </div>
                <CardTitle className="text-foreground">Fraud Protection</CardTitle>
                <CardDescription className="text-muted-foreground">
                  Enterprise-grade security keeps your business and customers safe.
                </CardDescription>
              </CardHeader>
            </Card>

            <Card className="bg-card border-border backdrop-blur-sm">
              <CardHeader>
                <div className="w-12 h-12 bg-primary/10 rounded-lg flex items-center justify-center mb-3">
                  <Sparkles className="w-6 h-6 text-primary" />
                </div>
                <CardTitle className="text-foreground">Paid Modules</CardTitle>
                <CardDescription className="text-muted-foreground">
                  Optional premium features available. Only pay for what you need, when you need it.
                </CardDescription>
              </CardHeader>
            </Card>

            <Card className="bg-card border-border backdrop-blur-sm">
              <CardHeader>
                <div className="w-12 h-12 bg-primary/10 rounded-lg flex items-center justify-center mb-3">
                  <Users className="w-6 h-6 text-primary" />
                </div>
                <CardTitle className="text-foreground">Unlimited Team Members</CardTitle>
                <CardDescription className="text-muted-foreground">
                  Add your entire team. No per-user fees. Ever.
                </CardDescription>
              </CardHeader>
            </Card>
          </div>
        </div>
      </section>

      {/* How It Works */}
      <section className="relative px-4 py-16 lg:px-8 bg-secondary/30">
        <div className="mx-auto max-w-7xl">
          <div className="text-center mb-16">
            <h2 className="text-4xl font-bold text-foreground mb-4">
              Launch in minutes, not months
            </h2>
            <p className="text-lg text-muted-foreground max-w-2xl mx-auto">
              Get your store online faster than ordering lunch
            </p>
          </div>

          <div className="grid md:grid-cols-4 gap-8">
            <div className="text-center">
              <div className="w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center mx-auto mb-4 text-2xl font-bold text-primary">
                1
              </div>
              <h3 className="text-xl font-bold text-foreground mb-2">Sign Up Free</h3>
              <p className="text-muted-foreground">
                No credit card needed. Just your email and you're in.
              </p>
            </div>

            <div className="text-center">
              <div className="w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center mx-auto mb-4 text-2xl font-bold text-primary">
                2
              </div>
              <h3 className="text-xl font-bold text-foreground mb-2">Add Products</h3>
              <p className="text-muted-foreground">
                Upload your inventory with our simple bulk import tools.
              </p>
            </div>

            <div className="text-center">
              <div className="w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center mx-auto mb-4 text-2xl font-bold text-primary">
                3
              </div>
              <h3 className="text-xl font-bold text-foreground mb-2">Customize Design</h3>
              <p className="text-muted-foreground">
                Pick a theme or design your own. Make it yours in minutes.
              </p>
            </div>

            <div className="text-center">
              <div className="w-16 h-16 bg-primary/10 rounded-full flex items-center justify-center mx-auto mb-4 text-2xl font-bold text-primary">
                4
              </div>
              <h3 className="text-xl font-bold text-foreground mb-2">Start Selling</h3>
              <p className="text-muted-foreground">
                Go live and start accepting payments immediately.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Testimonials / Stats */}
      <section className="relative px-4 py-16 lg:px-8">
        <div className="mx-auto max-w-7xl">
          <div className="text-center mb-16">
            <h2 className="text-4xl font-bold text-foreground mb-4">
              Trusted by merchants worldwide
            </h2>
          </div>

          <div className="grid md:grid-cols-3 gap-8 mb-16">
            <Card className="bg-card border-border backdrop-blur-sm text-center">
              <CardContent className="pt-8">
                <div className="text-4xl font-bold text-primary mb-2">$0</div>
                <p className="text-muted-foreground">Average monthly fees saved</p>
              </CardContent>
            </Card>

            <Card className="bg-card border-border backdrop-blur-sm text-center">
              <CardContent className="pt-8">
                <div className="text-4xl font-bold text-primary mb-2">2.5%</div>
                <p className="text-muted-foreground">Average revenue increase from lower fees</p>
              </CardContent>
            </Card>

            <Card className="bg-card border-border backdrop-blur-sm text-center">
              <CardContent className="pt-8">
                <div className="text-4xl font-bold text-primary mb-2">15min</div>
                <p className="text-muted-foreground">Average time to launch a store</p>
              </CardContent>
            </Card>
          </div>

          {/* Mock testimonial */}
          <div className="max-w-3xl mx-auto">
            <Card className="bg-card border-border backdrop-blur-sm">
              <CardContent className="pt-8">
                <p className="text-lg text-muted-foreground mb-6 italic">
                  "We were paying Shopify $299/month plus 2.9% on every transaction. With tana, we pay nothing. Same features, zero fees. It's honestly ridiculous that this is free."
                </p>
                <div className="flex items-center gap-4">
                  <div className="w-12 h-12 bg-primary/10 rounded-full flex items-center justify-center">
                    <Users className="w-6 h-6 text-primary" />
                  </div>
                  <div>
                    <p className="font-semibold text-foreground">Sarah Chen</p>
                    <p className="text-sm text-muted-foreground">Owner, Artisan Coffee Co.</p>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="relative px-4 py-24 lg:px-8 bg-secondary/30">
        <div className="mx-auto max-w-4xl text-center">
          <h2 className="text-4xl lg:text-5xl font-bold text-foreground mb-6">
            Ready to keep 100% of your revenue?
          </h2>
          <p className="text-lg text-muted-foreground mb-8 max-w-2xl mx-auto">
            Join thousands of merchants who've ditched expensive platforms. Launch your free store today.
          </p>
          <div className="flex flex-col sm:flex-row gap-4 justify-center">
            <Button size="lg" className="bg-primary hover:bg-primary/90 text-primary-foreground font-bold">
              Start Free Store
            </Button>
            <Button variant="outline" size="lg" className="border-border text-foreground hover:bg-accent">
              Schedule a Demo
            </Button>
          </div>
          <p className="text-sm text-muted-foreground mt-6">
            No credit card required • Cancel anytime • Migrate from any platform
          </p>
        </div>
      </section>

    </div>
  )
}
