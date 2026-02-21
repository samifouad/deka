---
category: "phpx-js-target-semantics"
categoryLabel: "Overview"
categoryOrder: 0
version: "latest"
---
# PHPX JavaScript Target Semantics

Status: Locked

When compiling `.phpx` to JavaScript (`deka build`), runtime behavior follows **JavaScript semantics**.

PHPX still enforces its language-specific guarantees at compile time:
- strict type checking
- module/import/export validation
- JSX/component validation
- PHPX-only language rules (forbidden constructs, annotation checks, etc.)

## Execution Model

1. Parse + validate PHPX source.
2. Typecheck and apply PHPX rules.
3. Emit JavaScript.
4. Execute emitted JavaScript with normal JS engine behavior.
5. Emit `importmap.json` alongside output JS to make browser module resolution deterministic for short paths/bare specifiers.

This means emitted JS is the runtime source of truth, not PHP compatibility emulation.

## What stays PHPX-specific

- Type declarations (`interface`, `type`, struct/type rules) remain compile-time constraints.
- Import/export restrictions and symbol validation remain compile-time constraints.
- Disallowed PHPX constructs remain errors during compile/validation.

## What follows JS at runtime

- truthiness and boolean coercion
- object/array behavior in emitted code
- exception behavior in emitted code
- async/await scheduling and promise behavior
- string/number coercion in lowered expressions
- emitted `==`/`!=` comparisons are lowered to strict JS operators (`===`/`!==`)
- `print(...)` lowering is side-effect only (`console.log`) with `undefined` expression value

## Import Map Output

`deka build` writes an `importmap.json` in the same directory as the emitted JS module.

Default prefix mappings:
- `@/` -> `/`
- `component/` -> `/php_modules/component/`
- `deka/` -> `/php_modules/deka/`
- `encoding/` -> `/php_modules/encoding/`
- `db/` -> `/php_modules/db/`

Additional fallback mappings are generated for any other bare import specifiers found in frontmatter imports. Relative/absolute/URL imports are not added.

## Design Rule

For JS target features:
- prefer direct lowering to idiomatic JS
- avoid PHP runtime behavior emulation shims
- add compatibility helpers only when explicitly required and documented

## Example

Source (`.phpx`):

```phpx
---
interface User {
  $id: int
  $name: string
  $isAdmin: bool
}

function greet($user: User): string {
  if ($user.isAdmin) {
    return "Welcome, " + $user.name
  }
  return "Hi, " + $user.name
}

$ok: User = { id: 1, name: "Sami", isAdmin: true }
---
<div>{greet($ok)}</div>
```

Conceptual JS output:

```js
function greet(user) {
  if (user.isAdmin) return "Welcome, " + user.name;
  return "Hi, " + user.name;
}
const ok = { id: 1, name: "Sami", isAdmin: true };
```

If the source violates `User` shape/types, build fails before emit.

## Introspection Integration

When a module imports `deka/i`, the JS emitter injects a local runtime helper and pre-populates `__phpxTypeRegistry` from top-level PHPX `struct` declarations.

- `struct` fields are lowered to runtime schemas (string/number/boolean/object/array/optional/union where available).
- `i.get("MyStruct")`, `i.parse("MyStruct", value)`, and `i.safeParse(...)` read from that registry.
- JSX lowering is unchanged; components remain normal functions. You opt into runtime checks by calling `i.safeParse` inside component logic (for example to validate props at boundary points).

## Non-goal

Do not make JS target behave like PHP runtime by default.

If a compatibility behavior is needed, it must be:
1. explicit
2. documented
3. opt-in where possible

## Related

- `docs/phpx/phpx-dx.md`
- `docs/phpx/reboot-platform-architecture.md`
