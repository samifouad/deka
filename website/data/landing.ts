export type FeatureCard = {
  label: string
  title: string
  description: string
}

export type ExpandableCardPoint = FeatureCard

export type ExpandableCard = FeatureCard & {
  longDescription: string
  points: ExpandableCardPoint[]
}

export type FeatureSection = {
  id: string
  eyebrow: string
  title: string
  description: string
  cards: FeatureCard[]
}

export type DetailSection = {
  title: string
  description: string
  cards: FeatureCard[]
}

export type DetailPage = {
  slug: string
  title: string
  subtitle: string
  summary: string
  highlights: string[]
  sections: DetailSection[]
}

export const featureSections: FeatureSection[] = [
  {
    id: 'serve',
    eyebrow: 'Serve',
    title: 'Serve fast, everywhere.',
    description:
      'Deploy edge-grade runtimes with consistent latency and the isolation you want for production traffic.',
    cards: [
      {
        label: 'Global routing',
        title: 'Request paths tuned for proximity.',
        description: 'Edge-aware routing with automatic locality and zero extra configuration.'
      },
      {
        label: 'Cold start control',
        title: 'Warm isolates, always ready.',
        description: 'Reuse cached runtimes with predictable p95s even under bursts.'
      }
    ]
  },
  {
    id: 'run',
    eyebrow: 'Run',
    title: 'Run with clear operational boundaries.',
    description:
      'Thread-safe execution, predictable resource budgets, and a runtime that stays fast under pressure.',
    cards: [
      {
        label: 'Isolate pooling',
        title: 'Multi-core, without the complexity.',
        description: 'Worker orchestration that keeps caches hot while scaling across cores.'
      },
      {
        label: 'Observability',
        title: 'Metrics that fit your workflow.',
        description: 'Structured logs and runtime counters that surface what matters.'
      }
    ]
  },
  {
    id: 'build',
    eyebrow: 'Build',
    title: 'Build the app you want to ship.',
    description:
      'A runtime that respects your toolchain, plus a build flow that stays close to modern frameworks.',
    cards: [
      {
        label: 'Framework ready',
        title: 'Next, Remix, Astro and more.',
        description: 'Launch with the frameworks your team already trusts.'
      },
      {
        label: 'TypeScript first',
        title: 'Typed builds, fast refresh.',
        description: 'Compile cleanly with minimal overhead and stable outputs.'
      }
    ]
  },
  {
    id: 'compile',
    eyebrow: 'Compile',
    title: 'Compile for speed, ship as one.',
    description:
      'Turn projects into lean artifacts with embedded assets and tuned runtime hints.',
    cards: [
      {
        label: 'Single binary',
        title: 'One artifact per release.',
        description: 'Bundle code, assets, and config into a cohesive runtime target.'
      },
      {
        label: 'Fast boot',
        title: 'Instant startup paths.',
        description: 'Precompute modules and skip redundant startup work.'
      }
    ]
  },
  {
    id: 'deploy',
    eyebrow: 'Deploy',
    title: 'Deploy with confidence.',
    description:
      'A release model designed for predictability: simple rollouts, fast restores, steady performance.',
    cards: [
      {
        label: 'Safe rollouts',
        title: 'Ship with guardrails.',
        description: 'Gradual deploys with clear health signals and instant rollback.'
      },
      {
        label: 'Secure by default',
        title: 'Runtime isolation in every layer.',
        description: 'Guardrails baked into the runtime and the edge.'
      }
    ]
  }
]

