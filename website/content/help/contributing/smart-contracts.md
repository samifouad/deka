---
title: Smart Contracts
description: Learn to write TypeScript smart contracts for Tana
sidebar:
  order: 3
---

Tana smart contracts are TypeScript files that export up to 4 functions: `init()`, `contract()`, `get()`, and `post()`. All functions must return JSON-serializable objects.

## Quick Start

### 1. Scaffold a Contract

```bash
tana new contract mycontract
```

This creates two files:
- `contract.ts` - Your contract code
- `contract.json` - Contract metadata

### 2. Edit Your Contract

```typescript
// contract.ts
import { console } from 'tana:core'
import { block } from 'tana:block'
import { kv } from 'tana:kv'
import { context } from 'tana:context'

// On-chain execution (called via transactions)
export function contract() {
  const caller = context.caller()
  const owner = context.owner()

  if (!caller || caller.id !== owner.id) {
    return { error: 'Unauthorized: owner only' }
  }

  return { success: true, caller: caller.username }
}

// HTTP GET handler (served by tana-edge)
export function get() {
  return {
    message: 'Hello from Tana!',
    timestamp: Date.now()
  }
}
```

### 3. Test Locally

```bash
tana run contract.ts
```

### 4. Deploy

```bash
tana deploy contract contract.ts
```

## Contract Structure

Every contract exports up to 4 functions. None are strictly required - export only what you need.

| Function | Runs When | Writes Allowed | Context |
|----------|-----------|----------------|---------|
| `init()` | Contract deployment | ✅ Yes | `context.owner()`, `context.block()` |
| `contract()` | Transaction execution | ✅ Yes | `context.owner()`, `context.caller()`, `context.block()`, `context.input()` |
| `get()` | HTTP GET request | ❌ Read-only | Basic request info |
| `post()` | HTTP POST request | ❌ Read-only | Basic request info + body |

### init() - Deployment Hook

Runs **once** when the contract is deployed. Use for initial setup.

```typescript
import { context } from 'tana:context'
import { kv } from 'tana:kv'

export function init() {
  const owner = context.owner()
  const block = context.block()

  // Store deployment info
  await kv.put('owner_id', owner.id)
  await kv.put('deployed_at', String(block.timestamp))

  return {
    initialized: true,
    owner: owner.username,
    block: block.height
  }
}
```

### contract() - On-Chain Logic

Runs when users call your contract via blockchain transactions. This is where state changes happen.

```typescript
import { context } from 'tana:context'
import { kv } from 'tana:kv'

export async function contract() {
  const caller = context.caller()
  const owner = context.owner()
  const input = context.input()

  // Only owner can execute
  if (!caller || caller.id !== owner.id) {
    return { error: 'Unauthorized' }
  }

  // Read and update state
  const count = await kv.get('count') || '0'
  await kv.put('count', String(parseInt(count) + 1))

  return {
    success: true,
    count: parseInt(count) + 1,
    caller: caller.username
  }
}
```

### get() - HTTP Read Endpoint

Handles HTTP GET requests via tana-edge. **Read-only** - cannot modify state.

```typescript
import { kv } from 'tana:kv'

export async function get() {
  // Read state
  const count = await kv.get('count') || '0'

  return {
    count: parseInt(count),
    timestamp: Date.now()
  }
}
```

### post() - HTTP Write Endpoint

Handles HTTP POST requests via tana-edge. **Read-only** - cannot modify state directly.

```typescript
export function post(body: any) {
  // Validate input
  if (!body.name) {
    return { error: 'Name is required' }
  }

  // Return response (cannot write to KV here)
  return {
    received: body,
    message: `Hello, ${body.name}!`
  }
}
```

> **Note:** Even `post()` handlers cannot write to KV storage. State changes must happen through blockchain transactions (`contract()` function).

## Available Modules

### tana:core - Logging

```typescript
import { console } from 'tana:core'

console.log('Debug message')
console.error('Error message')
```

### tana:kv - Key-Value Storage

Cloudflare Workers-compatible KV storage. Each contract has isolated storage.

```typescript
import { kv } from 'tana:kv'

// Store values
await kv.put('username', 'alice')
await kv.put('user', { name: 'Alice', balance: 1000 })

// Read values
const username = await kv.get('username')  // 'alice'
const user = await kv.get('user', { type: 'json' })  // { name: 'Alice', ... }

// List keys
const result = await kv.list({ prefix: 'user:' })
console.log(result.keys)  // [{ name: 'user:1' }, { name: 'user:2' }]

// Delete
await kv.delete('username')
```

