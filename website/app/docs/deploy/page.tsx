export default function DeployReferenceHomePage() {
  return (
    <div className="max-w-5xl mx-auto px-8 py-12">
      <div className="space-y-6">
        <div>
          <h1 className="text-4xl font-bold text-foreground mb-2">
            Deploy server
          </h1>
          <p className="text-xl text-muted-foreground">
            Operate, secure, and automate the Deka deploy server for production environments.
          </p>
        </div>

        <div className="border-l-4 border-primary pl-4 py-2">
          <p className="text-muted-foreground">
            These docs will move to generated reference pages once the deploy server APIs are wired.
          </p>
        </div>

        <div className="grid md:grid-cols-2 gap-6 pt-6">
          <div className="border border-border rounded-lg p-6">
            <h3 className="text-lg font-semibold text-foreground mb-2">
              Authentication
            </h3>
            <p className="text-muted-foreground text-sm">
              Protect deploy endpoints with signed requests and scoped credentials.
            </p>
          </div>

          <div className="border border-border rounded-lg p-6">
            <h3 className="text-lg font-semibold text-foreground mb-2">
              Release workflows
            </h3>
            <p className="text-muted-foreground text-sm">
              Roll out, rollback, and monitor releases with guardrails.
            </p>
          </div>
        </div>

        <div className="bg-secondary/50 border border-border rounded-lg p-6 mt-6">
          <h2 className="text-lg font-semibold text-foreground mb-3">Server endpoints</h2>
          <p className="text-muted-foreground mb-3">
            The sidebar currently mirrors the API reference while deploy server docs are staged.
          </p>
          <div className="bg-background border border-border rounded-md p-4 font-mono text-xs overflow-x-auto">
            <code className="text-foreground">
              {`POST /deployments
GET  /deployments/:id
POST /rollbacks`}
            </code>
          </div>
        </div>
      </div>
    </div>
  )
}
