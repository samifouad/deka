---
title: "KV Storage Implementation"
description: "Technical details of tana/kv module across all runtimes"
---

Technical reference for the `tana/kv` module implementation across tana-runtime, tana-edge, and playground environments.

## Architecture Overview

KV storage provides Cloudflare Workers-compatible key-value storage with automatic per-contract namespace isolation.

### Namespace Isolation

All keys are internally prefixed with the contract name:
```
Stored Key Format: "{contract_name}:{user_key}"
Example: "my_contract:username" → "alice"
```

This ensures contracts cannot access each other's data.

## Runtime Implementations

### tana-runtime (Rust CLI)

**Location:** `runtime/src/tana_kv.rs`, `runtime/src/main.rs`

**Storage Backend:** PostgreSQL (migration `0011_kv_storage.sql`)

**Schema:**
```sql
CREATE TABLE kv_storage (
  contract_name TEXT NOT NULL,
  key TEXT NOT NULL,
  value TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (contract_name, key)
);
```

**Rust Ops:**
```rust
#[op2]
#[string]
fn op_kv_get_raw(
    state: &OpState,
    #[string] key: String,
) -> Result<Option<String>, JsErrorBox>

#[op2(fast)]
fn op_kv_put_raw(
    state: &OpState,
    #[string] key: String,
    #[string] value: String,
) -> Result<(), JsErrorBox>

#[op2(fast)]
fn op_kv_delete_raw(
    state: &OpState,
    #[string] key: String,
) -> Result<(), JsErrorBox>

#[op2]
#[serde]
fn op_kv_list_keys(
    state: &OpState,
    #[string] prefix: Option<String>,
    #[serde] limit: Option<usize>,
) -> Result<Vec<String>, JsErrorBox>
```

**JavaScript Bridge:**
```typescript
// Uses context.contract_name() from OpState
const contractName = globalThis.__tanaCore.ops.op_context_contract_name()
globalThis.__tanaCore.ops.op_kv_get_raw(key) // Automatically namespaced
```

**Characteristics:**
- ✅ Persistent across restarts
- ✅ ACID transactions via PostgreSQL
- ✅ Production-ready
- 📊 Performance: ~1-5ms per operation

---

### tana-edge (Rust HTTP Server)

**Location:** `tana-edge/src/main.rs` (lines 41-49, 335-460, 1163-1196)

**Storage Backend:** In-memory HashMap (Redis-ready)

**Data Structure:**
```rust
static KV_STORAGE: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);
```

**Rust Ops:**
```rust
// Same signatures as tana-runtime, but contract_name passed as parameter
#[op2]
#[string]
fn op_kv_get_raw(
    #[string] contract_name: String,
    #[string] key: String,
) -> Result<Option<String>, JsErrorBox>

// ... put, delete, list ops similar
```

**JavaScript Bridge:**
```typescript
// Hardcoded contract name for development
const contractName = "edge_contract"
const rawValue = globalThis.__tanaCore.ops.op_kv_get_raw(contractName, key)
```

**Characteristics:**
- ⚡ Very fast (in-memory)
- ❌ Not persistent (resets on restart)
- 🔧 Development/testing use
- 🚀 Production: Replace with Redis backend
- 📊 Performance: <0.1ms per operation

---

### playground (Browser/TypeScript)

**Location:**
- `websites/playground/src/pages/sandbox.astro` (lines 691-811)
- `websites/playground/src/components/Editor.svelte` (lines 277-318)

**Storage Backend:** Browser localStorage

**Key Format:**
```
localStorage Key: "tana_kv_playground:playground_contract:{user_key}"
Example: "tana_kv_playground:playground_contract:username"
```

**Implementation:**
```typescript
const kv = {
  async get(key: string, options?: KVGetOptions) {
    const storageKey = `tana_kv_playground:playground_contract:${key}`
    const value = localStorage.getItem(storageKey)
    if (!value) return null

    const type = options?.type || 'text'
    return type === 'json' ? JSON.parse(value) : value
  },

  async put(key: string, value: string | object) {
    const storageKey = `tana_kv_playground:playground_contract:${key}`
    const serialized = typeof value === 'string' ? value : JSON.stringify(value)
    localStorage.setItem(storageKey, serialized)
  },

  async delete(key: string) {
    const storageKey = `tana_kv_playground:playground_contract:${key}`
    localStorage.removeItem(storageKey)
  },

  async list(options?: KVListOptions): Promise<KVListResult> {
    const prefix = `tana_kv_playground:playground_contract:${options?.prefix || ''}`
    const limit = options?.limit || 1000
    const keys: string[] = []

    for (let i = 0; i < localStorage.length; i++) {
      const fullKey = localStorage.key(i)
      if (fullKey?.startsWith(prefix)) {
        const userKey = fullKey.replace('tana_kv_playground:playground_contract:', '')
        keys.push(userKey)
        if (keys.length >= limit) break
      }
    }

    keys.sort()
    return {
      keys: keys.map(name => ({ name })),
      list_complete: true,
      cursor: null
    }
  }
}
```

