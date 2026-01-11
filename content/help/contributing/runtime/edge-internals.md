---
title: Edge Internals
description: How the tana-edge HTTP server works internally
sidebar:
  order: 3
---

This document explains how `tana-edge` works internally for developers who want to contribute to or understand the codebase.

## Architecture Overview

Edge is a Rust HTTP server using Actix-web with embedded V8 via `deno_core`:

```
┌─────────────────────────────────────────────────────┐
│                   tana-edge                         │
├─────────────────────────────────────────────────────┤
│                                                     │
│   ┌─────────────┐    ┌─────────────────────────┐   │
│   │ Actix-web   │───►│ Request Router          │   │
│   │ HTTP Server │    │ /{contract_id}          │   │
│   └─────────────┘    └───────────┬─────────────┘   │
│                                  │                 │
│                      ┌───────────▼─────────────┐   │
│                      │ V8 Isolate Pool         │   │
│                      │ (fresh per request)     │   │
│                      └───────────┬─────────────┘   │
│                                  │                 │
│   ┌─────────────┐    ┌───────────▼─────────────┐   │
│   │ Contract    │◄───│ JavaScript Runtime      │   │
│   │ Code        │    │ (deno_core)             │   │
│   └─────────────┘    └───────────┬─────────────┘   │
│                                  │                 │
│                      ┌───────────▼─────────────┐   │
│                      │ Tana Modules            │   │
│                      │ tana:core, tana:kv, ... │   │
│                      └─────────────────────────┘   │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## Key Components

### 1. HTTP Server (Actix-web)

Located in `edge/src/main.rs`:

```rust
HttpServer::new(move || {
    App::new()
        .route("/health", web::get().to(health_check))
        .route("/{contract_id}", web::get().to(handle_get))
        .route("/{contract_id}", web::post().to(handle_post))
})
.bind(("0.0.0.0", port))?
.run()
```

### 2. V8 Isolate Creation

Each request gets a fresh V8 isolate (Cloudflare Workers model):

```rust
async fn handle_get(path: web::Path<String>, req: HttpRequest) -> impl Responder {
    let contract_id = path.into_inner();

    // Create fresh isolate
    let mut runtime = JsRuntime::new(RuntimeOptions {
        module_loader: Some(Rc::new(TanaModuleLoader)),
        ..Default::default()
    });

    // Execute contract
    let result = execute_contract(&mut runtime, &contract_id, "get").await;

    // Isolate is dropped here - no state leakage
    HttpResponse::Ok().json(result)
}
```

### 3. Module System

Tana modules are injected into the V8 runtime via `deno_core`:

```rust
// edge/src/modules.rs
pub fn get_tana_modules() -> Vec<Extension> {
    vec![
        tana_core::init_ops(),
        tana_kv::init_ops(),
        tana_block::init_ops(),
        tana_context::init_ops(),
    ]
}
```

### 4. Contract Loading

Contracts are loaded from the database:

```rust
async fn load_contract(contract_id: &str) -> Result<Contract> {
    let contract = sqlx::query_as!(
        Contract,
        "SELECT * FROM contracts WHERE id = $1 AND is_active = true",
        contract_id
    )
    .fetch_one(&pool)
    .await?;

    Ok(contract)
}
```

## Request Flow

1. **HTTP Request** arrives at Actix-web
2. **Router** extracts `contract_id` from path
3. **Contract Loader** fetches contract code from database
4. **V8 Isolate** is created with Tana modules
5. **Contract Code** is executed (`get()` or `post()`)
6. **Response** is serialized to JSON
7. **Isolate** is dropped (memory freed)

## Security Model

### Sandbox Isolation

Each request runs in a completely isolated V8 environment:

- **No filesystem access** - `fs` module not available
- **No network access** - Only whitelisted `fetch` via `tana:utils`
- **No process access** - No `child_process`, `os`, etc.
- **Memory limits** - V8 heap size constrained

### Read-Only Execution

Edge handlers (`get`/`post`) cannot write to blockchain state:

```rust
// KV operations in edge are read-only
impl TanaKv for EdgeKv {
    async fn get(&self, key: &str) -> Option<String> {
        // Allowed - reads from database
        self.db.get_kv(self.contract_id, key).await
    }

    async fn put(&self, key: &str, value: &str) -> Result<()> {
        // Blocked in edge mode
        Err(Error::ReadOnlyMode)
    }
}
```

## Performance Optimizations

### Isolate Pooling (Future)

Currently each request creates a new isolate. Future optimization:

```rust
// Potential isolate pool implementation
struct IsolatePool {
    available: Vec<JsRuntime>,
    max_size: usize,
}

impl IsolatePool {
    fn acquire(&mut self) -> JsRuntime {
        self.available.pop().unwrap_or_else(|| create_isolate())
    }

    fn release(&mut self, runtime: JsRuntime) {
        if self.available.len() < self.max_size {
            // Reset isolate state
            runtime.clear_context();
            self.available.push(runtime);
        }
        // Otherwise drop
    }
}
```

### Contract Caching

Compiled contract code is cached:

```rust
lazy_static! {
    static ref CONTRACT_CACHE: RwLock<HashMap<String, CompiledContract>> =
        RwLock::new(HashMap::new());
}
```

## Building from Source

```bash
cd edge
cargo build --release

# Binary at: target/release/tana-edge
```

### Dependencies

- `deno_core` - V8 JavaScript runtime
- `actix-web` - HTTP server
- `sqlx` - Database access
- `tokio` - Async runtime

## Testing

```bash
# Run tests
cd edge
cargo test

# Integration tests
cargo test --features integration
```

## Contributing

Key files:

| File | Purpose |
|------|---------|
| `src/main.rs` | HTTP server setup, routing |
| `src/runtime.rs` | V8 isolate management |
| `src/modules/` | Tana module implementations |
| `src/contract.rs` | Contract loading and execution |

## Related

- [Runtime Internals](/docs/contributing/runtime/intro/) - How tana-runtime works
- [Architecture](/docs/contributing/architecture/) - System overview
