---
title: Environment Variables
description: Configuration via environment variables
sidebar:
  order: 3
---

Tana uses environment variables for configuration across all components. This provides a simple, consistent way to configure development, testing, and production environments.

## TANA_LEDGER_URL

**Purpose:** Configures the ledger API endpoint for all Tana components

**Default:** `http://localhost:8080`

**Usage:**
```bash
# Development (default - no env var needed)
tana new user @alice

# Remote chain
export TANA_LEDGER_URL=https://mainnet.tana.network
tana new user @alice

# Per-command override
TANA_LEDGER_URL=https://testnet.tana.network tana deploy user @alice
```

### Components That Use This Variable

**CLI (TypeScript/Bun):**
```typescript
// cli/utils/config.ts
export function getLedgerUrl(): string {
  return process.env.TANA_LEDGER_URL || 'http://localhost:8080'
}
```

**Contract Runner Script:**
```typescript
// scripts/run-contract.ts
const LEDGER_URL = process.env.TANA_LEDGER_URL || 'http://localhost:8080';
```

**Edge Server (Rust):**
```rust
// tana-edge/src/main.rs
use std::env;
let ledger_url = env::var("TANA_LEDGER_URL")
    .unwrap_or_else(|_| "http://localhost:8080".to_string());
```

**Mobile App (React Native/Expo):**
```javascript
// mobile/app.config.js
extra: {
  ledgerApiUrl: process.env.TANA_LEDGER_URL || 'http://localhost:8080'
}
```

### Framework-Specific Considerations

Different frameworks access environment variables differently:

**Bun/Node.js:**
```typescript
process.env.TANA_LEDGER_URL
```

**Astro:**
```typescript
import.meta.env.TANA_LEDGER_URL
// Add to .env file in project root
```

**Next.js:**
```typescript
// Server-side
process.env.TANA_LEDGER_URL

// Client-side (requires NEXT_PUBLIC_ prefix)
process.env.NEXT_PUBLIC_TANA_LEDGER_URL
```

**Vite:**
```typescript
import.meta.env.VITE_TANA_LEDGER_URL
// Add to .env file with VITE_ prefix
```

**Rust:**
```rust
use std::env;
env::var("TANA_LEDGER_URL").unwrap_or_else(|_| "default".to_string())
```

## Development Workflows

### Local Development

For local development, the default value (`http://localhost:8080`) works automatically:

```bash
# Start the ledger
tana start

# Use CLI (automatically connects to localhost:8080)
tana new user @alice
tana deploy user @alice
```

### Multi-Chain Development

Switch between chains by setting the environment variable:

```bash
# Work on local development chain
TANA_LEDGER_URL=http://localhost:8080 tana status

# Work on testnet
TANA_LEDGER_URL=https://testnet.tana.network tana status

# Work on custom community chain
TANA_LEDGER_URL=https://my-chain.example.com tana status
```

### Docker/Container Deployment

**Dockerfile:**
```dockerfile
ENV TANA_LEDGER_URL=https://mainnet.tana.network
```

**docker-compose.yml:**
```yaml
services:
  tana-edge:
    environment:
      - TANA_LEDGER_URL=https://mainnet.tana.network
```

**Kubernetes:**
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: tana-config
data:
  TANA_LEDGER_URL: "https://mainnet.tana.network"
```

### CI/CD Pipelines

**GitHub Actions:**
```yaml
env:
  TANA_LEDGER_URL: https://testnet.tana.network

steps:
  - name: Run tests
    run: bun test
    env:
      TANA_LEDGER_URL: http://localhost:8080
```

**GitLab CI:**
```yaml
variables:
  TANA_LEDGER_URL: "https://testnet.tana.network"
```

## Production Configuration

### Managed Hosting (tana.network)

When using the official tana.network hosted service:

```bash
export TANA_LEDGER_URL=https://api.tana.network
```

### Self-Hosted

When running your own blockchain infrastructure:

```bash
# Single instance
export TANA_LEDGER_URL=https://ledger.mycompany.com

# Load balanced
export TANA_LEDGER_URL=https://ledger-lb.mycompany.com

# Internal network
export TANA_LEDGER_URL=http://ledger.internal:8080
```

## Validation and Debugging

### Check Current Configuration

**CLI Status:**
```bash
tana status
# Shows: Ledger: ● Running on http://localhost:8080
```

**Environment Check:**
```bash
echo $TANA_LEDGER_URL
# Should output the URL or be empty (uses default)
```

### Common Issues

**Issue:** CLI can't connect to ledger
```bash
# Check if ledger is reachable
curl $TANA_LEDGER_URL/health

# Verify environment variable is set correctly
echo $TANA_LEDGER_URL
```

**Issue:** Wrong chain being targeted
```bash
# Check current configuration
tana status

# Override temporarily
TANA_LEDGER_URL=https://correct-chain.com tana status
```

**Issue:** Docker container can't reach localhost
```bash
# In Docker, localhost refers to the container
# Use host.docker.internal on Mac/Windows or host network mode on Linux

# Docker for Mac/Windows
export TANA_LEDGER_URL=http://host.docker.internal:8080

# Docker for Linux (use host network)
docker run --network host -e TANA_LEDGER_URL=http://localhost:8080 ...
```

## DATABASE_URL

**Purpose:** PostgreSQL connection string for blockchain data (ledger service)

**Default:** None (required for ledger service)

**Format:** `postgres://username:password@host:port/database`