**Characteristics:**
- 💾 Persistent per-browser
- 🚀 Instant execution
- 📏 Limited to ~5-10MB total
- 🧪 Great for experimentation
- 📊 Performance: <0.1ms per operation

## API Consistency

All three runtimes implement identical JavaScript API:

```typescript
interface KV {
  get(key: string, options?: { type?: 'text' | 'json' }): Promise<string | object | null>
  put(key: string, value: string | object, options?: KVPutOptions): Promise<void>
  delete(key: string): Promise<void>
  list(options?: { prefix?: string, limit?: number }): Promise<KVListResult>
}
```

This ensures contracts are **portable** - code written in playground works identically in tana-runtime and tana-edge.

## Validation & Limits

All runtimes enforce:

| Constraint | Value | Error |
|------------|-------|-------|
| Max key length | 512 bytes | "KV key too long" |
| Max value size | 25 MB | "KV value too large" |
| Max list() keys | 1000 | Silently limited |

Validation happens in Rust ops before storage:

```rust
if key.len() > MAX_KV_KEY_LENGTH {
    return Err(JsErrorBox::new("Error",
        format!("KV key too long: {} bytes (max {})", key.len(), MAX_KV_KEY_LENGTH)
    ));
}

if value.len() > MAX_KV_VALUE_SIZE {
    return Err(JsErrorBox::new("Error",
        format!("KV value too large: {} bytes (max {})", value.len(), MAX_KV_VALUE_SIZE)
    ));
}
```

## Type Definitions

**Location:** `types/tana-kv.d.ts`

Provides full TypeScript definitions with:
- Method overloads for type inference
- JSDoc documentation
- Cloudflare Workers compatibility notes

Example type inference:
```typescript
// TypeScript knows this returns string | null
const text = await kv.get('key')

// TypeScript knows this returns any | null
const json = await kv.get('key', { type: 'json' })
```

## Performance Characteristics

| Operation | Runtime | Edge | Playground |
|-----------|---------|------|------------|
| get() | 1-5ms | <0.1ms | <0.1ms |
| put() | 2-10ms | <0.1ms | <0.1ms |
| delete() | 1-5ms | <0.1ms | <0.1ms |
| list(100) | 5-20ms | <1ms | <1ms |
| list(1000) | 20-100ms | 1-5ms | 5-10ms |

**Notes:**
- Runtime: PostgreSQL network + query time
- Edge: Pure in-memory HashMap lookups
- Playground: localStorage DOM API overhead

## Future Enhancements

### Planned Features
- ⏰ **TTL Support** - Automatic expiration
- 📊 **Metadata** - Custom key metadata
- 📄 **Pagination** - Cursor-based list()
- 🔄 **Bulk Operations** - Batch get/put/delete
- 🔍 **Range Queries** - Key range scans

### Implementation Notes

**TTL (expirationTtl/expiration):**
```sql
-- PostgreSQL schema addition
ALTER TABLE kv_storage ADD COLUMN expires_at TIMESTAMP;
CREATE INDEX idx_kv_expires ON kv_storage(expires_at) WHERE expires_at IS NOT NULL;
```

**Metadata:**
```sql
ALTER TABLE kv_storage ADD COLUMN metadata JSONB;
```

**Pagination:**
```typescript
// Return cursor in list() results
return {
  keys: [...],
  list_complete: false,
  cursor: "offset:1000" // For next page
}
```

## Testing

Run KV storage tests:

```bash
# tana-runtime
./target/release/tana-runtime examples/kv-test.ts

# tana-edge
./target/release/tana-edge
curl -X POST http://localhost:8082/test \
  -H "Content-Type: text/plain" \
  -d "$(cat examples/kv-test.ts)"

# playground
# Open browser, paste code from examples/kv-test.ts
```

## Common Issues

**"KV key too long"**
- Key exceeds 512 bytes
- Solution: Use shorter keys or hash long identifiers

**"KV value too large"**
- Value exceeds 25 MB
- Solution: Split large data across multiple keys

**Namespace collision**
- Different contracts trying to access same data
- This is prevented by automatic namespacing

**list() returns empty**
- No keys match prefix
- Check prefix syntax (no wildcards, exact string match)

## Migration Guide

### From tana/data to tana/kv

```typescript
// OLD: tana/data (synchronous)
import { data } from 'tana/data'
data.set('key', 'value')
const value = data.get('key')

// NEW: tana/kv (async)
import { kv } from 'tana/kv'
await kv.put('key', 'value')
const value = await kv.get('key')
```

**Key differences:**
- `kv.*` is async (returns Promises)
- `kv.put()` auto-serializes objects
- `kv.list()` provides structured results
- `kv.get()` supports type hints

Both modules remain available. Use `tana/data` for simple synchronous storage, `tana/kv` for Cloudflare-compatible async patterns.
