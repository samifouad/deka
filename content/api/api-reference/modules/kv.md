---
title: "tana/kv Module"
description: "KV Storage API Reference"
---

Cloudflare Workers-compatible key-value storage for smart contracts.

## Import

```typescript
import { kv } from 'tana/kv'
```

## Methods

### kv.get()

Retrieve a value from storage.

```typescript
kv.get(key: string, options?: KVGetOptions): Promise<string | object | null>
```

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `key` | string | Yes | Storage key (max 512 bytes) |
| `options.type` | 'text' \| 'json' | No | Return type (default: 'text') |

**Returns:** `Promise<string | object | null>`

- Returns `null` if key doesn't exist
- Returns string when `type: 'text'` (default)
- Returns parsed object when `type: 'json'`

**Examples:**

```typescript
// Get as text (default)
const username = await kv.get('username')
console.log(username) // "alice"

// Get as JSON
const user = await kv.get('user', { type: 'json' })
console.log(user) // { name: "alice", balance: 1000 }

// Handle missing key
const missing = await kv.get('nonexistent')
if (missing === null) {
  console.log('Key not found')
}
```

---

### kv.put()

Store a value.

```typescript
kv.put(key: string, value: string | object, options?: KVPutOptions): Promise<void>
```

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `key` | string | Yes | Storage key (max 512 bytes) |
| `value` | string \| object | Yes | Value to store (max 25 MB) |
| `options` | KVPutOptions | No | Reserved for future features |

**Returns:** `Promise<void>`

**Serialization:**
- Strings: Stored as-is
- Objects: Automatically JSON-serialized

**Throws:**
- "KV key too long" - Key exceeds 512 bytes
- "KV value too large" - Value exceeds 25 MB

**Examples:**

```typescript
// Store string
await kv.put('username', 'alice')

// Store object (auto-serialized)
await kv.put('user', {
  name: 'alice',
  balance: 1000,
  roles: ['admin', 'user']
})

// Overwrite existing key
await kv.put('username', 'bob') // Previous value replaced
```

---

### kv.delete()

Delete a key from storage.

```typescript
kv.delete(key: string): Promise<void>
```

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `key` | string | Yes | Key to delete |

**Returns:** `Promise<void>`

**Notes:**
- No error if key doesn't exist
- Idempotent operation

**Examples:**

```typescript
// Delete existing key
await kv.delete('username')

// Delete non-existent key (no error)
await kv.delete('nonexistent') // Succeeds silently
```

---

### kv.list()

List keys in storage with optional filtering.

```typescript
kv.list(options?: KVListOptions): Promise<KVListResult>
```

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `options.prefix` | string | No | Filter keys by prefix |
| `options.limit` | number | No | Max keys to return (default: 1000) |

**Returns:** `Promise<KVListResult>`

```typescript
interface KVListResult {
  keys: Array<{ name: string }>
  list_complete: boolean
  cursor: string | null
}
```

**Result Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `keys` | Array<{ name: string }> | Array of matching keys |
| `list_complete` | boolean | Whether all results returned (always `true` currently) |
| `cursor` | string \| null | Pagination cursor (always `null` currently, reserved for future) |

**Examples:**

```typescript
// List all keys
const all = await kv.list()
console.log(all.keys) // [{ name: "user" }, { name: "config" }]

// List with prefix
await kv.put('user:1', { name: 'alice' })
await kv.put('user:2', { name: 'bob' })
await kv.put('config', { theme: 'dark' })

const users = await kv.list({ prefix: 'user:' })
console.log(users.keys) // [{ name: "user:1" }, { name: "user:2" }]

// Limit results
const first10 = await kv.list({ limit: 10 })
console.log(first10.keys.length) // 10 or fewer

// Check if more results available
if (!first10.list_complete) {
  // Use cursor for next page (future feature)
  const next = await kv.list({ cursor: first10.cursor })
}
```

---

## Type Definitions

### KVGetOptions

```typescript
interface KVGetOptions {
  type?: 'text' | 'json' | 'arrayBuffer' | 'stream'
}
```

**Supported types:**
- ✅ `'text'` - Return as string (default)
- ✅ `'json'` - Parse and return as object
- ❌ `'arrayBuffer'` - Future feature
- ❌ `'stream'` - Future feature

---

### KVPutOptions

```typescript
interface KVPutOptions {
  metadata?: Record<string, any>
  expirationTtl?: number
  expiration?: number
}
```

**Reserved for future features:**
- `metadata` - Custom metadata for the key
- `expirationTtl` - TTL in seconds
- `expiration` - Absolute expiration timestamp

