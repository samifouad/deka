'use client'

import Link from 'next/link'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { BlueprintGrid } from '@/components/landing/BlueprintGrid'
import { Navbar } from '@/components/landing/Navbar'
import {
  Terminal,
  Server,
  Zap,
  Gauge,
  TrendingUp,
  CheckCircle2,
  Package,
  X
} from 'lucide-react'

export default function RuntimePage() {
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
                DEKA RUNTIME
              </div>
              <h1 className="text-5xl lg:text-7xl font-bold text-foreground mb-6 leading-tight">
                The fastest JavaScript runtime for production workloads.
              </h1>
              <p className="text-xl lg:text-2xl text-muted-foreground mb-8 max-w-2xl">
                World class single core performance. On another planet for multicore performance.
              </p>

              <div className="mb-8">
                <div className="inline-flex items-center gap-2 rounded-lg border border-primary/30 bg-primary/10 px-4 py-2 text-sm text-primary">
                  <Server className="w-4 h-4" />
                  <span className="font-semibold">Early Prototype</span>
                  <span className="text-primary/80">— Available Soon</span>
                </div>
              </div>

              <div className="flex flex-wrap gap-4 justify-center lg:justify-start text-sm text-muted-foreground">
                <div className="flex items-center gap-2">
                  <Server className="w-4 h-4 text-primary" />
                  <span>Multi-core isolate pooling</span>
                </div>
                <div className="flex items-center gap-2">
                  <Gauge className="w-4 h-4 text-primary" />
                  <span>2100+ RPS on commodity hardware</span>
                </div>
                <div className="flex items-center gap-2">
                  <Zap className="w-4 h-4 text-primary" />
                  <span>Sub-millisecond warm starts</span>
                </div>
              </div>
            </div>

            <div className="flex justify-center">
              <div className="w-full max-w-2xl">
                {/* Benchmark Table */}
                <div className="overflow-hidden rounded-xl border-2 border-border bg-card shadow-2xl">
                  <table className="w-full">
                    <thead>
                      <tr className="border-b border-border bg-muted/30">
                        <th className="px-4 py-3 text-left text-xs font-semibold text-foreground">Runtime</th>
                        <th className="px-4 py-3 text-right text-xs font-semibold text-foreground">RPS</th>
                        <th className="px-4 py-3 text-right text-xs font-semibold text-foreground">P50</th>
                        <th className="px-4 py-3 text-right text-xs font-semibold text-foreground">P95</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr className="border-b border-border bg-primary/5">
                        <td className="px-4 py-3 font-medium text-foreground">
                          <div className="flex items-center gap-2">
                            <span className="text-sm">deka-multi-core</span>
                            <span className="inline-flex items-center gap-1 rounded-full bg-primary/20 px-1.5 py-0.5 text-[10px] font-semibold text-primary">
                              <TrendingUp className="w-2.5 h-2.5" />
                              8x
                            </span>
                          </div>
                        </td>
                        <td className="px-4 py-3 text-right font-bold text-primary text-sm">2,126.7</td>
                        <td className="px-4 py-3 text-right font-bold text-primary text-sm">76.22 ms</td>
                        <td className="px-4 py-3 text-right font-bold text-primary text-sm">91.83 ms</td>
                      </tr>
                      <tr className="border-b border-border">
                        <td className="px-4 py-3 font-medium text-foreground text-sm">deka-single-core</td>
                        <td className="px-4 py-3 text-right text-muted-foreground text-sm">258.3</td>
                        <td className="px-4 py-3 text-right text-muted-foreground text-sm">699.99 ms</td>
                        <td className="px-4 py-3 text-right text-muted-foreground text-sm">700.01 ms</td>
                      </tr>
                      <tr>
                        <td className="px-4 py-3 font-medium text-foreground text-sm">bun</td>
                        <td className="px-4 py-3 text-right text-muted-foreground text-sm">258.3</td>
                        <td className="px-4 py-3 text-right text-muted-foreground text-sm">691.99 ms</td>
                        <td className="px-4 py-3 text-right text-muted-foreground text-sm">696 ms</td>
                      </tr>
                    </tbody>
                  </table>
                </div>
                <p className="text-center text-xs text-muted-foreground mt-4">
                  175 concurrent connections • 4ms simulated work • 20 second duration
                </p>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Benchmarks Section */}
      <section className="relative px-4 py-16 lg:px-8 bg-white dark:bg-background">
        <div className="mx-auto max-w-7xl">
          <div className="text-center mb-16">
            <div className="inline-flex items-center gap-2 rounded-full border border-primary/30 bg-primary/10 px-4 py-2 text-xs font-semibold tracking-[0.2em] text-primary mb-4">
              PERFORMANCE BENCHMARKS
            </div>
            <h2 className="text-4xl font-bold text-foreground mb-4">
              Real-world performance that scales
            </h2>
            <p className="text-lg text-muted-foreground max-w-3xl mx-auto">
              Tested with 175 concurrent connections, 4ms simulated work per request, 20 second duration on commodity hardware.
            </p>
          </div>

          {/* Benchmark Table */}
          <div className="overflow-hidden rounded-xl border border-border bg-card mb-8">
            <table className="w-full">
              <thead>
                <tr className="border-b border-border bg-muted/30">
                  <th className="px-6 py-4 text-left text-sm font-semibold text-foreground">Runtime</th>
                  <th className="px-6 py-4 text-right text-sm font-semibold text-foreground">RPS</th>
                  <th className="px-6 py-4 text-right text-sm font-semibold text-foreground">P50 Latency</th>
                  <th className="px-6 py-4 text-right text-sm font-semibold text-foreground">P95 Latency</th>
                  <th className="px-6 py-4 text-right text-sm font-semibold text-foreground">Total Requests</th>
                </tr>
              </thead>
              <tbody>
                <tr className="border-b border-border bg-primary/5">
                  <td className="px-6 py-4 font-medium text-foreground">
                    <div className="flex items-center gap-2">
                      deka-multi-core
                      <span className="inline-flex items-center gap-1 rounded-full bg-primary/20 px-2 py-0.5 text-xs font-semibold text-primary">
                        <TrendingUp className="w-3 h-3" />
                        8x
                      </span>
                    </div>
                  </td>
                  <td className="px-6 py-4 text-right font-bold text-primary">2,126.7</td>
                  <td className="px-6 py-4 text-right font-bold text-primary">76.22 ms</td>
                  <td className="px-6 py-4 text-right font-bold text-primary">91.83 ms</td>
                  <td className="px-6 py-4 text-right font-bold text-primary">42,534</td>
                </tr>
                <tr className="border-b border-border">
                  <td className="px-6 py-4 font-medium text-foreground">deka-single-core</td>
                  <td className="px-6 py-4 text-right text-muted-foreground">258.3</td>
                  <td className="px-6 py-4 text-right text-muted-foreground">699.99 ms</td>
                  <td className="px-6 py-4 text-right text-muted-foreground">700.01 ms</td>
                  <td className="px-6 py-4 text-right text-muted-foreground">5,166</td>
                </tr>
                <tr>
                  <td className="px-6 py-4 font-medium text-foreground">bun</td>
                  <td className="px-6 py-4 text-right text-muted-foreground">258.3</td>
                  <td className="px-6 py-4 text-right text-muted-foreground">691.99 ms</td>
                  <td className="px-6 py-4 text-right text-muted-foreground">696 ms</td>
                  <td className="px-6 py-4 text-right text-muted-foreground">5,166</td>
                </tr>
              </tbody>
            </table>
          </div>

          {/* Performance Highlights */}
          <div className="grid md:grid-cols-3 gap-6 mb-12">
            <Card className="bg-card border-border">
              <CardContent className="pt-6">
                <div className="flex items-center gap-3 mb-3">
                  <div className="p-2 bg-primary/10 rounded-lg">
                    <TrendingUp className="w-6 h-6 text-primary" />
                  </div>
                  <div className="text-3xl font-bold text-primary">8.2x</div>
                </div>
                <h3 className="text-lg font-semibold text-foreground mb-2">Faster Throughput</h3>
                <p className="text-sm text-muted-foreground">
                  Multi-core pooling delivers 8.2x more requests per second than single-threaded runtimes.
                </p>
              </CardContent>
            </Card>

            <Card className="bg-card border-border">
              <CardContent className="pt-6">
                <div className="flex items-center gap-3 mb-3">
                  <div className="p-2 bg-primary/10 rounded-lg">
                    <Zap className="w-6 h-6 text-primary" />
                  </div>
                  <div className="text-3xl font-bold text-primary">9.1x</div>
                </div>
                <h3 className="text-lg font-semibold text-foreground mb-2">Lower Latency</h3>
                <p className="text-sm text-muted-foreground">
                  P50 latency of 76ms vs 692ms in Bun. Isolate pooling eliminates cold start penalties.
                </p>
              </CardContent>
            </Card>

            <Card className="bg-card border-border">
              <CardContent className="pt-6">
                <div className="flex items-center gap-3 mb-3">
                  <div className="p-2 bg-primary/10 rounded-lg">
                    <CheckCircle2 className="w-6 h-6 text-primary" />
                  </div>
                  <div className="text-3xl font-bold text-primary">1.20x</div>
                </div>
                <h3 className="text-lg font-semibold text-foreground mb-2">Consistent P95</h3>
                <p className="text-sm text-muted-foreground">
                  P95/P50 ratio of only 1.20x shows predictable performance under load with minimal variance.
                </p>
              </CardContent>
            </Card>
          </div>

          {/* Key Insights */}
          <div className="bg-muted/30 border border-border rounded-xl p-6">
            <h3 className="text-lg font-semibold text-foreground mb-4">Key Insights</h3>
            <ul className="space-y-3">
              <li className="flex items-start gap-3">
                <CheckCircle2 className="w-5 h-5 text-primary mt-0.5 flex-shrink-0" />
                <span className="text-muted-foreground">
                  <strong className="text-foreground">Single-core parity:</strong> deka-single-core matches Bun's performance (258 RPS), proving V8 optimization eliminates any JSC advantage
                </span>
              </li>
              <li className="flex items-start gap-3">
                <CheckCircle2 className="w-5 h-5 text-primary mt-0.5 flex-shrink-0" />
                <span className="text-muted-foreground">
                  <strong className="text-foreground">Linear scaling:</strong> Multi-core pooling achieves 8.2x throughput with ~8 workers, demonstrating efficient CPU utilization
                </span>
              </li>
              <li className="flex items-start gap-3">
                <CheckCircle2 className="w-5 h-5 text-primary mt-0.5 flex-shrink-0" />
                <span className="text-muted-foreground">
                  <strong className="text-foreground">Predictable latency:</strong> Tight P95/P50 ratio (1.20x) indicates minimal cold starts, effective isolate reuse, and stable GC behavior
                </span>
              </li>
            </ul>
          </div>
        </div>
      </section>

      {/* Architecture Section */}
      <section className="relative px-4 py-16 lg:px-8 bg-white dark:bg-background">
        <div className="mx-auto max-w-7xl">
          <div className="text-center mb-16">
            <h2 className="text-4xl font-bold text-foreground mb-4">
              Built for production workloads
            </h2>
            <p className="text-lg text-muted-foreground max-w-3xl mx-auto">
              Deploy on bare metal, in containers, or alongside your existing orchestration. The runtime gives you low-latency services with clear operational boundaries.
            </p>
          </div>

          <div className="grid md:grid-cols-3 gap-6">
            <Card className="bg-card border-border">
              <CardContent className="pt-6">
                <Server className="w-8 h-8 text-primary mb-3" />
                <h3 className="text-lg font-bold text-foreground mb-2">Isolate pooling</h3>
                <p className="text-muted-foreground text-sm">
                  26x faster warm starts (~0.3ms) through intelligent caching and LRU eviction strategies.
                </p>
              </CardContent>
            </Card>

            <Card className="bg-card border-border">
              <CardContent className="pt-6">
                <Terminal className="w-8 h-8 text-primary mb-3" />
                <h3 className="text-lg font-bold text-foreground mb-2">Multi-threaded workers</h3>
                <p className="text-muted-foreground text-sm">
                  Thread-local isolate ownership with consistent hashing ensures maximum cache reuse.
                </p>
              </CardContent>
            </Card>

            <Card className="bg-card border-border">
              <CardContent className="pt-6">
                <Zap className="w-8 h-8 text-primary mb-3" />
                <h3 className="text-lg font-bold text-foreground mb-2">Optimized execution</h3>
                <p className="text-muted-foreground text-sm">
                  TypeScript transpilation caching, import pre-transformation, and V8 bytecode caching.
                </p>
              </CardContent>
            </Card>
          </div>
        </div>
      </section>

      {/* Framework Compatibility Section */}
      <section className="relative px-4 py-16 lg:px-8 bg-white dark:bg-background">
        <div className="mx-auto max-w-7xl">
          <div className="text-center mb-16">
            <div className="inline-flex items-center gap-2 rounded-full border border-primary/30 bg-primary/10 px-4 py-2 text-xs font-semibold tracking-[0.2em] text-primary mb-4">
              FRAMEWORK SUPPORT
            </div>
            <h2 className="text-4xl font-bold text-foreground mb-4">
              Runs your favorite frameworks
            </h2>
            <p className="text-lg text-muted-foreground max-w-3xl mx-auto">
              Validated compatibility with popular JavaScript frameworks. Deploy Next.js, Remix, Astro, and more on Deka runtime.
            </p>
          </div>

          {/* Framework Logos Grid */}
          <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-8 mb-12">
            {[
              { name: 'Next.js', logo: '▲' },
              { name: 'Remix', logo: 'R' },
              { name: 'Astro', logo: '🚀' },
              { name: 'SvelteKit', logo: 'SK' },
              { name: 'Nuxt', logo: 'N' },
              { name: 'Express', logo: 'E' },
            ].map((framework) => (
              <div key={framework.name} className="flex flex-col items-center gap-3 p-6 bg-card border border-border rounded-xl hover:border-primary/30 transition-colors">
                <div className="w-16 h-16 bg-muted rounded-lg flex items-center justify-center text-2xl">
                  {framework.logo}
                </div>
                <span className="text-sm font-medium text-foreground">{framework.name}</span>
              </div>
            ))}
          </div>

          {/* Validation Status */}
          <div className="bg-muted/30 border border-border rounded-xl p-6">
            <h3 className="text-lg font-semibold text-foreground mb-4">Framework Validation Status</h3>
            <div className="grid md:grid-cols-2 gap-4">
              {[
                { framework: 'Next.js', status: 'Validated', version: 'v14+' },
                { framework: 'Remix', status: 'Validated', version: 'v2+' },
                { framework: 'Astro', status: 'In Progress', version: 'v4+' },
                { framework: 'SvelteKit', status: 'In Progress', version: 'v2+' },
                { framework: 'Nuxt', status: 'Planned', version: 'v3+' },
                { framework: 'Express', status: 'Validated', version: 'v4+' },
              ].map((item) => (
                <div key={item.framework} className="flex items-center justify-between p-4 bg-card rounded-lg border border-border">
                  <div className="flex items-center gap-3">
                    <code className="text-foreground font-mono text-sm">{item.framework}</code>
                    <span className="text-xs text-muted-foreground">{item.version}</span>
                  </div>
                  <span className={`text-xs font-semibold px-2 py-1 rounded-full ${
                    item.status === 'Validated'
                      ? 'bg-primary/20 text-primary'
                      : item.status === 'In Progress'
                      ? 'bg-muted text-muted-foreground'
                      : 'bg-muted/50 text-muted-foreground'
                  }`}>
                    {item.status}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* Node.js Compatibility Section */}
      <section className="relative px-4 py-16 lg:px-8 bg-white dark:bg-background">
        <div className="mx-auto max-w-7xl">
          <div className="text-center mb-16">
            <div className="inline-flex items-center gap-2 rounded-full border border-primary/30 bg-primary/10 px-4 py-2 text-xs font-semibold tracking-[0.2em] text-primary mb-4">
              NODE.JS COMPATIBILITY
            </div>
            <h2 className="text-4xl font-bold text-foreground mb-4">
              Comprehensive Node.js compatibility
            </h2>
            <p className="text-lg text-muted-foreground max-w-3xl mx-auto">
              Drop-in replacement for Node.js. Deka passes <strong className="text-foreground">89.4%</strong> of Node.js compatibility tests with active development toward 100%.
            </p>
          </div>

          {/* Compatibility Comparison Table */}
          <div className="overflow-hidden rounded-xl border border-border bg-card mb-8">
            <div className="overflow-x-auto">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-border bg-muted/30">
                    <th className="px-6 py-4 text-left text-sm font-semibold text-foreground">API</th>
                    <th className="px-6 py-4 text-center text-sm font-semibold text-foreground">Deka</th>
                    <th className="px-6 py-4 text-center text-sm font-semibold text-foreground">Bun</th>
                    <th className="px-6 py-4 text-center text-sm font-semibold text-foreground">Node.js</th>
                  </tr>
                </thead>
                <tbody className="text-sm">
                  {/* Built-in Modules */}
                  <tr className="border-b border-border">
                    <td colSpan={4} className="px-6 py-3 bg-muted/20">
                      <strong className="text-foreground">Built-in Modules</strong>
                    </td>
                  </tr>
                  {[
                    { name: 'node:assert', deka: true, bun: true },
                    { name: 'node:buffer', deka: true, bun: true },
                    { name: 'node:console', deka: true, bun: true },
                    { name: 'node:crypto', deka: true, bun: true },
                    { name: 'node:events', deka: true, bun: true },
                    { name: 'node:fs', deka: true, bun: true },
                    { name: 'node:http', deka: true, bun: true },
                    { name: 'node:https', deka: true, bun: true },
                    { name: 'node:net', deka: true, bun: true },
                    { name: 'node:os', deka: true, bun: true },
                    { name: 'node:path', deka: true, bun: true },
                    { name: 'node:process', deka: true, bun: true },
                    { name: 'node:stream', deka: true, bun: true },
                    { name: 'node:timers', deka: true, bun: true },
                    { name: 'node:url', deka: true, bun: true },
                    { name: 'node:v8', deka: true, bun: false },
                    { name: 'node:zlib', deka: false, bun: true },
                    { name: 'node:inspector', deka: true, bun: false },
                    { name: 'node:sqlite', deka: true, bun: false },
                  ].map((api) => (
                    <tr key={api.name} className="border-b border-border">
                      <td className="px-6 py-3">
                        <code className="text-foreground font-mono text-xs">{api.name}</code>
                      </td>
                      <td className="px-6 py-3 text-center">
                        {api.deka ? (
                          <CheckCircle2 className="w-5 h-5 text-primary inline-block" />
                        ) : (
                          <X className="w-5 h-5 text-muted-foreground inline-block" />
                        )}
                      </td>
                      <td className="px-6 py-3 text-center">
                        {api.bun ? (
                          <CheckCircle2 className="w-5 h-5 text-muted-foreground inline-block" />
                        ) : (
                          <X className="w-5 h-5 text-red-400 inline-block" />
                        )}
                      </td>
                      <td className="px-6 py-3 text-center">
                        <CheckCircle2 className="w-5 h-5 text-muted-foreground inline-block" />
                      </td>
                    </tr>
                  ))}

                  {/* Globals */}
                  <tr className="border-b border-border">
                    <td colSpan={4} className="px-6 py-3 bg-muted/20">
                      <strong className="text-foreground">Globals</strong>
                    </td>
                  </tr>
                  {[
                    { name: 'Buffer', deka: true, bun: true },
                    { name: 'Blob', deka: true, bun: true },
                    { name: 'fetch', deka: true, bun: true },
                    { name: 'FormData', deka: true, bun: true },
                    { name: 'ReadableStream', deka: true, bun: true },
                    { name: 'WritableStream', deka: true, bun: true },
                    { name: 'TransformStream', deka: true, bun: true },
                    { name: 'require()', deka: true, bun: true },
                    { name: 'module', deka: true, bun: true },
                    { name: 'process', deka: true, bun: true },
                    { name: 'structuredClone()', deka: true, bun: true },
                    { name: 'performance', deka: true, bun: true },
                    { name: 'WebAssembly', deka: false, bun: true },
                  ].map((api) => (
                    <tr key={api.name} className="border-b border-border last:border-b-0">
                      <td className="px-6 py-3">
                        <code className="text-foreground font-mono text-xs">{api.name}</code>
                      </td>
                      <td className="px-6 py-3 text-center">
                        {api.deka ? (
                          <CheckCircle2 className="w-5 h-5 text-primary inline-block" />
                        ) : (
                          <X className="w-5 h-5 text-muted-foreground inline-block" />
                        )}
                      </td>
                      <td className="px-6 py-3 text-center">
                        {api.bun ? (
                          <CheckCircle2 className="w-5 h-5 text-muted-foreground inline-block" />
                        ) : (
                          <X className="w-5 h-5 text-red-400 inline-block" />
                        )}
                      </td>
                      <td className="px-6 py-3 text-center">
                        <CheckCircle2 className="w-5 h-5 text-muted-foreground inline-block" />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>

          {/* Summary Stats */}
          <div className="grid md:grid-cols-3 gap-6 mb-8">
            <Card className="bg-card border-border">
              <CardContent className="pt-6">
                <div className="flex items-center gap-3 mb-3">
                  <div className="p-2 bg-primary/10 rounded-lg">
                    <Package className="w-6 h-6 text-primary" />
                  </div>
                  <div className="text-3xl font-bold text-primary">89.4%</div>
                </div>
                <h3 className="text-lg font-semibold text-foreground mb-2">Deka Compatibility</h3>
                <p className="text-sm text-muted-foreground">
                  101 of 113 Node.js APIs passing in automated compatibility tests.
                </p>
              </CardContent>
            </Card>

            <Card className="bg-card border-border">
              <CardContent className="pt-6">
                <div className="flex items-center gap-3 mb-3">
                  <div className="p-2 bg-primary/10 rounded-lg">
                    <CheckCircle2 className="w-6 h-6 text-primary" />
                  </div>
                  <div className="text-3xl font-bold text-primary">113</div>
                </div>
                <h3 className="text-lg font-semibold text-foreground mb-2">Total APIs Tested</h3>
                <p className="text-sm text-muted-foreground">
                  All built-in Node.js modules and globals tested against compatibility suite.
                </p>
              </CardContent>
            </Card>

            <Card className="bg-card border-border">
              <CardContent className="pt-6">
                <div className="flex items-center gap-3 mb-3">
                  <div className="p-2 bg-primary/10 rounded-lg">
                    <TrendingUp className="w-6 h-6 text-primary" />
                  </div>
                  <div className="text-3xl font-bold text-primary">Daily</div>
                </div>
                <h3 className="text-lg font-semibold text-foreground mb-2">Active Development</h3>
                <p className="text-sm text-muted-foreground">
                  New APIs added daily with goal of 100% Node.js compatibility.
                </p>
              </CardContent>
            </Card>
          </div>

          {/* Disclaimer */}
          <div className="bg-muted/30 border border-border rounded-xl p-6 mb-8">
            <p className="text-sm text-muted-foreground">
              <strong className="text-foreground">Test Methodology:</strong> Deka results from internal automated compatibility tests.
              Bun results based on official documentation (<a href="https://bun.com/docs/runtime/nodejs-compat.md" className="text-primary hover:underline" target="_blank" rel="noopener noreferrer">bun.com/docs/runtime/nodejs-compat.md</a>).
              All Node.js APIs are 100% compatible with Node.js by definition.
            </p>
          </div>

          {/* Full compatibility table link */}
          <div className="text-center">
            <p className="text-sm text-muted-foreground mb-4">
              View the complete compatibility test results including all 113 Node.js APIs
            </p>
            <Button variant="outline" size="lg" className="border-border text-foreground hover:bg-accent" asChild>
              <a href="https://github.com/deka/deka-runtime/blob/main/test/compat/REPORT.md" target="_blank" rel="noopener noreferrer">
                View Full Test Report
              </a>
            </Button>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="relative px-4 py-24 lg:px-8 bg-white dark:bg-background">
        <div className="mx-auto max-w-4xl text-center">
          <h2 className="text-4xl lg:text-5xl font-bold text-foreground mb-6">
            Coming Soon: The runtime that outperforms everything.
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
            <strong className="text-foreground">Early Prototype:</strong> Performance benchmarks shown are from internal testing. Public release coming soon.
          </p>
        </div>
      </section>
    </div>
  )
}
