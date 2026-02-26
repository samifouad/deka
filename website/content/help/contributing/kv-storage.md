---
title: "KV Storage"
description: "Fast key-value storage for smart contracts"
---

Tana provides Cloudflare Workers-compatible KV storage for smart contracts. Each contract gets isolated storage with automatic namespacing.

## Quick Start

```typescript
import { kv } from 'tana/kv'

// Store text
await kv.put('username', 'alice')

// Retrieve text
const name = await kv.get('username')
console.log(name) // "alice"

// Store JSON
await kv.put('user', { name: 'alice', balance: 1000 })

// Retrieve JSON
const user = await kv.get('user', { type: 'json' })
console.log(user.balance) // 1000

// Delete
await kv.delete('username')

// List keys
const result = await kv.list()
console.log(result.keys) // [{ name: "user" }]
```

## API Reference

### `kv.get(key, options?)`

Retrieve a value.

**Parameters:**
- `key` (string): Storage key
- `options.type` ('text' | 'json'): Return type (default: 'text')

**Returns:** `Promise<string | object | null>`

```typescript
// Get as text (default)
const text = await kv.get('message')

// Get as JSON
const data = await kv.get('config', { type: 'json' })

// Missing key returns null
const missing = await kv.get('nonexistent') // null
```

### `kv.put(key, value, options?)`

Store a value. Objects are automatically JSON-serialized.

**Parameters:**
- `key` (string): Storage key (max 512 bytes)
- `value` (string | object): Value to store (max 25 MB)
- `options` (optional): Reserved for future features (TTL, metadata)

**Returns:** `Promise<void>`

```typescript
// Store string
await kv.put('status', 'active')

// Store object (auto-serialized)
await kv.put('user', { id: 1, name: 'alice' })
```

### `kv.delete(key)`

Delete a key. No error if key doesn't exist.

**Parameters:**
- `key` (string): Key to delete

**Returns:** `Promise<void>`

```typescript
await kv.delete('session')
```

### `kv.list(options?)`

List keys, optionally filtered by prefix.

**Parameters:**
- `options.prefix` (string): Filter by prefix
- `options.limit` (number): Max keys to return (default: 1000)

**Returns:** `Promise<{ keys: Array<{ name: string }>, list_complete: boolean, cursor: string | null }>`

```typescript
// List all keys
const all = await kv.list()

// List with prefix
const users = await kv.list({ prefix: 'user:' })

// Limit results
const first10 = await kv.list({ limit: 10 })
```

## Common Patterns

### Counter

```typescript
const current = await kv.get('counter', { type: 'json' })
const count = (current?.count || 0) + 1
await kv.put('counter', { count })
```

### Session Storage

```typescript
// Create session
await kv.put(`session:${sessionId}`, {
  userId: 'alice',
  createdAt: Date.now()
})

// Get session
const session = await kv.get(`session:${sessionId}`, { type: 'json' })

// Delete session
await kv.delete(`session:${sessionId}`)
```

### Cache with Prefix

```typescript
// Store multiple items
await kv.put('cache:user:1', { name: 'alice' })
await kv.put('cache:user:2', { name: 'bob' })
await kv.put('cache:post:1', { title: 'Hello' })

// List only cache:user: items
const users = await kv.list({ prefix: 'cache:user:' })
// Result: [{ name: "cache:user:1" }, { name: "cache:user:2" }]
```

### User Preferences

```typescript
import { context } from 'tana/context'
import { kv } from 'tana/kv'

export async function get() {
  const caller = context.caller()
  const prefs = await kv.get(`prefs:${caller}`, { type: 'json' })

  return prefs || { theme: 'dark', notifications: true }
}

export async function post(req) {
  const caller = context.caller()
  const prefs = req.body

  await kv.put(`prefs:${caller}`, prefs)
  return { success: true }
}
```

## Limits & Constraints

- **Key length:** 512 bytes max
- **Value size:** 25 MB max
- **Keys per list():** 1000 max (use prefix to narrow)
- **Namespace:** Automatic per-contract isolation
- **TTL:** Not yet supported (coming soon)
- **Metadata:** Not yet supported (coming soon)

## Storage Backends

KV storage uses different backends depending on environment:

| Environment | Backend | Notes |
|-------------|---------|-------|
| tana-runtime | PostgreSQL | Persistent across restarts |
| tana-edge | In-memory | Fast but not persistent |
| Playground | localStorage | Browser-based, persistent per-browser |

All backends provide identical API and behavior.

## Best Practices

✅ **Use prefixes** to organize related keys:
```typescript
await kv.put('user:1:profile', data)
await kv.put('user:1:settings', data)
```

✅ **Check for null** when retrieving:
```typescript
const data = await kv.get('key', { type: 'json' })
if (!data) {
  // Handle missing key
}
```

✅ **Use type parameter** for JSON to get proper typing:
```typescript
const user = await kv.get('user', { type: 'json' }) as User
```

❌ **Don't exceed limits:**
```typescript
// BAD: Key too long
await kv.put('x'.repeat(1000), 'value') // Error!

// GOOD: Keep keys short
await kv.put('user:1', 'value')
```

❌ **Don't store sensitive data** without encryption:
```typescript
// BAD: Plain text password
await kv.put('password', 'secret123')

// GOOD: Hash or encrypt first
await kv.put('password_hash', hashPassword('secret123'))
```

## Next Steps

- See [Smart Contracts](/guides/smart-contracts) for full contract development guide
- Check [examples/kv-demo.ts](https://github.com/yourusername/tana/blob/main/examples/kv-demo.ts) for complete examples
- Read [KV Patterns](/guides/patterns/kv-patterns) for advanced techniques
