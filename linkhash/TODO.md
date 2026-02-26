# Linkhash TODO (Post-MVP Hardening)

## Auth and Security Hardening
- Replace bootstrap-token-first setup with explicit one-time bootstrap flow.
- Enforce bootstrap token rotation/removal after initial admin setup.
- Add token scopes and enforce them in middleware:
  - `repo:read`
  - `repo:write`
  - `package:publish`
  - `admin:*`
- Add token lifecycle endpoints:
  - create
  - list
  - revoke
  - rotate
- Add audit logging for auth-sensitive actions:
  - repo create/push/delete
  - package publish
  - SSH key add/remove
  - token create/revoke
- Add org/team-aware ACL model beyond `owner == auth.username`.
- Add IP-aware rate limiting for auth and publish endpoints.
- Add structured security events/metrics export.
- Add integration tests for authz failures and privilege boundaries.

## Optional Future Enhancements
- Add SSH Git transport alongside Smart HTTP.
- Add signed package metadata and verification at install time.
- Add immutable release policy and yanked-release semantics.
