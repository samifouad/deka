# Registry Auth + Context Tasks

## Goals

- [ ] Use Bluesky OAuth as the primary user identity for linkhash.
- [ ] Map Bluesky handle to package namespace (`@handle/package`).
- [ ] Issue scoped publish tokens for CLI use (`read`, `read:write`, `read:write:delete`).
- [ ] Provide first-class PHPX auth primitives (`crypto`, `jwt`, `cookies`).
- [ ] Add framework-level request context providers (React-like provider model, server-first).

## Auth Primitives (Runtime + PHPX modules)

- [ ] `crypto` module
- [ ] random bytes/id helpers
- [ ] SHA-256 + HMAC-SHA256 helpers
- [ ] `jwt` module
- [ ] sign + verify JWT (HS256 first)
- [ ] validate `exp`, `nbf`, `iat`, `iss`, `aud`
- [ ] `cookies` module
- [ ] ergonomic read from `$_COOKIE`
- [ ] safe `Set-Cookie` builder (`HttpOnly`, `Secure`, `SameSite`, `Path`, `Domain`, `Max-Age`)

## Linkhash OAuth + Token Flow

- [ ] `/auth/login` redirect to Bluesky OAuth
- [ ] `/auth/callback` code exchange + identity fetch
- [ ] local account/org mapping from Bluesky identity
- [ ] session cookie issuance and rotation
- [ ] PAT management (create/list/revoke) with scope + expiry
- [ ] token hashing-at-rest + last-used metadata

## Registry Permission Model

- [ ] anonymous `read` for package metadata/install endpoints
- [ ] `read:write` required for publish/update
- [ ] `read:write:delete` required for delete/yank/owner mutation
- [ ] unified scope checker reused by HTTP handlers and CLI auth validation

## Framework Context Provider Model (Server-first)

- [ ] per-request context store in runtime/framework layer
- [ ] provider composition at route/layout boundary
- [ ] `auth()` helper for current request auth state
- [ ] `requireAuth()` guard helper
- [ ] `requireScope(...)` guard helper
- [ ] `useContext(...)` read helper for component render
- [ ] context re-evaluation on partial navigation requests
- [ ] minimal safe auth snapshot for hydration (no secrets/tokens)

## CLI + Registry Integration

- [ ] `deka publish` uses `LINKHASH_TOKEN` PAT
- [ ] `deka install` resolves PHPX ecosystem packages from linkhash registry
- [ ] write PHP lock entries under `deka.lock.php`
- [ ] docs for `.env` token setup and scope requirements

## Acceptance Criteria

- [ ] User logs in via Bluesky and gets a valid session in linkhash UI.
- [ ] User can create PAT with explicit scopes and expiry.
- [ ] `deka publish` succeeds with valid PAT and fails with clear scope errors otherwise.
- [ ] `deka install` pulls published PHPX package from linkhash.
- [ ] Route-level provider guards correctly allow/deny access without client-side trust.
