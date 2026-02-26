---
title: System Architecture
description: Learn how Tana's components work together
---

## Overview

Tana consists of **five compiled binaries** and integrated services working together to provide a complete blockchain platform.

## Binary Structure

### 1. `tana` (CLI Binary)

- **Built from:** `cli/` directory (TypeScript/Bun)
- **Compiled with:** `bun build --compile`
- **Purpose:** Main user interface and orchestrator
- **Published:** `@tananetwork/tana` on npm

**Contains:**
- All commands (`new`, `deploy`, `balance`, `transfer`, `status`, `test`)
- Configuration management (`~/.config/tana/`)
- Network communication
- Most business logic

### 2. `tana-runtime` (Execution Binary)

- **Built from:** `runtime/` directory (Rust)
- **Compiled with:** `cargo build --release`
- **Purpose:** Sandboxed TypeScript contract execution for on-chain execution
- **Invoked by:** Block producer during consensus
- **Not persistent:** One-shot execution per contract run

### 3. `tana-edge` (Edge Server Binary)

- **Built from:** `edge/` directory (Rust)
- **Compiled with:** `cargo build --release`
- **Purpose:** High-performance HTTP server for off-chain contract execution
- **Port:** 8082 (default)
- **Persistent:** Runs as long-lived HTTP service
- **Features:**
  - GET/POST endpoints for smart contracts
  - Fresh V8 isolate per request (Cloudflare Workers-style)
  - Sub-100ms latency for blockchain queries
  - Read-only access (cannot modify state)
  - Accesses ledger API for blockchain data

### 4. `tana-mesh` (Network Coordinator)

- **Built from:** `mesh/` directory (TypeScript/Bun)
- **Purpose:** Network coordination and validator discovery
- **Port:** 8190 (default)
- **Features:**
  - Validator registration via heartbeats
  - Network topology visualization
  - Chaos engineering controls
  - Real-time WebSocket updates

### 5. `tana-t4` (Content Storage)

- **Built from:** `t4/` directory (TypeScript/Bun)
- **Purpose:** Content-addressable storage service
- **Port:** 8180 (default)
- **Features:**
  - Static asset storage
  - Content addressing (hash-based)

## Integrated Services

### Ledger Service

- **Location:** `cli/services/ledger`
- **Built into:** Standalone service
- **Purpose:** Blockchain state management (users, balances, transactions, blocks)
- **Port:** 8080
- **Persistent:** Runs as a long-lived service

**Stack:** TypeScript/Bun, Hono, Drizzle ORM, PostgreSQL

**Responsibilities:**
- User/Team account management
- Multi-currency balance tracking
- Transaction validation
- Block storage and queries
- REST API for all blockchain data

### Queue Service

- **Location:** `cli/services/queue`
- **Package:** `@tana/queue` (independent workspace)
- **Purpose:** High-performance transaction queue using Redis Streams
- **Shared by:** All validators for distributed block production

**Stack:** TypeScript/Bun, ioredis

**Responsibilities:**
- Transaction queueing with O(1) submission (100,000+ tx/sec)
- Consumer groups for distributed validators
- Exactly-once processing with XACK
- Deterministic transaction ordering via Stream IDs
- Pub/Sub notifications for instant updates

### Identity Service

- **Location:** `cli/services/identity`
- **Package:** `@tana/identity` (independent workspace)
- **Purpose:** User authentication and session management (NOT blockchain data)
- **Port:** 8090
- **Database:** Separate PostgreSQL tables (not replicated across validators)

**Stack:** TypeScript/Bun, Hono, Drizzle ORM, PostgreSQL

**Responsibilities:**
- Mobile-first QR code authentication (like WhatsApp Web)
- Session management with real-time updates (Server-Sent Events)
- Device token management for push notifications
- Transaction approval workflow for mobile apps
- Secure authentication without exposing private keys to desktop/laptop

**Security Model:**
- Private keys ONLY exist on mobile devices (hardware security)
- Desktop/laptop computers receive session tokens only (read-only interface)
- No MetaMask-style browser extensions (protects against malware)
- Ed25519 signature verification for all approvals

## Data Flow

### Contract Deployment Flow

