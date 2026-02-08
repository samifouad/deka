# Registry Auth + Context Tasks

## Goals

- [x] Use Bluesky OAuth as the primary user identity for linkhash.
- [ ] Map Bluesky handle to package namespace (`@handle/package`).
- [x] Introduce canonical immutable package IDs (`linkha.sh/lh_<sha256(...)>`).
- [x] Issue scoped publish tokens for CLI use (`read`, `read:write`, `read:write:delete`).
- [ ] Provide first-class PHPX auth primitives (`crypto`, `jwt`, `cookies`).
- [ ] Add framework-level request context providers (React-like provider model, server-first).

## Artifact Storage (Local now, R2-ready)

- [ ] Keep metadata/auth state in Postgres only.
- [ ] Store package tarball bytes in pluggable artifact backend.
- [x] Implement `ARTIFACT_BACKEND=local|r2` config.
- [x] Implement local backend as default (`ARTIFACT_LOCAL_ROOT`).
- [x] Add R2 backend adapter interface and wire R2 config/envs.
- [x] Persist artifact metadata in Postgres (`artifact_key`, `size_bytes`, `sha256`, `backend`, `mime`).
- [ ] Ensure install/download endpoints resolve artifact by key through active backend.

## Canonical Package Identity

- [ ] Define canonical hash input contract:
- [x] `lower(handle) + "/" + lower(package) + ":" + version + ":" + lock_hash`
- [x] Compute `canonical_id = "lh_" + sha256(canonical_input)`.
- [x] Persist canonical identity per published version (unique).
- [ ] Support both URL forms:
- [ ] Human alias `linkha.sh/@handle/package[@version]`
- [ ] Canonical `linkha.sh/lh_<sha256...>`
- [ ] Always resolve aliases to canonical identity in responses.

## Auth Primitives (Runtime + PHPX modules)

- [x] `crypto` module
- [x] random bytes/id helpers
- [x] SHA-256 + HMAC-SHA256 helpers
- [ ] `jwt` module
- [x] sign + verify JWT (HS256 first)
- [x] validate `exp`, `nbf`, `iat`, `iss`, `aud`
- [x] `cookies` module
- [x] ergonomic read from `$_COOKIE`
- [x] safe `Set-Cookie` builder (`HttpOnly`, `Secure`, `SameSite`, `Path`, `Domain`, `Max-Age`)

### JWT blocker note

- [x] Fix `encoding/json` object decode stability (unblocked JWT claim decode/validation in PHPX runtime).

## Linkhash OAuth + Token Flow

- [x] `/auth/login` redirect to Bluesky OAuth
- [x] `/auth/callback` code exchange + identity fetch
- [x] local account/org mapping from Bluesky identity (dev-login path in place, Bluesky callback wiring pending)
- [x] session cookie issuance and rotation
- [x] PAT management (create/list/revoke) with scope + expiry
- [x] token hashing-at-rest + last-used metadata

## Registry Permission Model

- [ ] anonymous `read` for package metadata/install endpoints
- [ ] `read:write` required for publish/update
- [ ] `read:write:delete` required for delete/yank/owner mutation
- [ ] unified scope checker reused by HTTP handlers and CLI auth validation
- [ ] add package visibility model (`public`, `private`)
- [ ] enforce private-package read access (`publisher`, `maintainer`) or scoped token

### Role Policy (Locked)

- `publisher` is the owner role (full control of org/package settings and release lifecycle)
- `maintainer` has edit/publish access but cannot transfer ownership or delete org
- `public` is read/view only access for public packages
- package visibility:
- `public` packages readable by anyone
- `private` packages readable only by org `publisher`/`maintainer` or tokens with explicit private-read grant

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

## Hardening Pass (After Bulk Auth Build)

- [x] Rotate session ID on login/privilege changes (fixation protection).
- [x] Add session idle timeout + absolute timeout enforcement.
- [ ] Enforce PKCE + OAuth state + nonce checks.
- [x] Add strict redirect URI allowlist.
- [x] Add CSRF protection for state-changing browser endpoints.

Notes:
- State-changing endpoints enforce `POST` and validate `X-CSRF-Token` against server-side session CSRF hash.
- [x] One-time PAT reveal + hashed-at-rest token storage.
- [x] Add PAT last-used metadata + bulk revoke.
- [ ] Add org role model (`publisher`, `maintainer`, `public`) with private package support.
- [ ] Add reserved namespace policy.
- [ ] Add auth/publish rate limiting and abuse guardrails.
- [ ] Add immutable audit trail for auth + publish actions.

### Reserved Namespace Defaults (Locked)

- deny org/handle registration for:
- `deka`, `linkhash`, `admin`, `support`, `system`, `root`, `security`, `api`, `auth`, `www`, `assets`, `cdn`, `status`, `ops`, `infra`, `help`, `docs`, `mail`, `postmaster`, `abuse`
- deny package names for:
- `core`, `runtime`, `php`, `phpx`, `stdlib`, `modules`, `internal`

### Rate Limit Defaults (Locked)

- auth start/callback:
- `GET /api/auth/login`, `GET /api/auth/callback`: `30/min/IP`
- token mutation:
- `POST /api/auth/token/create`, `POST /api/auth/token/revoke`, `POST /api/auth/token/revoke-all`: `10/min/user`, `30/min/IP`
- publish mutation:
- publish/update/delete endpoints: `5/min/user`, `20/min/IP`

## Acceptance Criteria

- [ ] User logs in via Bluesky and gets a valid session in linkhash UI.
- [ ] User can create PAT with explicit scopes and expiry.
- [ ] `deka publish` succeeds with valid PAT and fails with clear scope errors otherwise.
- [ ] `deka install` pulls published PHPX package from linkhash.
- [ ] Published package versions can be addressed via canonical hash ID and alias routes.
- [ ] Local artifact backend works in dev without R2.
- [ ] Switching to R2 backend does not require changing publish/install API contracts.
- [ ] Route-level provider guards correctly allow/deny access without client-side trust.
