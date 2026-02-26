---
title: módulo tana/kv
description: Referencia de la API de Almacenamiento KV
---
Almacenamiento de clave-valor compatible con Cloudflare Workers para contratos inteligentes.

## Importar

```typescript
import { kv } from 'tana/kv'
```

## Métodos

### kv.get()

Recuperar un valor del almacenamiento.

```typescript
kv.get(key: string, options?: KVGetOptions): Promise<string | object | null>
```

**Parámetros:**

| Nombre | Tipo | Requerido | Descripción |
|--------|------|-----------|-------------|
| `key` | string | Sí | Clave de almacenamiento (máx. 512 bytes) |
| `options.type` | 'text' \| 'json' | No | Tipo de retorno (predeterminado: 'text') |

**Devuelve:** `Promise<string | object | null>`

- Devuelve `null` si la clave no existe
- Devuelve una cadena cuando `type: 'text'` (predeterminado)
- Devuelve un objeto analizado cuando `type: 'json'`

**Ejemplos:**

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

Almacenar un valor.

```typescript
kv.put(key: string, value: string | object, options?: KVPutOptions): Promise<void>
```

**Parámetros:**

| Nombre | Tipo | Requerido | Descripción |
|--------|------|-----------|-------------|
| `key` | string | Sí | Clave de almacenamiento (máx. 512 bytes) |
| `value` | string \| object | Sí | Valor a almacenar (máx. 25 MB) |
| `options` | KVPutOptions | No | Reservado para futuras características |

**Devuelve:** `Promise<void>`

**Serialización:**
- Cadenas: Almacenadas tal cual
- Objetos: Serializados automáticamente en JSON

**Lanza:**
- "Clave KV demasiado larga" - La clave excede 512 bytes
- "Valor KV demasiado grande" - El valor excede 25 MB

**Ejemplos:**

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

Eliminar una clave del almacenamiento.

```typescript
kv.delete(key: string): Promise<void>
```

**Parámetros:**

| Nombre | Tipo | Requerido | Descripción |
|--------|------|-----------|-------------|
| `key` | string | Sí | Clave a eliminar |

**Devuelve:** `Promise<void>`

**Notas:**
- No hay error si la clave no existe
- Operación idempotente

**Ejemplos:**

```typescript
// Delete existing key
await kv.delete('username')

// Delete non-existent key (no error)
await kv.delete('nonexistent') // Succeeds silently
```

---

### kv.list()

Listar claves en el almacenamiento con filtrado opcional.

```typescript
kv.list(options?: KVListOptions): Promise<KVListResult>
```

**Parámetros:**

| Nombre | Tipo | Requerido | Descripción |
|--------|------|-----------|-------------|
| `options.prefix` | string | No | Filtrar claves por prefijo |
| `options.limit` | number | No | Máx. claves a devolver (predeterminado: 1000) |

**Devuelve:** `Promise<KVListResult>`

```typescript
interface KVListResult {
  keys: Array<{ name: string }>
  list_complete: boolean
  cursor: string | null
}
```

**Campos de Resultado:**

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `keys` | Array<{ name: string }> | Array de claves coincidentes |
| `list_complete` | boolean | Si todos los resultados fueron devueltos (siempre `true` actualmente) |
| `cursor` | string \| null | Cursor de paginación (siempre `null` actualmente, reservado para futuro) |

**Ejemplos:**

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

## Definiciones de Tipo

### KVGetOptions

```typescript
interface KVGetOptions {
  type?: 'text' | 'json' | 'arrayBuffer' | 'stream'
}
```

**Tipos soportados:**
- ✅ `'text'` - Devolver como cadena (predeterminado)
- ✅ `'json'` - Analizar y devolver como objeto
- ❌ `'arrayBuffer'` - Característica futura
- ❌ `'stream'` - Característica futura

---

### KVPutOptions

```typescript
interface KVPutOptions {
  metadata?: Record<string, any>
  expirationTtl?: number
  expiration?: number
}
```

**Reservado para futuras características:**
- `metadata` - Metadatos personalizados para la clave
- `expirationTtl` - TTL en segundos
- `expiration` - Marca de tiempo de expiración absoluta

Actualmente, estas opciones son aceptadas pero no tienen efecto.

---

### KVListOptions

```typescript
interface KVListOptions {
  prefix?: string
  limit?: number
  cursor?: string
}
```

**Campos:**
- `prefix` - Filtrar claves que comienzan con el prefijo (predeterminado: ninguno)
- `limit` - Máx. claves a devolver (predeterminado: 1000, máx: 1000)
- `cursor` - Cursor de paginación (característica futura)

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

**Campos:**
- `name` - El nombre de la clave
- `expiration` - Característica futura (siempre indefinido)
- `metadata` - Característica futura (siempre indefinido)

---

## Restricciones y Límites

| Restricción | Límite | Mensaje de Error |
|-------------|--------|------------------|
| Longitud de clave | 512 bytes | "Clave KV demasiado larga: X bytes (máx 512)" |
| Tamaño de valor | 25 MB | "Valor KV demasiado grande: X bytes (máx 25 MB)" |
| Claves por list() | 1000 | Limitado silenciosamente a 1000 |
| Claves por contrato | Ilimitado | Sujeto a límites del backend de almacenamiento |

---

## Aislamiento de Espacio de Nombres

El almacenamiento KV está automáticamente aislado por contrato:

```typescript
// Contract A
await kv.put('username', 'alice')

// Contract B
await kv.get('username') // Returns null, not "alice"
```

Cada contrato tiene su propio espacio de nombres aislado. Las claves se prefijan internamente con el nombre del contrato para evitar conflictos.

---

## Compatibilidad con Cloudflare Workers

La API está diseñada para coincidir con Cloudflare Workers KV:

**Características Coincidentes:**
- ✅ Métodos `get()` / `put()` / `delete()` / `list()`
- ✅ Opciones de tipo ('text', 'json')
- ✅ Filtrado por prefijo
- ✅ Parámetro de límite
- ✅ Mismos tipos de retorno

**Diferencias:**
- ❌ No hay `getWithMetadata()` aún
- ❌ No hay TTL/expiración aún
- ❌ No hay cursores de paginación aún
- ❌ No hay tipos `'arrayBuffer'` / `'stream'` aún

Esto permite que los contratos escritos para Tana se adapten fácilmente para Cloudflare Workers y viceversa.

---

## Ejemplos

### Contador

```typescript
import { kv } from 'tana/kv'

export async function contract() {
  const current = await kv.get('counter', { type: 'json' })
  const count = (current?.count || 0) + 1

  await kv.put('counter', { count, lastUpdated: Date.now() })

  return { count }
}
```

### Gestión de Sesiones

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

### Preferencias de Usuario

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

### Caché con Prefijo

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

## Ver También

- [Guía de Almacenamiento KV](/guides/kv-storage) - Guía de uso con patrones
- [Implementación de KV](/contributing/runtime/modules/kv) - Detalles técnicos
- [Contratos Inteligentes](/guides/smart-contracts) - Guía de desarrollo de contratos