```
User runs: tana deploy contract
          ↓
    [tana CLI binary]
          ↓
    1. Read config from ~/.config/tana/
    2. Determine deployment target (local/remote)
    3. Sign transaction with Ed25519
    4. Validate contract code
          ↓
    [Ledger API] ← HTTP POST /contracts
          ↓
    1. Verify signature
    2. Create transaction
    3. Queue to Redis Streams
          ↓
    [Redis Queue Service] ← XADD
          ↓
    [Block Producer] ← XREADGROUP (consumer)
          ↓
    1. Consume pending transactions
    2. Sort by Stream ID (deterministic ordering)
    3. Execute transactions
    4. Create new block
    5. XACK processed transactions
          ↓
    [tana-runtime binary] ← Execute contracts
          ↓
    Return state changes
          ↓
    [Ledger Service] ← Commit to PostgreSQL
```

### Authentication Flow (Mobile QR Login)

```
User clicks "Login" on website
          ↓
    [Website] ← POST /auth/session/create
          ↓
    [Identity Service] creates session
          ↓
    Returns: sessionId + challenge + QR data
          ↓
    [Website] displays QR code
    [Website] opens SSE connection (/auth/session/:id/events)
          ↓
    [User scans QR with mobile app]
          ↓
    [Mobile App] ← GET /auth/session/:id
          ↓
    [Identity Service] marks as "scanned"
          ↓
    [SSE] → "scanned" event to website
          ↓
    [User approves on mobile]
          ↓
    [Mobile App] signs challenge with Ed25519
    [Mobile App] ← POST /auth/session/:id/approve
          ↓
    [Identity Service] verifies signature
    [Identity Service] generates session token
          ↓
    [SSE] → "approved" event with token to website
          ↓
    [Website] receives session token
    [Website] redirects to app with authentication
```

## Configuration Structure

### Global Configuration

```
~/.config/tana/
├── config.json              # Global settings (default chain, user)
├── chains/
│   ├── local.json          # Local chain config
│   └── mainnet.json        # Remote chain configs
├── nodes/
│   └── node-xyz.json       # Node participation configs
└── users/
    └── alice.json          # User credentials (keys)
```

### Project Structure

```
my-app/
├── contract.ts             # Contract code
└── contract.json           # Contract metadata
```

## Component Details

### Ledger Service (Port 8080)

**Endpoints:**
- `GET /health` - Health check
- `GET /users` - List users
- `POST /users` - Create user
- `GET /balances` - Query balances
- `POST /balances` - Set balance
- `GET /transactions` - List transactions
- `POST /transactions` - Create transaction (queues to Redis)
- `GET /blocks` - Query blocks
- `POST /blocks` - Create block
- `GET /contracts` - List contracts
- `POST /contracts` - Deploy contract

**Database Tables:**
- `users` - User accounts
- `balances` - Multi-currency balances
- `transactions` - Transaction history (confirmed only)
- `currencies` - Supported currencies
- `blocks` - Blockchain blocks
- `contracts` - Deployed smart contracts

### Identity Service (Port 8090)

**Endpoints:**
- `GET /health` - Health check
- `POST /auth/session/create` - Create QR code authentication session
- `GET /auth/session/:id` - Get session details (mobile app scans QR)
- `GET /auth/session/:id/events` - SSE stream for real-time session updates
- `POST /auth/session/:id/approve` - Mobile app approves login with signature
- `POST /auth/session/:id/reject` - Mobile app rejects login
- `POST /auth/session/validate` - Validate session token

**Database Tables (Separate from blockchain):**
- `auth_sessions` - QR code login sessions with status tracking
- `transaction_requests` - Pending transaction approvals from mobile
- `device_tokens` - Push notification tokens for mobile devices

**Authentication Flow:**
1. Website creates session with `/auth/session/create` → receives session ID + challenge
2. Website displays QR code with session data
3. Website opens SSE connection to `/auth/session/:id/events` for real-time updates
4. Mobile app scans QR → calls `/auth/session/:id` → marks as "scanned"
5. User approves on mobile → signs challenge with Ed25519 private key
6. Mobile calls `/auth/session/:id/approve` with signature
7. SSE stream notifies website → session approved with token
8. Website receives session token for subsequent authenticated requests

### Consensus Service

- **Location:** `cli/services/consensus`
- **Purpose:** Multi-validator P2P consensus coordination
- **Ports:** 9000 (WebSocket P2P), 9001 (HTTP API)
- **Features:**
  - Validator-to-validator P2P communication
  - Block proposal and voting
  - Leader election
  - State synchronization

### Rust Runtime

**Purpose:** Sandboxed V8 TypeScript execution engine

- Execute smart contracts in isolated environment
- No network access, filesystem, or system calls
- Deterministic execution
- Gas metering