export const detailPages: Record<string, DetailPage> = {
  serve: {
    slug: 'serve',
    title: 'Serve',
    subtitle: 'Deliver production traffic with confidence.',
    summary:
      'Deka keeps request paths short and latency predictable. Your services stay warm, edge-aware, and tuned for the workloads you actually ship.',
    highlights: [
      'Edge-aware routing with locality by default',
      'Cache-warm isolates that keep p95s stable',
      'Traffic shaping for bursts and rollouts'
    ],
    sections: [
      {
        title: 'Traffic stays close to users.',
        description:
          'Serve routes are optimized for region proximity and cache reuse. Deka minimizes cold paths so performance feels consistent under load.',
        cards: [
          {
            label: 'Routing',
            title: 'Automatic locality profiles.',
            description: 'Route to the closest available capacity without hand-tuned rules.'
          },
          {
            label: 'Caching',
            title: 'Isolates stay hot.',
            description: 'Pre-warm and reuse runtime instances for predictable latency.'
          }
        ]
      },
      {
        title: 'Serve at scale without spikes.',
        description:
          'Concurrency stays smooth with built-in controls for queueing, backpressure, and runtime reuse.',
        cards: [
          {
            label: 'Burst handling',
            title: 'Graceful ramp-up.',
            description: 'Smooth throughput under sudden traffic surges.'
          },
          {
            label: 'Control planes',
            title: 'Operational clarity.',
            description: 'Service-level metrics without extra dashboards.'
          }
        ]
      }
    ]
  },
  run: {
    slug: 'run',
    title: 'Run',
    subtitle: 'Keep runtime behavior stable at scale.',
    summary:
      'Deka prioritizes deterministic execution with clear resource budgets. Run multi-core workloads without overhead or guesswork.',
    highlights: [
      'Thread-aware isolation with stable memory',
      'Predictable budgets for CPU and latency',
      'Instrumentation that stays out of the way'
    ],
    sections: [
      {
        title: 'Isolation you can reason about.',
        description:
          'Keep concurrency safe with isolate pooling and predictable memory caps. Runtime paths stay consistent, even under pressure.',
        cards: [
          {
            label: 'Isolation',
            title: 'Dedicated runtimes per request class.',
            description: 'Keep tenant boundaries clear and secure.'
          },
          {
            label: 'Runtime budgets',
            title: 'Cap what matters.',
            description: 'Protect latency-sensitive services with tight budgets.'
          }
        ]
      },
      {
        title: 'Operations without the overhead.',
        description:
          'Metrics, logs, and runtime diagnostics are built-in and tuned for high-signal reporting.',
        cards: [
          {
            label: 'Observability',
            title: 'Telemetry that fits your stack.',
            description: 'Stream structured events directly into your systems.'
          },
          {
            label: 'Diagnostics',
            title: 'Know why latency moves.',
            description: 'Runtime insight with minimal sampling overhead.'
          }
        ]
      }
    ]
  },
  build: {
    slug: 'build',
    title: 'Build',
    subtitle: 'Ship with the tools your team already uses.',
    summary:
      'Stay close to modern frameworks and TypeScript defaults. Build flows remain familiar while Deka handles runtime optimization.',
    highlights: [
      'Framework-ready without special adapters',
      'TypeScript-first build pipeline',
      'Fast local iteration with stable output'
    ],
    sections: [
      {
        title: 'Framework-first workflows.',
        description:
          'Deploy Next.js, Remix, Astro, and more with minimal translation. Keep your tooling intact while Deka compiles for speed.',
        cards: [
          {
            label: 'Frameworks',
            title: 'Keep your stack.',
            description: 'Build with what your team already knows.'
          },
          {
            label: 'Build output',
            title: 'Predictable artifacts.',
            description: 'Stable outputs across staging and production.'
          }
        ]
      },
      {
        title: 'Fast iteration loops.',
        description:
          'Skip redundant work with cached pipelines, smart invalidation, and fast rebuilds.',
        cards: [
          {
            label: 'TypeScript',
            title: 'Zero-waste compilation.',
            description: 'Only recompile what changes.'
          },
          {
            label: 'Developer velocity',
            title: 'Shorten feedback cycles.',
            description: 'Tight loops without bespoke tooling.'
          }
        ]
      }
    ]
  },
  compile: {
    slug: 'compile',
    title: 'Compile',
    subtitle: 'Lean artifacts, fast startup.',
    summary:
      'Compile into single artifacts with embedded assets and tuned runtime hints. Ship faster and simplify release management.',
    highlights: [
      'Single binary release artifacts',
      'Precomputed module caches',
      'Tuned startup paths for low-latency boot'
    ],
    sections: [
      {
        title: 'Release-ready outputs.',
        description:
          'Bundle app code, runtime config, and assets into a single artifact designed for quick shipping.',
        cards: [
          {
            label: 'Packaging',
            title: 'Everything in one release.',
            description: 'Simplify release pipelines and reduce moving parts.'
          },
          {
            label: 'Assets',
            title: 'Embedded and optimized.',
            description: 'Serve static assets without external dependencies.'
          }
        ]
      },
      {
        title: 'Startup speed without compromise.',
        description:
          'Preload modules and cache hot paths. Cut time-to-first-request across deployments.',
        cards: [
          {
            label: 'Startup',
            title: 'Instant boot paths.',
            description: 'Skip redundant runtime setup.'
          },
          {
            label: 'Runtime hints',
            title: 'Smarter initialization.',
            description: 'Guide the runtime toward the right entry points.'
          }
        ]
      }
    ]
  },
  deploy: {
    slug: 'deploy',
    title: 'Deploy',
    subtitle: 'Release confidently and recover quickly.',
    summary:
      'Deka deployments are built around predictable rollouts, quick restores, and clear insight into runtime health.',
    highlights: [
      'Safe rollouts with guardrails',
      'Instant restores and traffic shifts',
      'Security baked into every layer'
    ],
    sections: [
      {
        title: 'Guardrails for every release.',
        description:
          'Incremental rollouts and clear health signals keep releases safe across environments.',
        cards: [
          {
            label: 'Release safety',
            title: 'Gradual traffic shifts.',
            description: 'Roll out without surprises or manual routing.'
          },
          {
            label: 'Recovery',
            title: 'Instant rollback paths.',
            description: 'Restore previous versions in seconds.'
          }
        ]
      },
      {
        title: 'Secure by default.',
        description:
          'Runtime isolation, audit-ready logs, and hardened defaults protect production workloads.',
        cards: [
          {
            label: 'Isolation',
            title: 'Workload boundaries baked in.',
            description: 'Protect services from noisy neighbors.'
          },
          {
            label: 'Compliance',
            title: 'Audit-friendly by design.',
            description: 'Structured logging and access controls out of the box.'
          }
        ]
      }
    ]
  },
  create: {
    slug: 'create',
    title: 'Create',
    subtitle: 'Spin up a new Deka project with defaults that ship.',
    summary:
      'Create gets you a ready-to-run workspace with runtime-friendly defaults for config, routing, and deployment. Customize when you are ready.',
    highlights: [
      'Starter templates for common stacks',
      'Runtime defaults wired in',
      'Clear upgrade path to production'
    ],
    sections: [
      {
        title: 'Start with a clean base.',
        description:
          'Choose a template and get structure for routes, services, and runtime configuration.',
        cards: [
          {
            label: 'Templates',
            title: 'Prebuilt layouts.',
            description: 'Select a starter that matches how your team ships.'
          },
          {
            label: 'Structure',
            title: 'Organized from day one.',
            description: 'Consistent project layout for fast onboarding.'
          }
        ]
      },
      {
        title: 'Stay aligned with the runtime.',
        description:
          'Generated projects keep build and deploy settings close to the runtime so handoff is smooth.',
        cards: [
          {
            label: 'Config',
            title: 'Runtime-ready defaults.',
            description: 'Sane settings you can extend later.'
          },
          {
            label: 'Local dev',
            title: 'Fast iteration loops.',
            description: 'Run and test without extra wiring.'
          }
        ]
      }
    ]
  },
  introspect: {
    slug: 'introspect',
    title: 'Introspect',
    subtitle: 'See what the runtime is doing, instantly.',
    summary:
      'Introspect surfaces runtime behavior, isolate health, and request flow without extra tooling. Get answers fast without noisy logs.',
    highlights: [
      'Live isolate and request visibility',
      'Resource budgets and limits at a glance',
      'High-signal diagnostics with low overhead'
    ],
    sections: [
      {
        title: 'Runtime visibility that stays fast.',
        description:
          'Inspect isolates, memory, and scheduling while services run so you can spot drift early.',
        cards: [
          {
            label: 'Isolates',
            title: 'Live runtime views.',
            description: 'Monitor hot paths and isolate health in real time.'
          },
          {
            label: 'Resources',
            title: 'Budgets at a glance.',
            description: 'See CPU and memory limits before they bite.'
          }
        ]
      },
      {
        title: 'Trace what matters.',
        description:
          'Follow request paths and identify bottlenecks with clean diagnostics.',
        cards: [
          {
            label: 'Requests',
            title: 'Flow-centric insights.',
            description: 'Understand how work moves through the runtime.'
          },
          {
            label: 'Diagnostics',
            title: 'Focused debug signals.',
            description: 'Find the cause without log spelunking.'
          }
        ]
      }
    ]
  },
  desktop: {
    slug: 'desktop',
    title: 'Desktop',
    subtitle: 'Local runtime control with production parity.',
    summary:
      'Desktop brings the Deka runtime to your machine for fast iteration, offline testing, and consistent behavior.',
    highlights: [
      'Local runtime with full feature set',
      'Offline-first workflows',
      'Easy handoff to production'
    ],
    sections: [
      {
        title: 'Develop close to production.',
        description:
          'Use the same runtime features locally to avoid surprises later.',
        cards: [
          {
            label: 'Parity',
            title: 'Same runtime, local.',
            description: 'Test with real behavior, not mocks.'
          },
          {
            label: 'Services',
            title: 'Local service stack.',
            description: 'Run the core services on your machine.'
          }
        ]
      },
      {
        title: 'Ship from desktop to cluster.',
        description:
          'Move configs and artifacts into deployment workflows without rework.',
        cards: [
          {
            label: 'Export',
            title: 'Portable artifacts.',
            description: 'Package outputs for staging or production.'
          },
          {
            label: 'Sync',
            title: 'Consistent handoff.',
            description: 'Align local and remote runtime settings.'
          }
        ]
      }
    ]
  }
}

