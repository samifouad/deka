export default function ReferenceHomePage() {
  return (
    <div className="max-w-5xl mx-auto px-8 py-12">
      <div className="space-y-6">
        <div>
          <h1 className="text-4xl font-bold text-foreground mb-2">
            API Reference
          </h1>
          <p className="text-xl text-muted-foreground">
            Complete REST API documentation for tana services.
          </p>
        </div>

        <div className="border-l-4 border-primary pl-4 py-2">
          <p className="text-muted-foreground">
            Browse the API endpoints in the sidebar. All endpoints return JSON and support
            CORS for development. Production deployments should implement rate limiting.
          </p>
        </div>

        <div className="grid md:grid-cols-2 gap-6 pt-6">
          <div className="border border-border rounded-lg p-6">
            <h3 className="text-lg font-semibold text-foreground mb-2">
              Ledger API
            </h3>
            <p className="text-muted-foreground text-sm mb-3">
              Blockchain operations: users, balances, transactions, blocks, and smart contracts.
            </p>
            <div className="bg-background border border-border rounded-md p-3 font-mono text-xs">
              <code className="text-foreground">http://localhost:8501</code>
            </div>
          </div>

          <div className="border border-border rounded-lg p-6">
            <h3 className="text-lg font-semibold text-foreground mb-2">
              Identity API
            </h3>
            <p className="text-muted-foreground text-sm mb-3">
              QR code authentication and session management. Private keys stay on mobile devices.
            </p>
            <div className="bg-background border border-border rounded-md p-3 font-mono text-xs">
              <code className="text-foreground">http://localhost:8504</code>
            </div>
          </div>
        </div>

        <div className="bg-secondary/50 border border-border rounded-lg p-6 mt-6">
          <h2 className="text-lg font-semibold text-foreground mb-3">Authentication</h2>
          <p className="text-muted-foreground mb-3">
            All state-changing operations require Ed25519 cryptographic signatures to prove ownership.
          </p>
          <div className="bg-background border border-border rounded-md p-4 font-mono text-xs overflow-x-auto">
            <code className="text-foreground">
              {`const message = createTransactionMessage(tx)
const signature = await signMessage(message, privateKey)

POST /transactions {
  ...tx,
  signature
}`}
            </code>
          </div>
        </div>

        <div className="grid md:grid-cols-3 gap-4 pt-4">
          <div className="border border-border rounded-lg p-4">
            <div className="text-sm font-semibold text-foreground mb-1">Response Format</div>
            <div className="text-xs text-muted-foreground">All responses are JSON</div>
          </div>
          <div className="border border-border rounded-lg p-4">
            <div className="text-sm font-semibold text-foreground mb-1">CORS Enabled</div>
            <div className="text-xs text-muted-foreground">Development mode allows all origins</div>
          </div>
          <div className="border border-border rounded-lg p-4">
            <div className="text-sm font-semibold text-foreground mb-1">Pagination</div>
            <div className="text-xs text-muted-foreground">?page=1&limit=20</div>
          </div>
        </div>
      </div>
    </div>
  )
}
