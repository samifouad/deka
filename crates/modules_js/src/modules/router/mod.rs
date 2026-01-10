//! deka/router module
//!
//! Provides a Hono-style HTTP router for better ergonomics in route handlers.
//!
//! Usage:
//!   import { Router } from 'deka/router'
//!
//!   const app = new Router()
//!   app.get('/', (c) => c.json({ message: 'Hello' }))
//!
//!   export default app.fetch

deno_core::extension!(
    deka_router,
    esm_entry_point = "ext:deka_router/router.js",
    esm = [ dir "src/modules/router", "router.js" ],
);

pub fn init() -> deno_core::Extension {
    deka_router::init_ops_and_esm()
}
