# PHPX Conformance Checklist

This checklist maps PHPX language/runtime features to concrete fixtures in `tests/phpx/`.
Each feature should have at least one fixture file and corresponding `.out` expectations.

## Core Syntax + Semantics
- [x] Variables, literals, and expressions
- [x] Arithmetic + boolean operators
- [x] String interpolation + escapes
- [x] Arrays + destructuring
- [x] Object literals + dot access
- [x] Functions + default args
- [x] Arrow functions
- [x] Closures + use bindings
- [x] Control flow: if/else, match, switch
- [x] Loops: for/foreach/while
- [x] Error handling semantics (no exceptions)

## PHPX Type System
- [x] Type annotations (scalars + unions)
- [x] Struct definitions
- [x] Struct value semantics
- [x] Struct composition (`use`)
- [x] Enums (payload + unit)
- [x] Option / Result core behavior

## Modules + Imports
- [x] `import { x } from 'module'` (PHPX entry)
- [x] Module cycles
- [x] Missing export error
- [x] Unused import error
- [x] Frontmatter module export

## JSX + Components
- [x] JSX element creation
- [x] JSX props + children
- [x] JSX context provider + hook
- [x] renderToString (DOM)
- [x] Hydration component output
- [x] Island directive metadata (`clientLoad/Idle/Visible/Media/Only`)

## Runtime Bridge
- [x] PHP -> PHPX Object/Struct bridging
- [x] PHP -> PHPX Option bridging
- [x] PHP -> PHPX Result bridging

## Operators + New Syntax
- [x] Pipeline operator `|>`
- [x] Placeholder `...` in calls

## Coverage Map (Implemented)

| Feature | Fixture | Expectations |
| --- | --- | --- |
| Object literals + dot access | `tests/phpx/objects/object_literals.phpx` | `.out` |
| Literals basics | `tests/phpx/objects/literals_basic.phpx` | `.out` |
| Arithmetic ops | `tests/phpx/objects/arithmetic_ops.phpx` | `.out` |
| Boolean ops | `tests/phpx/objects/boolean_ops.phpx` | `.out` |
| Ternary + coalesce | `tests/phpx/objects/ternary_coalesce.phpx` | `.out` |
| Array access | `tests/phpx/objects/array_access.phpx` | `.out` |
| Foreach key/value | `tests/phpx/objects/foreach_kv.phpx` | `.out` |
| String edge cases | `tests/phpx/objects/string_edge_cases.phpx` | `.out` |
| Array append + unset | `tests/phpx/objects/array_append_unset.phpx` | `.out` |
| Switch fallthrough | `tests/phpx/objects/switch_fallthrough.phpx` | `.out` |
| Match default | `tests/phpx/objects/match_default.phpx` | `.out` |
| Increment/decrement | `tests/phpx/objects/inc_dec.phpx` | `.out` |
| isset + empty | `tests/phpx/objects/isset_empty.phpx` | `.out` |
| Enums + match | `tests/phpx/enums/match.phpx` | `.out` |
| Arrow functions | `tests/phpx/functions/arrow_function.phpx` | `.out` |
| Struct value semantics | `tests/phpx/structs/value_semantics.phpx` | `.out` |
| Struct composition | `tests/phpx/structs/embedded_use.phpx` | `.out` |
| Option + Result | `tests/phpx/options/option_result.phpx` | `.out` |
| Pipeline operator | `tests/phpx/pipeline/basic.phpx` | `.out` |
| JSX element shape | `tests/phpx/jsx/basic.phpx` | `.out` |
| JSX context | `tests/phpx/jsx/context.phpx` | `.out` |
| JSX renderToString | `tests/phpx/jsx/render.phpx` | `.out` |
| Modules import/export | `tests/phpx/modules/import_export.phpx` | `.out` |
| Modules cyclic import | `tests/phpx/modules/cyclic_import.phpx` | `.out` |
| Missing export error | `tests/phpx/modules/missing_export.phpx` | `.err` |
| Unused import error | `tests/phpx/modules/unused_import.phpx` | `.err` |
| Frontmatter entry | `tests/phpx/modules/frontmatter_entry.phpx` | `.out` |
| Frontmatter module | `tests/phpx/modules/frontmatter_module.phpx` | `.out` |
| Hydration component | `tests/phpx/modules/hydration_component.phpx` | `.out` |
| Island directives | `tests/phpx/modules/island_directives.phpx` | `.out` |
| PHP bridge object/struct | `tests/phpx/bridge/object_struct_from_php.php` | `.out` |
| PHP bridge Option | `tests/phpx/bridge/option_from_php.php` | `.out` |
| PHP bridge Result | `tests/phpx/bridge/result_from_php.php` | `.out` |
| Control flow + loops | `tests/phpx/objects/arrays_control_flow.phpx` | `.out` |
| String interpolation | `tests/phpx/objects/string_interpolation.phpx` | `.out` |
| Type annotations | `tests/phpx/structs/type_annotations.phpx` | `.out` |
| Closures + use bindings | `tests/phpx/structs/closure_use.phpx` | `.out` |
| Panic semantics | `tests/phpx/options/error_panic.phpx` | `.err` + `.code` |
| Array destructuring | `tests/phpx/objects/array_destructuring.phpx` | `.out` |
| Struct definition | `tests/phpx/structs/definition_only.phpx` | `.out` |
| Function defaults | `tests/phpx/functions/default_args.phpx` | `.out` |
| Return types | `tests/phpx/functions/return_types.phpx` | `.out` |

## Next Additions (To Do)
- [x] Arrays + destructuring fixture
- [x] Struct definition-only fixture
