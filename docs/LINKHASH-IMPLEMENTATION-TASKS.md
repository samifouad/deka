# Linkhash Implementation Track (App Integration)

This track is linkhash app/product work that consumes runtime capabilities.

## Scope

- Linkhash auth endpoints/UI/session flow.
- Registry metadata, publish/install endpoints, artifact routing.

## Tasks

1. [x] Identity + namespace
1. [x] map Bluesky identity to `@handle` namespace
1. [x] support package coordinates `@handle/package`
1. [x] reserved namespace enforcement on org/package create

2. [x] Canonical package identity + routing
1. [x] canonical id formula (`lh_<sha256(...)>`) implemented
1. [x] alias route support (`/@handle/package[@version]`)
1. [x] canonical route support (`/lh_<sha256...>`)
1. [x] always return canonical id in API responses

3. [ ] Metadata + artifact storage behavior
1. [x] metadata/auth state fully in Postgres
1. [x] artifact bytes via pluggable backend (`local` now, `r2` later)
1. [x] install/download endpoints resolve artifact via backend by key
1. [ ] parity tests for backend switch without API contract changes

4. [ ] Auth/session/token product flow
1. [ ] Bluesky OAuth callback final wiring + identity fetch path
1. [ ] session issuance/rotation flow validation in app routes
1. [ ] PAT create/list/revoke UX + API docs
1. [ ] permission checks per role/scope for private/public packages

5. [ ] CLI integration E2E against linkhash
1. [ ] `deka publish` succeeds with PAT, fails with clear scope errors
1. [ ] `deka install` can install published PHPX package from linkhash
1. [ ] docs for local dev config and token setup

## Acceptance

1. [ ] User can login and publish a package in local dev.
1. [ ] Another project can install package using `deka install`.
1. [ ] Private package access is denied without proper role/scope.
