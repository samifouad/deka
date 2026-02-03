# PHPX Task Tracker

This file tracks the live phase/task status for the PHPX upgrade work.

## Phase Status (Current)
- Phase 0: Baseline ✅
- Phase 1: Parser/Lexer (phpx mode) ✅
- Phase 2: Runtime Object Value ✅
- Phase 3: Struct Value Semantics ✅
- Phase 4: Strict Typing (phpx) ✅
- Phase 4.5: Type System Enhancements (phpx) ✅
- Phase 4.75: Rust-Style Struct Syntax (phpx) ✅
- Phase 5: Module Isolation (phpx) ✅
- Phase 6: PHP <-> PHPX Bridge ✅
- Phase 7: Tests + Docs ⏳
- Phase 8: JSX + Component Core ✅
- Phase 9: component/dom (Replace Mode Default) ✅

## Active TODOs

### Phase 6: PHP <-> PHPX Bridge (hardening)
- [x] Fix namespaced struct coercion (type metadata should use fully-qualified struct names).
- [x] Resolve type aliases for applied types (e.g., `type Maybe<T> = Option<T>`).
- [x] Define `Result` array schema rules (`ok` boolean vs truthy) and align coercion.
- [x] Decide/document missing struct fields behavior (defaults vs unset).

### Phase 7: Tests + Docs
- Add PHP<->PHPX bridge tests:
  - `Option<T>`: null -> None, Some -> value, return mapping.
  - `Result<T,E>`: Ok/Err return mapping; array/stdClass coercions.
  - Object/object-shape + struct param coercion (extra keys ignored).
- Add runtime coverage for module isolation + unused import errors.
- Add tests/docs for frontmatter templates + `<Hydration />`.
- Update `docs/VALIDATION-STATUS.md` to reflect current typechecker + PHPX rule coverage.
- Update `docs/PHPX-VALIDATION.md` checklists to mark implemented rules (structs, enums, module rules).
- Deduplicate boundary coercion bullets in `docs/phpx-dx.md`. ✅

### Phase 4.x: Type System Hardening
- [x] Add match expression inference (union of arm types) so `match` participates in return/assignment typing.
- [x] Infer type params for `array<T>` from array literals / `Type::Array` actuals.
- [x] Infer type params for `Option<T>` / `Result<T,E>` when actuals are enum cases
  (`Option::Some`, `Result::Ok`, `Result::Err`).
- [x] Add method-call type checking for structs/interfaces (arity + arg types).
- [x] Add dot-access typing for promoted embedded fields in inference (if any gap remains).

### Phase 2–3: Object/Struct Runtime (hardening)
- [x] Add `unset($obj.field)` support for dot access (ObjectMap + struct).
- [x] Support `->` property access on ObjectMap for PHP compatibility
  (FetchProp/AssignProp/UnsetObj/IssetProp/Dynamic).
- [x] Decide how ObjectMap crosses PHP boundary: keep ObjectMap or coerce to stdClass.
- [x] Audit core object helpers (`get_class`, `property_exists`, `method_exists`, `count`)
  for ObjectMap/Struct semantics and document/implement decisions.
- [x] Define object-literal equality semantics (`==`/`===`) and implement deep comparison.
- [x] Add tests for dot-unset + object-literal equality (value semantics).

### Phase 8: JSX + Component Core (hardening)
- [ ] Implement JSX validation pass (syntax/expressions/components) as outlined in
  `docs/PHPX-VALIDATION.md`.
- [x] Enforce JSX expression rules (no statements; object literals require `{{ }}`).
- [ ] Add optional JSX/VNode type inference (e.g., `VNode` return type for components).
- [x] Decide on JSX whitespace normalization (current renderer trims text nodes).

### Phase 5: Module Isolation (hardening)
- [x] Verify unused-import detection in presence of synthetic JSX/runtime imports
  (avoid false positives/negatives).
- [x] Decide whether `import` in `.php` should allow additional PHP statements before it
  when `<?php` is present (currently must be first non-empty line).
- [x] Add explicit tests for duplicate imports, duplicate export aliases, and cyclic imports.
- [x] Clarify behavior of `phpx_import()` when module load fails (panic/trigger_error/echo).

### Phase 4.75: Struct Syntax (hardening)
- [ ] Allow unary +/− constant expressions in struct defaults (e.g. `$x: int = -1`).
- [ ] Decide whether object/struct literals should be permitted as struct defaults;
  if yes, extend constant-expr validation + runtime init.

### Phase 9: component/dom (hardening)
- [ ] ContextProvider should push/pop context even when JSX passes callable (not string).
- [ ] Decide on `createRoot` `mode` support (implement or remove + document).
- [ ] Implement or remove `Link` prefetch option (currently unused in hydration).
- [ ] Add helper to emit partial JSON responses with proper headers
  (or document required headers in examples).
- [ ] Clarify layout semantics (where layout id is set and when partial navigation falls back).

## Notes (Bridge Behavior)
- Boundary coercions are lenient for legacy PHP:
  - `null` -> `Option::None` for `Option<T>` params.
  - Arrays/stdClass -> Object/object-shape/struct (extra keys ignored).
  - `Option<T>` return -> `null` or inner value.
  - `Result<T,E>` return -> `T` or `['ok' => false, 'error' => ...]`.

## References
- Full plan/status: `docs/phpx-upgrade-plan.md`
- DX + syntax summary: `docs/phpx-dx.md`
