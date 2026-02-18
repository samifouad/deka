# Linkhash MVP Execution Tracker

Status date: 2026-02-18
Owner: codex

## Goal
Ship four MVP blocks for Linkhash with commit-per-item execution.

## Item 1: Doccomments + Versioned Docs Pipeline
- [x] Parse PHPX doccomments/signatures/examples at publish time.
- [x] Persist docs snapshot by package+version.
- [x] Add docs API endpoints for package/version/symbol.
- [x] Render docs in Linkhash package UI (version switch aware).
- [x] Add tests and docs.
- [x] Commit: `feat(linkhash): add versioned doccomments docs pipeline`

## Item 2: GitHub-Style File Explorer
- [ ] Add tree/blob APIs (ref/version aware).
- [ ] Build PHPX repo browser UI with file tree + code viewer + README default.
- [ ] Add package/repo page links into explorer.
- [ ] Add tests and docs.
- [ ] Commit: `feat(linkhash): add github-style file explorer`

## Item 3: Issues + PRs (MVP)
- [ ] Add schema and APIs for issues/comments/state.
- [ ] Add schema and APIs for PR basics (source/target refs, state, comments).
- [ ] Build minimal PHPX UI for list/detail/create flows.
- [ ] Add tests and docs.
- [ ] Commit: `feat(linkhash): add mvp issues and pull requests`

## Item 4: Adwa Integration
- [ ] Add preview URL resolver (`/@user/repo/ref`) -> resolved commit.
- [ ] Add Preview button in primary UI and Adwa launch flow.
- [ ] Add Fork flow (copy repo to user account + repo naming).
- [ ] Add post-fork instructions + one-click open in Adwa editor.
- [ ] Add audit events, tests, and docs.
- [ ] Commit: `feat(linkhash): add adwa preview and fork integration`

## Global Done Criteria
- [ ] All 4 items completed.
- [ ] Commit per item landed.
- [ ] APIs and UI are version-aware where required.
- [ ] Error messages are actionable for AI-agent workflows.