Currently these options are accepted but have no effect.

---

### KVListOptions

```typescript
interface KVListOptions {
  prefix?: string
  limit?: number
  cursor?: string
}
```

**Fields:**
- `prefix` - Filter keys starting with prefix (default: none)
- `limit` - Max keys to return (default: 1000, max: 1000)
- `cursor` - Pagination cursor (future feature)

---

### KVListResult

```typescript
interface KVListResult {
  keys: KVListResultKey[]
  list_complete: boolean
  cursor: string | null
}
```

---

### KVListResultKey

```typescript
interface KVListResultKey {
  name: string
  expiration?: number
  metadata?: Record<string, any>
}
```

**Fields:**
- `name` - The key name
- `expiration` - Future feature (always undefined)
- `metadata` - Future feature (always undefined)

---

## Constraints & Limits

| Constraint | Limit | Error Message |
|------------|-------|---------------|
| Key length | 512 bytes | "KV key too long: X bytes (max 512)" |
| Value size | 25 MB | "KV value too large: X bytes (max 25 MB)" |
| Keys per list() | 1000 | Silently limited to 1000 |
| Keys per contract | Unlimited | Subject to storage backend limits |

---

## Namespace Isolation

KV storage is automatically isolated per-contract:

```typescript
// Contract A
await kv.put('username', 'alice')

// Contract B
await kv.get('username') // Returns null, not "alice"
```

Each contract has its own isolated namespace. Keys are internally prefixed with the contract name to prevent conflicts.

---

## Cloudflare Workers Compatibility

The API is designed to match Cloudflare Workers KV:

**Matching Features:**
- ✅ `get()` / `put()` / `delete()` / `list()` methods
- ✅ Type options ('text', 'json')
- ✅ Prefix filtering
- ✅ Limit parameter
- ✅ Same return types

**Differences:**
- ❌ No `getWithMetadata()` yet
- ❌ No TTL/expiration yet
- ❌ No pagination cursors yet
- ❌ No `'arrayBuffer'` / `'stream'` types yet

This allows contracts written for Tana to be easily adapted for Cloudflare Workers and vice versa.

---

## Examples

### Counter

```typescript
import { kv } from 'tana/kv'

export async function contract() {
  const current = await kv.get('counter', { type: 'json' })
  const count = (current?.count || 0) + 1

  await kv.put('counter', { count, lastUpdated: Date.now() })

  return { count }
}
```

### Session Management

```typescript
import { kv } from 'tana/kv'
import { context } from 'tana/context'

export async function contract() {
  const input = context.input()

  if (input.action === 'create') {
    const session = {
      userId: input.userId,
      createdAt: Date.now()
    }
    await kv.put(`session:${input.sessionId}`, session)
    return { success: true }
  }

  if (input.action === 'get') {
    const session = await kv.get(`session:${input.sessionId}`, { type: 'json' })
    return session || { error: 'Session not found' }
  }

  if (input.action === 'delete') {
    await kv.delete(`session:${input.sessionId}`)
    return { success: true }
  }
}
```

### User Preferences

```typescript
import { kv } from 'tana/kv'
import { context } from 'tana/context'

export async function get(req: Request) {
  const caller = req.tana.caller
  if (!caller) return { error: 'Login required' }

  const prefs = await kv.get(`prefs:${caller.id}`, { type: 'json' })

  return prefs || {
    theme: 'dark',
    notifications: true,
    language: 'en'
  }
}

export async function post(req: Request) {
  const caller = req.tana.caller
  if (!caller) return { error: 'Login required' }

  const prefs = req.body

  await kv.put(`prefs:${caller.id}`, prefs)

  return { success: true }
}
```

### Cache with Prefix

```typescript
import { kv } from 'tana/kv'

export async function contract() {
  // Cache multiple items
  await kv.put('cache:api:users', [{ id: 1 }, { id: 2 }])
  await kv.put('cache:api:posts', [{ id: 1 }])
  await kv.put('cache:db:config', { host: 'localhost' })

  // List only cache:api: items
  const apiCache = await kv.list({ prefix: 'cache:api:' })
  console.log(apiCache.keys)
  // [{ name: "cache:api:posts" }, { name: "cache:api:users" }]

  // Clear all api cache
  for (const key of apiCache.keys) {
    await kv.delete(key.name)
  }

  return { cleared: apiCache.keys.length }
}
```

---

## See Also

- [KV Storage Guide](/guides/kv-storage) - Usage guide with patterns
- [KV Implementation](/contributing/runtime/modules/kv) - Technical details
- [Smart Contracts](/guides/smart-contracts) - Contract development guide