export const expandShowcase = {
  title: 'Why Deka is the best place to run production workloads.',
  description: 'Open each card to see the detail behind the platform decisions.',
  cards: [
    {
      label: 'performance',
      title: 'Performance you can feel.',
      description: 'Multi-core throughput without the overhead.',
      longDescription:
        'Deka optimizes cold paths, keeps isolates warm, and stays stable under real production traffic.',
      points: [
        {
          label: 'throughput',
          title: 'Linear multi-core scaling.',
          description: 'Scale from 1 to N cores without rewriting your app.'
        },
        {
          label: 'latency',
          title: 'Predictable p95s.',
          description: 'Stable tail latencies even under bursty loads.'
        }
      ]
    },
    {
      label: 'developer experience',
      title: 'Build with the tools you already use.',
      description: 'No surprise build steps or vendor lock-in.',
      longDescription:
        'Deka is designed to keep your build pipeline familiar while improving runtime output performance.',
      points: [
        {
          label: 'frameworks',
          title: 'Modern frameworks supported.',
          description: 'Next, Remix, Astro, and more ship without custom adapters.'
        },
        {
          label: 'typescript',
          title: 'TypeScript-first flow.',
          description: 'Smarter caching for faster rebuilds.'
        }
      ]
    },
    {
      label: 'operations',
      title: 'Operational clarity at scale.',
      description: 'Know what is happening without extra tooling.',
      longDescription:
        'Deka ships with runtime-aware telemetry that keeps operators informed without adding overhead.',
      points: [
        {
          label: 'telemetry',
          title: 'Signal over noise.',
          description: 'Metrics tuned to runtime health, not vanity charts.'
        },
        {
          label: 'reliability',
          title: 'Rollouts with guardrails.',
          description: 'Steady release workflows with clear rollback paths.'
        }
      ]
    },
    {
      label: 'security',
      title: 'Isolation by default.',
      description: 'Built for multi-tenant reliability.',
      longDescription:
        'Runtime isolation, access control, and deployment safety are part of the core runtime design.',
      points: [
        {
          label: 'isolation',
          title: 'Tenant boundaries enforced.',
          description: 'Keep workloads isolated without custom policies.'
        },
        {
          label: 'compliance',
          title: 'Audit-friendly outputs.',
          description: 'Structured logs and traceability from day one.'
        }
      ]
    }
  ]
}

