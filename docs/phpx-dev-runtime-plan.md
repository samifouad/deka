# PHPX Dev Runtime Plan

## Goals

- Keep `deka serve` production-only.
- Add `deka serve --dev` as the canonical development runtime mode.
- Support HMR for all rendered HTML, not only islands.
- Keep islands as hydration scheduling primitives, not HMR boundaries.
- Provide Tailwind-compatible utility CSS generation in runtime (no required Node build step).

## Command Model

- `deka serve <entry>`: production behavior only.
- `deka serve --dev <entry>`: watch + HMR WS + dev client injection.
- `deka run dev`: script alias flow from `deka.json` (expected usage: `deka serve --dev <entry>`).

## Architecture

### 1) Dev Server Core

- [ ] Add `--dev` flag to `serve`.
- [ ] Enable watch mode automatically when `--dev` is set.
- [ ] Start HMR websocket endpoint in dev mode.
- [ ] Inject dev client bootstrap into HTML responses in dev mode only.
- [ ] Keep `serve` output stable for production mode.

### 2) HTML-Wide HMR (Not Islands-Only)

- [ ] Assign stable deterministic node ids (`deka*`) for rendered output.
- [ ] Keep ids stable across renders when structure is unchanged.
- [ ] Compute server-side DOM diffs on file changes.
- [ ] Send granular patch operations over HMR WS.
- [ ] Apply node/subtree patches on client without full route replacement.
- [ ] Preserve focus, scroll, and form state where possible.
- [ ] Escalate to subtree replace, then hard reload only as last resort.

### 3) Islands and Hydration Scheduling

- [ ] Support directives: `client:load`, `client:idle`, `client:visible`, `client:media`, `client:only`.
- [ ] Keep island boundaries explicit and metadata-driven.
- [ ] Let non-island HTML still receive HMR patches.
- [ ] Maintain client scheduling behavior independent from HMR patch transport.

### 4) Utility CSS Runtime (Tailwind-Compatible Surface)

- [ ] Introduce runtime-native utility CSS engine crate.
- [ ] Start with core utility class compatibility and variant handling.
- [ ] Add optional preflight/base reset toggle.
- [ ] Generate deduped CSS during SSR and cache in project `.cache`.
- [ ] Document compatibility matrix and unsupported classes.

### 5) Tooling and LSP

- [x] Ensure import/export diagnostics use plain text in editor diagnostics.
- [x] Ensure import completion includes module exports reliably.
- [x] Add module path + export completion parity tests for editor integration.
- [ ] Document VS Code + Zed dev-mode workflow and troubleshooting.

## Phased Rollout

### Phase A: Dev Mode Scaffolding

- [x] `serve --dev` flag and runtime dev-mode plumbing.
- [x] File-watch event stream emits HMR change notifications (initial logging path).
- [x] Dev-mode feature gate for future WS and patch application.
- [x] `deka run <script>` resolves `deka.json` scripts for dev entrypoints (for `deka run dev` flow).

### Phase B: HMR Transport

- [x] WS endpoint (`/_deka/hmr`) and event protocol.
- [x] Browser dev client bootstrap and reconnect logic.
- [x] Basic reload fallback when patching is unavailable.

### Phase C: Granular DOM Patches

- [x] Stable DOM id assignment.
- [ ] Server diff generation and patch payload schema.
- [x] Client patch applier with state-preserving heuristics.

### Phase D: Islands Scheduling

- [ ] Directive parser + metadata emission.
- [ ] Client scheduler (`load/idle/visible/media`).
- [ ] Boundary-local remount fallback.

### Phase E: Utility CSS Engine

- [ ] Utility parser + resolver + emitter.
- [ ] Config file support (`deka.css.json` or equivalent).
- [ ] SSR injection + cache integration.

### Phase F: Hardening

- [ ] End-to-end tests for HMR state preservation.
- [ ] Performance baseline for patch path vs full reload.
- [ ] Documentation and migration notes.

## Acceptance Criteria

- [ ] `deka serve` never enables watcher/HMR/dev client behavior.
- [ ] `deka serve --dev` enables watch + HMR transport.
- [ ] Editing a template updates changed DOM nodes without full route replacement.
- [ ] Islands hydrate by directive schedule while non-island DOM still supports HMR.
- [ ] Tailwind-style utility classes can be used without separate build tooling.
