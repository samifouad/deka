---
title: Blocks & Blockchain
description: Understanding Tana's block structure and production
---

Blocks are the fundamental unit of Tana's blockchain, packaging transactions and establishing a tamper-proof history of all state changes.

## Block Structure

### Database Schema

```sql
CREATE TABLE blocks (
  -- Block identification
  height        BIGINT PRIMARY KEY,           -- 0 for genesis, then 1, 2, 3...
  hash          VARCHAR(64) NOT NULL UNIQUE,  -- SHA256 of block contents
  parent_hash   VARCHAR(64) NOT NULL,         -- Hash of previous block

  -- Block metadata
  timestamp     TIMESTAMP NOT NULL,           -- Block creation time
  producer      UUID NOT NULL,                -- Node/user that produced this block

  -- Block contents
  transactions  JSONB NOT NULL,               -- Array of transaction IDs
  state_root    VARCHAR(64) NOT NULL,         -- Merkle root of state

  -- Execution
  gas_used      BIGINT NOT NULL DEFAULT 0,    -- Total gas consumed
  gas_limit     BIGINT NOT NULL,              -- Maximum gas allowed

  -- Timestamps
  created_at    TIMESTAMP NOT NULL DEFAULT NOW()
);
```

### TypeScript Interface

```typescript
interface Block {
  // Identity
  height: number          // Block number (0 = genesis)
  hash: string           // Block hash
  previousHash: string   // Previous block hash

  // Metadata
  timestamp: number      // Unix timestamp (ms)
  producer: string       // Producer ID

  // Contents
  transactions: string[] // Array of transaction IDs
  stateRoot: string      // State merkle root

  // Execution
  gasUsed: number        // Gas consumed
  gasLimit: number       // Gas limit

  // Timestamps
  createdAt: Date
}
```

## Genesis Block (Block 0)

The **genesis block** is block #0 - the first block in the blockchain. It establishes the initial state and network parameters.

### Genesis Block Example

```typescript
{
  // Identity
  height: 0,
  hash: "0xa1b2c3d4e5f6...",
  previousHash: "0x0000000000000000000000000000000000000000000000000000000000000000",

  // Metadata
  timestamp: 1730678400000,  // Launch date
  producer: "tana:genesis",

  // Contents
  transactions: [],
  stateRoot: "0x...",

  // Execution
  gasUsed: 0,
  gasLimit: 1000000,

  // Genesis metadata
  metadata: {
    version: "0.1.0",
    networkName: "tana",
    chainId: 1,
    genesisMessage: "Tana blockchain - TypeScript smart contracts",
    blockTime: 6000,  // 6 second blocks
  }
}
```

## Block Production Flow

### Current State (Development)

```
Transaction submitted
    ↓
Directly applied to database
    ↓
Block created with transaction
    ↓
State root calculated
    ↓
Block finalized
```

### Future State (Production)

```
1. Transactions submitted to mempool
    ↓
2. Block producer selects transactions
    ↓
3. Execute transactions in order
    ↓
4. Calculate state root
    ↓
5. Create block with:
   - Transactions included
   - State root
   - Gas used
    ↓
6. Sign block
    ↓
7. Insert block into database
    ↓
8. Propagate to network
```

## Block Hash Calculation

Blocks are identified by the hash of their contents:

```typescript
function calculateBlockHash(block: Block): string {
  const data = JSON.stringify({
    height: block.height,
    previousHash: block.previousHash,
    timestamp: block.timestamp,
    stateRoot: block.stateRoot,
    transactions: block.transactions,
    gasUsed: block.gasUsed,
  })

  return crypto.createHash('sha256')
    .update(data)
    .digest('hex')
}
```

## State Root

The state root is a Merkle root of all account states after executing the block's transactions. This allows:

- **Efficient verification** - Prove account state without full blockchain
- **Light clients** - Verify transactions without downloading everything
- **State snapshots** - Quick sync for new nodes
- **Fraud proofs** - Prove invalid state transitions

### State Root Calculation

```typescript
function calculateStateRoot(accounts: Account[]): string {
  // Sort accounts by ID for deterministic hashing
  const sorted = accounts.sort((a, b) => a.id.localeCompare(b.id))

  // Hash each account state
  const accountHashes = sorted.map(account => {
    const state = {
      id: account.id,
      balances: account.balances,
      nonce: account.nonce,
    }
    return hash(JSON.stringify(state))
  })

  // Build Merkle tree and return root
  return buildMerkleRoot(accountHashes)
}
```

## Block Queries

### Get Latest Block

```typescript
// Via API
GET /blocks/latest

// Response
{
  "height": 12345,
  "hash": "0xa1b2c3d4...",
  "timestamp": 1699999999000,
  "transactions": ["tx1", "tx2", "tx3"],
  "stateRoot": "0x...",
  "gasUsed": 150000,
  "gasLimit": 1000000
}
```

### Get Specific Block

```typescript
// Via API
GET /blocks/12345

// Or by hash
GET /blocks/0xa1b2c3d4...
```

### Get Block Range

```typescript
// Via API
GET /blocks?from=100&to=200

// Returns array of blocks
```

## Block Context in Contracts

Smart contracts can access block information:

```typescript
import { console } from 'tana/core'
import { block } from 'tana/block'

// Current block context
console.log('Block height:', block.height)
console.log('Block hash:', block.hash)
console.log('Block timestamp:', block.timestamp)
console.log('Block producer:', block.producer)
console.log('Gas limit:', block.gasLimit)

// Query historical blocks (future)
const genesis = await block.getBlock(0)
console.log('Genesis message:', genesis.metadata.genesisMessage)
```

## Block Explorer

Query blocks via the API or CLI:

```bash
# Get latest block
curl http://localhost:8080/blocks/latest

# Get specific block
curl http://localhost:8080/blocks/12345

# Get block transactions
curl http://localhost:8080/blocks/12345/transactions
```

## Development Roadmap

### Phase 1: Basic Blocks (Current)
- ✅ Blocks table
- ✅ Genesis block
- ✅ Block creation on transaction
- ✅ Block queries via API

### Phase 2: Block Production
- ⏳ Mempool for pending transactions
- ⏳ Block producer service
- ⏳ Transaction selection
- ⏳ Automated block creation

### Phase 3: Consensus
- ⏳ Multiple validators
- ⏳ Consensus mechanism
- ⏳ Fork resolution
- ⏳ Finality rules

## Next Steps

- [Data Model](/guides/core/data-model/) - Learn about Tana's data structures
- [Contract Execution](/guides/core/contract-execution/) - How contracts interact with blocks
- [API Reference](/tana-api/intro/) - Query blocks via the REST API
