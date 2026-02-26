---
title: Runtime Architecture
description: V8-based TypeScript runtime for smart contracts
sidebar:
  order: 1
---

Tana Runtime is a lightweight JavaScript/TypeScript runtime built on **deno_core** (Deno's V8 engine). It provides a sandboxed execution environment for TypeScript smart contracts, similar to Cloudflare Workers.

## Overview

The runtime creates a secure sandbox for executing TypeScript with:

1. **V8 JavaScript Engine** - Via deno_core for native performance
2. **TypeScript Compiler** - Dynamically transpiles TypeScript to JavaScript
3. **Custom Module System** - Virtual modules like `tana:core`, `tana:data`, `tana:utils`
4. **Sandbox Isolation** - Hides Deno API, exposes only whitelisted functionality
5. **Dual Environment Support** - Same code runs in CLI runtime and browser playground

## Quick Start

```bash
# Run the Rust CLI runtime
cd runtime
cargo run

# Or run a specific script
cargo run example.ts
```

## Architecture

### Runtime Stack

```
┌─────────────────────────────────────┐
│   TypeScript Smart Contracts        │
├─────────────────────────────────────┤
│   Virtual Modules (tana:core, etc)  │
├─────────────────────────────────────┤
│   TypeScript Compiler (typescript.js)│
├─────────────────────────────────────┤
│   Sandboxed V8 Runtime (deno_core)  │
├─────────────────────────────────────┤
│   Rust Runtime / Browser Playground │
└─────────────────────────────────────┘
```

### Core Components

**src/main.rs** - Main runtime entry point
- Initializes `JsRuntime` with deno_core
- Loads internal modules
- Defines ops (Rust↔JavaScript bridge)
- Bootstraps V8 with tana-globals.ts

**src/lib.rs** - Library exports for WASM builds

**build.rs** - Build script
- Extracts version info (Tana, Deno Core, V8)
- Embeds metadata at compile time

**tana-globals.ts** - Bootstrap code
- Defines `globalThis.tana`
- Hides `Deno` API for isolation
- Sets up module loader

**typescript.js** - Embedded TypeScript compiler
- Bundled compiler for `.ts` → `.js` transpilation
- Loaded into V8 at runtime

### Execution Flow

1. `main.rs` bootstraps V8 + deno_core runtime
2. Loads `typescript.js` compiler into V8
3. Injects `tana-globals.ts` and hides `Deno` API
4. Registers virtual modules (`tana:core`, `tana:data`, etc.)
5. Reads user script (e.g., `example.ts`)
6. Transpiles TypeScript → JavaScript
7. Executes in sandbox with isolated state

## Implemented Features

### Storage API (`tana:data`)

**Status:** ✅ Feature parity achieved (works identically in both environments)

```typescript
import { data } from 'tana:data'

// Set values (staged, not committed yet)
await data.set('counter', 42)
await data.set('user', { name: 'Alice', balance: 1000 })

// Read values
const count = await data.get('counter')  // 42
const user = await data.get('user')      // { name: 'Alice', balance: 1000 }

// Pattern matching
await data.set('user:1:name', 'Bob')
await data.set('user:2:name', 'Charlie')
const userKeys = await data.keys('user:*')  // ['user:1:name', 'user:2:name']

// Atomic commit (all or nothing)
await data.commit()
```

**Implementation:**
- **Playground:** localStorage backend with full persistence
- **Rust Runtime:** In-memory HashMap (resets each run)
- **Planned:** Redis backend for production persistence

**Storage Limits:**
- Max key size: 256 bytes
- Max value size: 10 KB
- Max total size: 100 KB per contract
- Max keys: 1000

**Key Features:**
- Staging buffer with atomic commits
- Size validation before persistence
- JSON auto-serialization
- Glob pattern matching support

### Fetch API (`tana:utils`)

**Status:** ✅ Working in both environments

```typescript
import { fetch } from 'tana:utils'

// Whitelisted domains only
const data = await fetch('https://pokeapi.co/api/v2/pokemon/ditto')
console.log(data)
```

**Security:**
- Domain whitelist: `pokeapi.co`, `tana.dev`, `localhost`, etc.
- Rust: reqwest + tokio async runtime
- Playground: browser fetch with same whitelist

### Console API (`tana:core`)

**Status:** ✅ Working in both environments

```typescript
import { console } from 'tana:core'

console.log('Hello from Tana!')
console.error('Error message')

// Runtime version info
import { version } from 'tana:core'
console.log(version.tana)       // "0.1.0"
console.log(version.deno_core)  // "0.338"
console.log(version.v8)         // "13.2.281.5"
```

## Feature Parity Strategy

**Dual Environment Support:**

1. **Rust CLI Runtime** (`cargo run`) - Production-ready V8 sandbox
2. **Browser Playground** (Astro/Svelte web app) - Development/testing UI

**Synchronization Points:**
- `src/main.rs` - Rust ops definitions
- `playground/src/pages/sandbox.astro` - JavaScript module implementations
- `types/*.d.ts` - Shared TypeScript definitions
- `playground/src/components/Editor.svelte` - Monaco type definitions

**Rule:** If it's in the type definitions, it MUST work in BOTH environments.

## File Structure

### Core Runtime Files

| File | Description |
|------|-------------|
| **src/main.rs** | Main entry point, initializes JsRuntime, defines ops |
| **src/lib.rs** | Library exports for WASM builds |
| **build.rs** | Build script extracting version info |
| **Cargo.toml** | Rust dependencies and package configuration |
| **typescript.js** | Embedded TypeScript compiler |
| **tana-globals.ts** | Bootstrap defining `globalThis.tana` |

### TypeScript & Type Definitions

| File | Description |
|------|-------------|
| **types/tana.d.ts** | Type declarations for `tana` and `tana:core` |
| **types/tana-data.d.ts** | Type declarations for `tana:data` storage |
| **types/tana-utils.d.ts** | Type declarations for `tana:utils` |
| **tsconfig.json** | TypeScript config mapping `"tana:*"` paths |

### Example & Test Files

| File | Description |
|------|-------------|
| **example.ts** | Example TypeScript program |
| **counter-test.ts** | Simple counter demonstrating storage API |
| **test-storage.ts** | Comprehensive storage API tests |
| **test-fetch.ts** | Fetch API tests |

## Technical Stack

### Rust Dependencies

```toml
deno_core = "0.338"          # V8 runtime
deno_error = "0.5.7"         # Error handling
reqwest = "0.12"             # HTTP client
tokio = "1"                  # Async runtime
redis = "0.27"               # Database client (planned)
serde_json = "1.0"           # JSON handling
wasm-bindgen = "0.2"         # WASM support
```

## Example Smart Contract

```typescript
import { console } from 'tana:core'
import { data } from 'tana:data'

// Simple token transfer contract
async function transfer(from: string, to: string, amount: number) {
  // Read current balances
  const fromBalance = parseInt((await data.get(`balance:${from}`)) as string)
  const toBalance = parseInt((await data.get(`balance:${to}`)) as string)

  // Validate
  if (fromBalance < amount) {
    throw new Error('Insufficient balance')
  }

  // Update balances
  await data.set(`balance:${from}`, String(fromBalance - amount))
  await data.set(`balance:${to}`, String(toBalance + amount))

  // Commit atomically (both balances updated or neither)
  await data.commit()

  console.log(`Transferred ${amount} from ${from} to ${to}`)
}

// Initialize balances
await data.set('balance:alice', '1000')
await data.set('balance:bob', '500')
await data.commit()

// Execute transfer
await transfer('alice', 'bob', 200)

// Check new balances
console.log('Alice:', await data.get('balance:alice'))  // '800'
console.log('Bob:', await data.get('balance:bob'))      // '700'
```

## Testing

### CLI Runtime

```bash
# Run example script
cargo run

# Run specific test
cargo run counter-test.ts
cargo run test-storage.ts
cargo run test-fetch.ts
```

### Browser Playground

```bash
cd playground
npm install
npm run dev
```

Open http://localhost:4322/ and use the Monaco editor to write and run TypeScript contracts.

**Inspect Storage:**
1. Open Browser DevTools (F12)
2. Go to **Application > Local Storage**
3. Look for keys starting with `tana:data:`

## Development Status

### Working ✅

- V8 runtime bootstrapping
- TypeScript transpilation
- Module system (`tana:core`, `tana:data`, `tana:utils`)
- Storage API with staging + atomic commits
- Fetch API with domain whitelist
- Browser playground with Monaco editor
- Feature parity between CLI and playground
- JSON auto-serialization for storage
- Glob pattern matching for keys

### In Progress 🚧

- Redis backend for persistent storage (Rust runtime)
- Docker setup for database services
- Blockchain integration (`tana:blockchain` module)
- Data View tab in playground UI
- Gas costs and storage rent model

### Planned 📋

- PostgreSQL integration for ledger state
- Ed25519 signature verification
- Block batching and Merkle proofs
- Multi-currency account system
- Contract deployment and versioning
- API endpoints for network access

## Next Steps

1. **Redis Integration** - Add persistent storage backend to Rust runtime
2. **Docker Compose** - Setup Redis + PostgreSQL services
3. **Data View Tab** - Add UI to visualize storage in playground
4. **Blockchain Module** - Implement `tana:blockchain` API
5. **Gas Costs** - Add metering for storage operations
6. **Contract Deployment** - Support uploading and versioning contracts

## Important Notes

- `globalThis.Deno` is deleted to ensure true sandbox isolation
- The current module system will be replaced by a Rust `ModuleLoader`
- Storage limits prevent abuse (similar to Ethereum gas model)
- Same TypeScript code runs in CLI and browser playground
- Contracts are deterministic state machines identified by code hash

## Related Documentation

- [Smart Contract Examples](/guides/smart-contracts/) - Writing contracts
- [Edge Server](/contributing/edge/intro/) - Production contract execution
- [Environment Variables](/contributing/env-vars/) - Configuration
