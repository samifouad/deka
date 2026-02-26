export default function CLIReferenceHomePage() {
  return (
    <div className="max-w-5xl mx-auto px-8 py-12">
      <div className="space-y-6">
        <div>
          <h1 className="text-4xl font-bold text-foreground mb-2">
            Deka CLI
          </h1>
          <p className="text-xl text-muted-foreground">
            Command-line workflows for installing, operating, and upgrading Deka services.
          </p>
        </div>

        <div className="border-l-4 border-primary pl-4 py-2">
          <p className="text-muted-foreground">
            Browse commands in the sidebar. Each entry includes examples, flags, and related workflows.
          </p>
        </div>

        <div className="grid md:grid-cols-2 gap-6 pt-6">
          <div className="border border-border rounded-lg p-6">
            <h3 className="text-lg font-semibold text-foreground mb-2">
              Service lifecycle
            </h3>
            <p className="text-muted-foreground text-sm">
              Start, stop, upgrade, and monitor core services with predictable automation hooks.
            </p>
          </div>

          <div className="border border-border rounded-lg p-6">
            <h3 className="text-lg font-semibold text-foreground mb-2">
              Operator workflows
            </h3>
            <p className="text-muted-foreground text-sm">
              Inspect logs, manage repos, and keep runtime dependencies in sync.
            </p>
          </div>
        </div>

        <div className="bg-secondary/50 border border-border rounded-lg p-6 mt-6">
          <h2 className="text-lg font-semibold text-foreground mb-3">Quick example</h2>
          <p className="text-muted-foreground mb-3">
            Use the CLI to bootstrap and check service health from a single machine.
          </p>
          <div className="bg-background border border-border rounded-md p-4 font-mono text-xs overflow-x-auto">
            <code className="text-foreground">
              {`deka setup
deka start
deka status`}
            </code>
          </div>
        </div>
      </div>
    </div>
  )
}
