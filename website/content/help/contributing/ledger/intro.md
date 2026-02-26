---
title: Ledger Service
description: Account and balance management service architecture
sidebar:
  order: 1
---

The Tana Ledger is the core account and balance management service for the blockchain. It provides a REST API for user accounts, multi-currency balances, and transaction processing.

## Features

- **User Accounts** - Create and manage user accounts with Ed25519 keys
- **Team Management** - Team creation, membership, and treasury accounts
- **Multi-Currency Balances** - Track balances across multiple currencies (USD, EUR, BTC, etc.)
- **Transaction Processing** - Validate and process transactions through block production
- **REST API** - HTTP API for all account operations

## Architecture

```
ledger/
├── src/
│   ├── index.ts          # Main API server (Hono)
│   ├── accounts/
│   │   ├── users.ts      # User account management
│   │   └── teams.ts      # Team account management
│   ├── balances/
│   │   ├── index.ts      # Balance operations
│   │   └── currencies.ts # Currency management
│   ├── transactions/
│   │   ├── index.ts      # Transaction processing
│   │   └── validator.ts  # Transaction validation
│   ├── blocks/
│   │   └── producer.ts   # Block production logic
│   ├── api/
│   │   ├── routes/       # API route handlers
│   │   └── schemas.ts    # Zod validation schemas
│   └── db/
│       ├── schema.ts     # Drizzle ORM schema
│       └── index.ts      # Database connection
└── drizzle/              # Database migrations
```

## Development Setup

### Prerequisites

- Bun 1.1.38+
- PostgreSQL 16
- Docker (for local development)

### Installation

```bash
cd ledger

# Install dependencies
bun install

# Start PostgreSQL (via Docker)
docker-compose up -d postgres

# Run database migrations
bun run db:migrate

# Start development server
bun run dev
```

The API will be available at `http://localhost:8080`.

### Database Migrations

```bash
# Generate new migration
bun run db:generate

# Run migrations
bun run db:migrate

# Drop database (destructive!)
bun run db:drop

# Push schema changes (development only)
bun run db:push

# Open Drizzle Studio (GUI)
bun run db:studio
```

## Block Production

Transactions in Tana are processed into blocks. During development, you need to manually produce blocks to finalize pending transactions.

### Manual Block Production

```bash
# From project root:
npm run pending    # Process pending transactions
npm run block      # Same as above (alias)

# From ledger directory:
npm run pending    # Short command
npm run blockchain:produce  # Full command name
```

**When to run:**
- After creating users (via mobile app or CLI)
- After submitting transactions
- When you want to finalize pending operations

**What it does:**
1. Finds all pending transactions in the database
2. Executes them (user creation, transfers, contract calls, etc.)
3. Creates a new block with those transactions
4. Updates transaction status from "pending" → "confirmed"
5. Updates all affected balances and state

**Example output:**
```
🔨 Producing new block...
Latest block: 6
Found 3 pending transaction(s)

Processing transaction tx_abc123 (user_creation)...
  ✓ Created user: @alice (550e8400-e29b-41d4-a716-446655440000)

Processing transaction tx_def456 (transfer)...
  ✓ Transferred 100 USD from @alice to @bob

Processing transaction tx_ghi789 (contract_call)...
  ✓ Executed contract cnt_myapp

✅ Block produced successfully!
Block Details:
  Height: 7
  Transactions: 3
  Gas Used: 63000 / 1000000
  Block Hash: 0xabc123...
```

### Automatic Block Production (Future)

In production, blocks will be produced automatically at regular intervals (e.g., every 6 seconds) by a background worker or consensus mechanism. This is not yet implemented.

## API Endpoints

### Users

```
GET  /users               # List all users
POST /users               # Create new user
GET  /users/:id           # Get user by ID
GET  /users/username/:username  # Get user by username
PATCH /users/:id          # Update user
DELETE /users/:id         # Delete user
GET  /users/:id/balances  # Get user balances
```

### Balances

```
GET  /balances            # List all balances
POST /balances            # Set balance (admin)
GET  /balances/:id        # Get specific balance
```

### Transactions

```
GET  /transactions        # List all transactions
POST /transactions        # Create transaction
GET  /transactions/:id    # Get transaction details
```

### Blocks

```
GET  /blocks              # List all blocks
GET  /blocks/:height      # Get block by height
GET  /blocks/latest       # Get latest block
```

### Contracts

```
GET  /contracts           # List all contracts
POST /contracts           # Deploy contract
GET  /contracts/:id       # Get contract details
```

## Environment Variables

```env
# Server Configuration
PORT=8080
NODE_ENV=development

# Database
DATABASE_URL=postgres://tana:tana_dev_password@localhost:5432/tana

# CORS (for playground integration)
ALLOWED_ORIGINS=http://localhost:3000,http://localhost:4321

# Ledger URL (for consistency with other services)
TANA_LEDGER_URL=http://localhost:8080
```

See [Environment Variables](/contributing/env-vars/) for more details.

## Database Schema

The ledger uses PostgreSQL with Drizzle ORM. Key tables:

**users** - User accounts
- `id` (UUID, primary key)
- `username` (unique, must start with @)
- `publicKey` (Ed25519 public key)
- `displayName`, `bio`, `avatarData`

**balances** - Account balances
- `id` (UUID, primary key)
- `ownerId` (references account)
- `ownerType` (user, team, contract, system)
- `amount` (numeric, precise to 8 decimals)
- `currencyCode` (USD, EUR, BTC, etc.)

**transactions** - All blockchain transactions
- `id` (UUID, primary key)
- `type` (user_creation, transfer, contract_call, etc.)
- `from`, `to` (account references)
- `amount`, `currencyCode`
- `status` (pending, confirmed, failed)
- `blockHeight` (null if pending)

**blocks** - Blockchain blocks
- `height` (bigint, primary key)
- `hash` (unique block hash)
- `timestamp`
- `gasLimit`, `gasUsed`
- `transactionCount`

**contracts** - Deployed smart contracts
- `id` (UUID, primary key)
- `deployedBy` (user reference)
- `codeHash` (contract code hash)
- `storageRoot` (contract storage root)

## Testing

```bash
# Run all tests
bun test

# Run specific test file
bun test src/accounts/users.test.ts

# Watch mode
bun test --watch
```

## Building for Production

```bash
# Build TypeScript
bun run build

# Output in dist/
ls dist/

# Run production build
NODE_ENV=production bun dist/index.js
```

## Deployment

See the [Production Deployment](/guides/deployment/) guide for deploying the ledger service.

Key points:
- Use Docker for consistent environments
- Set `DATABASE_URL` for PostgreSQL connection
- Configure `ALLOWED_ORIGINS` for CORS
- Set up Nginx reverse proxy with SSL
- Enable automated backups
- Monitor with health checks

## Related Documentation

- [Environment Variables](/contributing/env-vars/) - Configuration
- [Block Production](/contributing/ledger/blocks/) - Block production details
- [Transactions](/contributing/ledger/transactions/) - Transaction processing
- [API Reference](/tana-api/intro/) - Complete API documentation
- [Deployment Guide](/guides/deployment/) - Production deployment