export const scrollShowcase = {
  title: 'Big ideas, laid out simply.',
  description: 'Swipe across the platform pillars to see how each piece fits into the Deka runtime stack.',
  cards: [
    {
      label: 'edge routing',
      title: 'Global paths with local latency.',
      description: 'Route requests to the closest healthy region, automatically.'
    },
    {
      label: 'runtime isolation',
      title: 'Secure execution per workload.',
      description: 'Isolates stay warm, scoped, and performance-safe.'
    },
    {
      label: 'build pipeline',
      title: 'Ship from modern frameworks.',
      description: 'Next, Remix, Astro, and more stay first-class.'
    },
    {
      label: 'artifact compile',
      title: 'One binary per release.',
      description: 'Bundle assets, config, and runtime hints together.'
    },
    {
      label: 'release safety',
      title: 'Rollouts with guardrails.',
      description: 'Control traffic, monitor health, and rollback instantly.'
    }
  ]
}

export const modelShowcase = {
  title: 'Choose the runtime profile that fits.',
  description: 'A quick look at how Deka tunes for different deployment styles.',
  cards: [
    {
      label: 'edge',
      title: 'Deka Edge',
      description: 'Low-latency routing with global cache warmth.'
    },
    {
      label: 'core',
      title: 'Deka Core',
      description: 'Balanced throughput for production microservices.'
    },
    {
      label: 'compile',
      title: 'Deka Compile',
      description: 'Single-binary release artifacts for controlled environments.'
    },
    {
      label: 'sovereign',
      title: 'Deka Sovereign',
      description: 'Self-hosted runtime control for regulated stacks.'
    }
  ]
}
