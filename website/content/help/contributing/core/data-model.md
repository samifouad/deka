---
title: Data Model
description: Core data structures in Tana blockchain
---

Tana's blockchain stores users, balances, transactions, contracts, and blocks. This page describes the core data model.

## Primary Objects

### User

User accounts are the foundation of Tana. Each user has a unique ID, public key for signatures, and multi-currency balances.

```typescript
interface User {
  id: string                    // Unique ID (address)
  publicKey: string             // Ed25519 public key
  username: string              // @alice
  displayName: string           // "Alice Johnson"
  metadata: {
    bio?: string
    avatarData?: string         // Small image stored on-chain (<100KB base64)
    avatarHash?: string         // Or content hash if stored off-chain
  }
  balances: Map<Currency, Decimal>  // Multi-currency
  createdAt: timestamp
  stateHash: string             // Merkle root of account state
}
```

### Transaction

All blockchain state changes are recorded as transactions.

```typescript
interface Transaction {
  id: string                    // Tx hash
  from: string                  // User/Team ID
  to: string                    // User/Team ID
  amount: Decimal
  currency: Currency            // USD, BTC, ETH, etc.
  type: 'transfer' | 'deposit' | 'withdraw' | 'contract_call' | 'user_creation'
  contractId?: string           // If contract execution
  contractInput?: any           // Arguments
  signature: string             // Ed25519
  timestamp: timestamp
  blockHeight: number           // Block inclusion
  status: 'pending' | 'confirmed' | 'failed'
}
```

### Currency

Tana supports multi-currency balances without requiring a native token.

```typescript
interface Currency {
  code: string                  // "USD", "BTC", "ETH"
  name: string                  // "US Dollar", "Bitcoin"
  symbol: string                // "$", "₿"
  decimals: number              // Precision (e.g., 2 for USD, 8 for BTC)
  type: 'fiat' | 'crypto'
  verified: boolean             // Is this a recognized currency?
}
```

### Smart Contract

Contracts are TypeScript code deployed on-chain with isolated key-value storage.

```typescript
interface SmartContract {
  id: string                    // Contract address
  codeHash: string              // SHA-256 of code
  code: string                  // TypeScript source
  author: string                // User ID
  deployedAt: timestamp
  storage: Map<string, string>  // Key-value state (tana/data)
  metadata: {
    name?: string
    description?: string
    version?: string
  }
}
```

### Block

Blocks package transactions and establish blockchain state.

```typescript
interface Block {
  id: string                    // Block hash
  height: number                // Block number (0 = genesis)
  timestamp: timestamp
  previousHash: string          // Previous block
  transactions: string[]        // Tx hashes
  stateRoot: string             // Merkle root of all account states
  validatorSignature: string    // Block producer signature
  gasUsed: number              // Gas consumed
  gasLimit: number             // Gas limit
}
```

## Database Schema

### Users Table

```sql
CREATE TABLE users (
  id TEXT PRIMARY KEY,
  username TEXT UNIQUE NOT NULL,
  display_name TEXT,
  public_key TEXT NOT NULL,
  bio TEXT,
  created_at TIMESTAMP DEFAULT NOW()
);
```

### Balances Table

```sql
CREATE TABLE balances (
  owner_id TEXT NOT NULL,
  owner_type TEXT NOT NULL,
  currency_code TEXT NOT NULL,
  amount TEXT NOT NULL,  -- Stored as string for precision
  updated_at TIMESTAMP DEFAULT NOW(),
  PRIMARY KEY (owner_id, currency_code)
);
```

### Transactions Table

```sql
CREATE TABLE transactions (
  id TEXT PRIMARY KEY,
  from_id TEXT,
  to_id TEXT,
  amount TEXT,
  currency_code TEXT NOT NULL,
  type TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  block_height INTEGER,
  metadata JSONB,
  created_at TIMESTAMP DEFAULT NOW()
);
```

### Currencies Table

```sql
CREATE TABLE currencies (
  code TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  symbol TEXT NOT NULL,
  decimals INTEGER NOT NULL,
  is_active BOOLEAN DEFAULT true,
  created_at TIMESTAMP DEFAULT NOW()
);
```

### Blocks Table

```sql
CREATE TABLE blocks (
  height INTEGER PRIMARY KEY,
  hash TEXT UNIQUE NOT NULL,
  parent_hash TEXT NOT NULL,
  state_root TEXT NOT NULL,
  timestamp TIMESTAMP NOT NULL,
  producer TEXT NOT NULL,
  transactions JSONB NOT NULL,
  gas_used BIGINT DEFAULT 0,
  gas_limit BIGINT NOT NULL,
  created_at TIMESTAMP DEFAULT NOW()
);
```

### Contracts Table

```sql
CREATE TABLE contracts (
  id TEXT PRIMARY KEY,
  code TEXT NOT NULL,
  owner_id TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT NOW()
);
```

### KV Storage Table

Per-contract key-value storage for `tana/kv` module.

```sql
CREATE TABLE kv_storage (
  contract_name TEXT NOT NULL,
  key TEXT NOT NULL,
  value TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (contract_name, key)
);

CREATE INDEX idx_kv_contract ON kv_storage(contract_name);
```

**Notes:**
- Each contract has isolated namespace via `contract_name`
- Keys automatically prefixed: `"{contract_name}:{key}"`
- Max key length: 512 bytes
- Max value size: 25 MB
- Used by `tana/kv` module (Cloudflare Workers-compatible API)

## State Management

### Account State

Each user and team has a state hash that represents their complete state:

- All balances across all currencies
- Account metadata
- Contract storage (if applicable)

### State Root

Each block has a state root - a Merkle root of all account state hashes. This allows:

- Efficient state verification
- Light client support
- State snapshots
- Fraud proofs

### Transaction Flow

```
User submits transaction
    ↓
Validate (signature, balance, etc.)
    ↓
Add to pending transactions
    ↓
Block producer includes in block
    ↓
Execute transaction
    ↓
Update account states
    ↓
Calculate new state root
    ↓
Finalize block
```

## Next Steps

- [Blocks & Blockchain](/guides/core/blocks/) - Learn about block structure
- [Contract Execution](/guides/core/contract-execution/) - How contracts interact with state
- [API Reference](/tana-api/intro/) - Interact with the data model via API
