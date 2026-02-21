# Deka Task: Linkhash ORM Migration + Integrity

Status date: 2026-02-21
Owner: codex

## Goal
Switch Linkhash PHPX registry/auth/query paths to the ORM client, align schema, and complete E2E integrity validation with linkhash-git.

## Tasks
- [x] Task 1: Align ORM models + migrations with `lh_*` schema (including integrity columns).
- [ ] Task 2: Replace raw SQL in `phpx/main.phpx` + `phpx/db/Database.phpx` with ORM client.
- [ ] Task 3: Update docs for ORM + integrity flow.
- [ ] Task 4: Run migrations + tests, then E2E publish/install against linkhash-git.

## Notes
- Commit per task.
- Use Postgres via Docker.
- `linkhash-git` must be running during E2E.