**Usage:**
```bash
# Development
export DATABASE_URL='postgres://tana:tana_dev_password@localhost:5432/tana'

# Production
export DATABASE_URL='postgres://prod_user:secure_password@db.example.com:5432/tana_prod'
```

**Used by:**
- Ledger service (blockchain state storage)
- Block producer scripts
- Database migration tools

## IDENTITY_DB_URL

**Purpose:** PostgreSQL connection string for authentication data (identity service)

**Default:** Falls back to `DATABASE_URL` if not set

**Format:** `postgres://username:password@host:port/database`

**Usage:**
```bash
# Development (can use same database as blockchain)
export IDENTITY_DB_URL='postgres://tana:tana_dev_password@localhost:5432/tana'

# Production (recommended: separate database for security)
export IDENTITY_DB_URL='postgres://identity_user:secure_password@identity-db.example.com:5432/tana_identity'
```

**Notes:**
- Authentication tables are separate from blockchain data
- NOT replicated across validators
- Can use same database as `DATABASE_URL` (tables don't overlap)
- Identity service only runs on API nodes (not validators)

**Used by:**
- Identity service (QR authentication, session management)

## REDIS_URL

**Purpose:** Redis connection string for transaction queue

**Default:** `redis://localhost:6379`

**Format:** `redis://[username:password@]host:port[/database]`

**Usage:**
```bash
# Development
export REDIS_URL='redis://localhost:6379'

# Production with authentication
export REDIS_URL='redis://username:password@redis.example.com:6379'

# Redis Cluster
export REDIS_URL='redis://redis-cluster.example.com:6379'
```

**Requirements:**
- Redis 7+ required (Redis Streams support)
- Must support XADD, XREADGROUP, XACK commands
- Shared by all validators in the network

**Used by:**
- Ledger service (transaction submission)
- Queue service (transaction queueing)
- Block producer scripts (transaction consumption)

## PORT

**Purpose:** HTTP port for ledger service API

**Default:** `8080`

**Usage:**
```bash
# Development (default)
export PORT=8080

# Production (custom port)
export PORT=3000
```

**Used by:**
- Ledger service HTTP server

## IDENTITY_PORT

**Purpose:** HTTP port for identity service API

**Default:** `8090`

**Usage:**
```bash
# Development (default)
export IDENTITY_PORT=8090

# Production (custom port)
export IDENTITY_PORT=3001
```

**Used by:**
- Identity service HTTP server

## EDGE_PORT

**Purpose:** HTTP port for edge server (contract execution)

**Default:** `8180`

**Usage:**
```bash
# Development (default)
export EDGE_PORT=8180

# Production (custom port)
export EDGE_PORT=8000
```

**Used by:**
- Tana Edge server (Rust binary)

## Service URLs for Client Applications

**Purpose:** Configure client apps to connect to Tana services

### TANA_IDENTITY_URL

**Default:** `http://localhost:8090`

**Usage:**
```bash
# Development
export TANA_IDENTITY_URL=http://localhost:8090

# Production
export TANA_IDENTITY_URL=https://auth.tana.network
```

**Used by:**
- Website applications (QR login pages)
- Mobile apps (authentication flows)

### TANA_EDGE_URL

**Default:** `http://localhost:8180`

**Usage:**
```bash
# Development
export TANA_EDGE_URL=http://localhost:8180

# Production
export TANA_EDGE_URL=https://edge.tana.network
```

**Used by:**
- Applications calling contract endpoints
- Edge function deployments

## Complete Development Setup

For local development, create a `.env` file in your project root:

```bash
# .env

# Database connections
DATABASE_URL='postgres://tana:tana_dev_password@localhost:5432/tana'
IDENTITY_DB_URL='postgres://tana:tana_dev_password@localhost:5432/tana'
REDIS_URL='redis://localhost:6379'

# Service ports
PORT=8080                    # Ledger service
IDENTITY_PORT=8090           # Identity service
EDGE_PORT=8180              # Edge server

# Client URLs (for connecting to services)
TANA_LEDGER_URL=http://localhost:8080
TANA_IDENTITY_URL=http://localhost:8090
TANA_EDGE_URL=http://localhost:8180

# Environment
NODE_ENV=development
```

## Complete Production Setup

For production deployment:

```bash
# .env.production

# Database connections (use secure passwords!)
DATABASE_URL='postgres://prod_user:SECURE_PASSWORD@db.internal:5432/tana_prod'
IDENTITY_DB_URL='postgres://identity_user:SECURE_PASSWORD@identity-db.internal:5432/tana_identity'
REDIS_URL='redis://redis_user:SECURE_PASSWORD@redis.internal:6379'

# Service ports (behind reverse proxy)
PORT=8080
IDENTITY_PORT=8090
EDGE_PORT=8180

# Public URLs (with SSL)
TANA_LEDGER_URL=https://api.tana.network
TANA_IDENTITY_URL=https://auth.tana.network
TANA_EDGE_URL=https://edge.tana.network

# Environment
NODE_ENV=production
```

## Best Practices

1. **Never commit `.env` files with secrets** - Use `.env.example` instead
2. **Use different values per environment** - dev, staging, production
3. **Validate URLs before deployment** - Ensure endpoints are reachable
4. **Document custom chains** - Keep track of which URL points where
5. **Use HTTPS in production** - Never use `http://` for production chains
6. **Set sensible defaults** - localhost:8080 works for most development

## Related Documentation

- [Development Setup](/contributing/setup/) - Initial environment configuration
- [Architecture](/contributing/architecture/) - System design overview
- [Deployment Guide](#) - Production deployment best practices (coming soon)