**Modules Provided:**
- `tana/core` or `tana:core` - Console, version info
- `tana/kv` or `tana:kv` - Cloudflare-compatible KV storage
- `tana/context` or `tana:context` - Execution context (owner, caller, block, input)
- `tana/block` or `tana:block` - Blockchain queries (getUser, getBalance, getTransaction)
- `tana/tx` or `tana:tx` - Transaction staging and execution
- `tana/utils` or `tana:utils` - Whitelisted fetch API

## Monorepo Structure

```
tana/                        # Monorepo root
├── runtime/                 # Rust - V8 TypeScript execution engine
│   ├── src/                 # Rust source (deno_core)
│   └── Cargo.toml
│
├── tana-edge/               # Rust - High-performance edge server
│   ├── src/                 # Rust source (Actix Web + V8)
│   └── Cargo.toml
│
├── cli/                     # TypeScript/Bun - Command-line tools
│   ├── core/                # Core commands and logic
│   ├── services/            # Integrated services
│   │   ├── ledger/          # @tana/ledger - Blockchain state (port 8080)
│   │   ├── queue/           # @tana/queue - Redis transaction queue
│   │   └── identity/        # @tana/identity - Authentication (port 8090)
│   └── package.json
│
├── websites/                # Web applications
│   ├── playground/          # Interactive contract playground
│   ├── landing/             # Landing page
│   └── docs/                # Documentation site (you are here!)
│
├── types/                   # Shared TypeScript type definitions
│   ├── tana-core.d.ts
│   ├── tana-kv.d.ts         # KV storage (Cloudflare-compatible)
│   ├── tana-utils.d.ts
│   ├── tana-block.d.ts
│   ├── tana-context.d.ts    # Execution context
│   ├── tana-net.d.ts        # HTTP request/response
│   └── tana-tx.d.ts
│
├── examples/                # Example smart contracts
├── docs/                    # Technical documentation (markdown)
├── mprocs.yaml              # Multi-process configuration
└── package.json             # Workspace management
```

## Development Workflow

### Database Setup

PostgreSQL stores both blockchain data and authentication data in separate tables:

#### Blockchain Tables (Ledger Service)

```sql
-- Users
CREATE TABLE users (
  id TEXT PRIMARY KEY,
  username TEXT UNIQUE,
  display_name TEXT,
  public_key TEXT,
  nonce INTEGER DEFAULT 0,
  bio TEXT,
  created_at TIMESTAMP
);

-- Balances
CREATE TABLE balances (
  owner_id TEXT,
  owner_type TEXT,
  currency_code TEXT,
  amount TEXT,
  updated_at TIMESTAMP
);

-- Transactions (confirmed only - pending in Redis queue)
CREATE TABLE transactions (
  id TEXT PRIMARY KEY,
  from_id TEXT,
  to_id TEXT,
  amount TEXT,
  currency_code TEXT,
  type TEXT,
  status TEXT,
  metadata JSONB,
  created_at TIMESTAMP,
  confirmed_at TIMESTAMP
);

-- Blocks
CREATE TABLE blocks (
  height INTEGER PRIMARY KEY,
  hash TEXT,
  parent_hash TEXT,
  state_root TEXT,
  timestamp TIMESTAMP,
  producer TEXT,
  transactions JSONB
);

-- Contracts
CREATE TABLE contracts (
  id TEXT PRIMARY KEY,
  code TEXT,
  code_hash TEXT,
  owner_id TEXT,
  created_at TIMESTAMP
);
```

#### Authentication Tables (Identity Service)

**Note:** These tables are separate from blockchain data and NOT replicated across validators.

```sql
-- Auth Sessions (QR code login)
CREATE TABLE auth_sessions (
  id TEXT PRIMARY KEY,
  challenge TEXT NOT NULL UNIQUE,
  status auth_session_status NOT NULL DEFAULT 'waiting',
  user_id TEXT,
  username TEXT,
  public_key TEXT,
  session_token TEXT UNIQUE,
  return_url TEXT NOT NULL,
  app_name TEXT,
  app_icon TEXT,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  expires_at TIMESTAMP NOT NULL,
  approved_at TIMESTAMP,
  scanned_at TIMESTAMP
);

-- Transaction Requests (mobile approval)
CREATE TABLE transaction_requests (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL REFERENCES auth_sessions(id),
  user_id TEXT NOT NULL,
  transaction_type TEXT NOT NULL,
  transaction_data TEXT NOT NULL,
  status transaction_request_status NOT NULL DEFAULT 'pending',
  transaction_id TEXT,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  expires_at TIMESTAMP NOT NULL,
  responded_at TIMESTAMP
);

-- Device Tokens (push notifications)
CREATE TABLE device_tokens (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  push_token TEXT NOT NULL UNIQUE,
  device_name TEXT,
  platform TEXT,
  app_version TEXT,
  is_active TEXT NOT NULL DEFAULT 'true',
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  last_used_at TIMESTAMP NOT NULL DEFAULT NOW()
);
```