### tana:context - Execution Context

Available in `init()` and `contract()` functions only.

```typescript
import { context } from 'tana:context'

// Who deployed the contract
const owner = context.owner()
// { id: 'usr_...', username: 'alice', publicKey: '...' }

// Who is calling (null in init())
const caller = context.caller()
// { id: 'usr_...', username: 'bob', publicKey: '...', nonce: 1 }

// Current block info
const block = context.block()
// { height: 12345, timestamp: 1700000000, hash: '0x...', producer: 'usr_...' }

// Transaction input (null in init())
const input = context.input()
// { action: 'transfer', amount: 100 }
```

### tana:block - Blockchain Queries

Query blockchain state from any function.

```typescript
import { block } from 'tana:block'

// Current block info
const height = block.height
const timestamp = block.timestamp
const hash = block.hash

// Query users
const user = await block.getUser('usr_abc123')
const users = await block.getUser(['usr_1', 'usr_2'])  // Batch query

// Query balances
const balance = await block.getBalance('usr_abc123', 'USD')

// Query transactions
const tx = await block.getTransaction('tx_xyz789')
```

### tana:tx - Transaction Creation

Stage transactions for execution.

```typescript
import { tx } from 'tana:tx'

// Transfer funds
tx.transfer('usr_from', 'usr_to', 100, 'USD')

// Set balance directly (privileged)
tx.setBalance('usr_id', 1000, 'USD')

// Execute staged transactions
const result = await tx.execute()
// { success: true, changes: [...], gasUsed: 100 }
```

### tana:utils - External HTTP

Make HTTP requests to whitelisted domains.

```typescript
import { fetch } from 'tana:utils'

const response = await fetch('https://api.example.com/data')
const json = await response.json()
```

> **Note:** Only whitelisted domains are allowed. See [Security](#security) for details.

## Import Syntax

Both import styles work:

```typescript
// Colon style (recommended)
import { kv } from 'tana:kv'
import { console } from 'tana:core'

// Slash style (also works)
import { kv } from 'tana/kv'
import { console } from 'tana/core'
```

## Return Values

**All functions must return JSON-serializable objects.**

```typescript
// ✅ Valid returns
return { success: true }
return { items: [1, 2, 3], count: 3 }
return { user: { name: 'Alice', age: 30 } }
return { value: null }

// ❌ Invalid returns
return 'string'           // Must be object
return 42                 // Must be object
return undefined          // Use { value: null }
return { fn: () => {} }   // No functions
return { date: new Date() }  // Use timestamps
```

## Authorization Patterns

### Owner-Only Access

```typescript
export function contract() {
  const caller = context.caller()
  const owner = context.owner()

  if (!caller || caller.id !== owner.id) {
    return { error: 'Unauthorized: owner only' }
  }

  // Owner-only logic here
  return { success: true }
}
```

### Role-Based Access

```typescript
export async function contract() {
  const caller = context.caller()
  if (!caller) {
    return { error: 'Authentication required' }
  }

  // Check user role from blockchain
  const user = await block.getUser(caller.id)
  if (user?.role !== 'staff' && user?.role !== 'sovereign') {
    return { error: 'Staff access required' }
  }

  return { success: true }
}
```

## Security

### Sandboxed Execution

Contracts run in isolated V8 sandboxes:
- No filesystem access
- No system calls
- Limited network (whitelisted domains only)
- Memory limits enforced
- Gas metering (limits computation)

### Network Whitelist

External `fetch()` is limited to:
- `tana.dev` and subdomains
- `localhost` (development only)
- Additional domains as configured

### Transaction Signing

All contract deployments and calls require Ed25519 signatures:
- Owner tracked on-chain
- Replay protection via nonces
- Timestamp validation

## Testing

### Local Testing

```bash
# Test contract execution
tana run contract.ts

# Check system health
tana test
```

### HTTP Testing

After deployment, test HTTP handlers:

```bash
# GET request
curl http://localhost:8180/mycontract

# POST request
curl -X POST http://localhost:8180/mycontract \
  -H "Content-Type: application/json" \
  -d '{"name": "Alice"}'
```

## Next Steps

- [KV Storage Guide](/docs/guides/kv-storage) - Advanced storage patterns
- [CLI Commands](/docs/tana-cli/commands) - All CLI options
- [API Reference](/docs/tana-api/intro) - REST API endpoints
