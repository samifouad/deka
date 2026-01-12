# Optimizations

This is a running list of runtime performance opportunities for serve/run.

## Bun findings (performance inspiration)
- Bun's HTTP server is native and uWebSockets-based, not JS-based.
- Request/response objects are backed by native structs (no JS Response in hot path).
- Headers are lazy and stored natively; they are only materialized to JS when accessed.
- Minimal JSON serialization between native and JS.

## Deka optimization plan (next steps)
1) Avoid JSON -> serde -> V8 conversions for serve requests.
   - Build `__requestData` directly in V8 from native request parts.
   - Likely highest impact: cut per-request allocation and serialization overhead.
2) Lazy headers for the adapter path.
   - Provide a lightweight header accessor and build the map only if Express reads it.
3) Avoid URL parsing in adapter hot path.
   - Pass raw path string when already normalized and only parse if needed.
4) Reuse buffers / reduce allocations in adapter.
   - Cache TextEncoder, reuse arrays for small responses, avoid per-request object churn.

## Why native request objects matter (trim the prelude)
Right now we build a full JS request object on every request (url, method, headers map, body),
then Express runs on top of that. That prelude work is heavy and happens even when Express
doesn't touch headers or body.

The native approach (Bun-style) is to keep the request in Rust and expose lazy getters:
- `req.url` and `req.method` return strings directly from native fields.
- `req.headers` builds a JS map only when accessed.
- `req.text()` / `req.json()` decode the body only when called.

This trims per-request allocations and avoids JSON -> V8 conversion in the hot path.

Plan for next session:
- Introduce a native-backed request object with lazy properties for url/method/headers/body.
- Replace `__requestData` construction with this object for serve-mode requests.

## Notes
- Express in serve mode uses the adapter; performance ceiling is still limited by Express itself.
- For benchmarks, ensure request path stays in serve mode (runtime owns listener).