### Environment Configuration

All sensitive configuration is stored in `.env` files:

```bash
# .env

# Database URLs
DATABASE_URL='postgres://tana:[PASSWORD]@localhost:5432/tana'
IDENTITY_DB_URL='postgres://tana:[PASSWORD]@localhost:5432/tana'  # Can use same DB
REDIS_URL='redis://localhost:6379'

# Service Ports
PORT=8080                    # Ledger service
IDENTITY_PORT=8090           # Identity service
EDGE_PORT=8180              # Edge server (tana-edge)

# Environment
NODE_ENV=development
```

**Environment Variable Reference:**

- `DATABASE_URL` - PostgreSQL connection string for blockchain data (ledger service)
- `IDENTITY_DB_URL` - PostgreSQL connection string for authentication data (identity service)
  - Can use the same database as `DATABASE_URL` (tables are separate)
  - Identity data is NOT replicated across validators
- `REDIS_URL` - Redis connection string for transaction queue
  - Shared by all validators
  - Must support Redis Streams
- `PORT` - HTTP port for ledger service API (default: 8080)
- `IDENTITY_PORT` - HTTP port for identity service API (default: 8090)
- `EDGE_PORT` - HTTP port for edge server (default: 8180)

## Security Model

### Sandbox Isolation

The Rust runtime provides complete isolation:

- No filesystem access
- No network access (except whitelisted fetch via `tana/utils`)
- No process spawning
- No system calls
- Memory limits enforced
- Gas metering for execution costs

### API Security

- CORS enabled for allowed origins
- Rate limiting on all endpoints
- Input validation with Zod schemas
- SQL injection protection via Drizzle ORM
- Environment-based configuration

## Deployment Architecture

### Development

```
mprocs → Multiple services in one terminal
  ├── PostgreSQL (Docker) - Blockchain + Auth data
  ├── Redis (Docker) - Transaction queue
  ├── Ledger Service (Bun:8080) - Blockchain API
  ├── Identity Service (Bun:8090) - Authentication API
  ├── Edge Server (Rust:8180) - Contract execution
  └── Docs/Playground (Astro/Node)
```

### Production - Single Node

```
Single VPS:
  ├── Nginx (Reverse proxy, SSL)
  │   ├── /api → Ledger Service (8080)
  │   ├── /auth → Identity Service (8090)
  │   └── /edge → Edge Server (8180)
  ├── Docker Compose
  │   ├── Ledger Service (port 8080)
  │   ├── Identity Service (port 8090)
  │   ├── PostgreSQL (blockchain + auth)
  │   └── Redis (transaction queue)
  ├── tana-runtime (Binary)
  └── tana-edge (Binary on 8180)
```

### Production - Multi-Validator Network

```
Network Topology:

[Queue Service Node]
  └── Redis (shared by all validators)

[Validator Node 1]
  ├── PostgreSQL (own copy)
  ├── Ledger Service (8080)
  └── Block Producer (consumes from shared Redis)

[Validator Node 2]
  ├── PostgreSQL (own copy)
  ├── Ledger Service (8080)
  └── Block Producer (consumes from shared Redis)

[API Node] (public-facing, read-only)
  ├── PostgreSQL (synced via consensus)
  ├── Ledger Service (8080)
  ├── Identity Service (8090)
  └── Edge Server (8180)

[Mobile App]
  └── Connects to Identity Service for auth
```

**Key Points:**
- Each validator runs its own PostgreSQL (database replication via blockchain consensus)
- Redis queue is shared by all validators
- Identity service typically runs on API nodes (not validators)
- Mobile apps connect to identity service for authentication
- Deterministic transaction ordering ensures consensus

## Next Steps

- [Quick Start Guide](/guides/quickstart/) - Set up your development environment
- [CLI Reference](/tana-cli/intro/) - Learn the CLI commands
- [API Reference](/tana-api/intro/) - Explore the REST API
