# Unified Pending Tasks (PHPX)

This file is the single, merged list of all *pending* work items across the
current PHPX effort. It is a union of the outstanding tasks from:

- `docs/TASKS.md`
- `docs/VALIDATION-STATUS.md`
- `docs/PHPX-VALIDATION.md`
- `docs/DX-TASKS.md`

Notes:
- Items are grouped by area, but all pending tasks are included.
- Where tasks were listed without checkboxes in the source docs (especially
  `docs/VALIDATION-STATUS.md`), they are listed explicitly here.
- Duplicates are kept only when they add distinct detail.

## Process (applies to every task)
- Always run relevant tests after completing a task before moving to the next.
- Always make a git commit after tests pass and before starting the next task.

## Local build/test loop notes
- Rebuild the CLI in release mode: `cargo build --release -p cli`
- The locally wired `deka` CLI points to the release binary: `target/release/cli`
  (use this for testing after rebuilding).

## Runtime + Language Hardening (from `docs/TASKS.md`)
- [ ] Fix namespaced struct coercion (type metadata should use fully-qualified struct names).
- [ ] Resolve type aliases for applied types (e.g., `type Maybe<T> = Option<T>`).
- [ ] Define `Result` array schema rules (`ok` boolean vs truthy) and align coercion.
- [ ] Decide/document missing struct fields behavior (defaults vs unset).
- [ ] Add match expression inference (union of arm types) so `match` participates in return/assignment typing.
- [ ] Infer type params for `array<T>` from array literals / `Type::Array` actuals.
- [ ] Infer type params for `Option<T>` / `Result<T,E>` when actuals are enum cases (`Option::Some`, `Result::Ok`, `Result::Err`).
- [ ] Add method-call type checking for structs/interfaces (arity + arg types).
- [ ] Add dot-access typing for promoted embedded fields in inference (if any gap remains).
- [ ] Add `unset($obj.field)` support for dot access (ObjectMap + struct).
- [ ] Support `->` property access on ObjectMap for PHP compatibility
- [ ] Decide how ObjectMap crosses PHP boundary: keep ObjectMap or coerce to stdClass.
- [ ] Audit core object helpers (`get_class`, `property_exists`, `method_exists`, `count`) for ObjectMap/Struct semantics and document/implement decisions.
- [ ] Define object-literal equality semantics (`==`/`===`) and implement deep comparison.
- [ ] Add tests for dot-unset + object-literal equality (value semantics).
- [ ] Implement JSX validation pass (syntax/expressions/components) as outlined in `docs/PHPX-VALIDATION.md`.
- [ ] Enforce JSX expression rules (no statements; object literals require `{{ }}`).
- [ ] Add optional JSX/VNode type inference (e.g., `VNode` return type for components).
- [ ] Decide on JSX whitespace normalization (current renderer trims text nodes).
- [ ] Verify unused-import detection in presence of synthetic JSX/runtime imports (avoid false positives/negatives).
- [ ] Decide whether `import` in `.php` should allow additional PHP statements before it when `<?php` is present (currently must be first non-empty line).
- [ ] Add explicit tests for duplicate imports, duplicate export aliases, and cyclic imports.
- [ ] Clarify behavior of `phpx_import()` when module load fails (panic/trigger_error/echo).
- [ ] Allow unary +/− constant expressions in struct defaults (e.g. `$x: int = -1`).
- [ ] Decide whether object/struct literals should be permitted as struct defaults; if yes, extend constant-expr validation + runtime init.
- [ ] ContextProvider should push/pop context even when JSX passes callable (not string).
- [ ] Decide on `createRoot` `mode` support (implement or remove + document).
- [ ] Implement or remove `Link` prefetch option (currently unused in hydration).
- [ ] Add helper to emit partial JSON responses with proper headers (or document required headers in examples).
- [ ] Clarify layout semantics (where layout id is set and when partial navigation falls back).

### Phase 7 Tests/Docs (from `docs/TASKS.md`, non-checkbox items)
- [ ] Add PHP<->PHPX bridge tests for `Option<T>` (null -> None, Some -> value, return mapping).
- [ ] Add PHP<->PHPX bridge tests for `Result<T,E>` (Ok/Err return mapping; array/stdClass coercions).
- [ ] Add PHP<->PHPX bridge tests for object/object-shape + struct param coercion (extra keys ignored).
- [ ] Add runtime coverage for module isolation + unused import errors.
- [ ] Add tests/docs for frontmatter templates + `<Hydration />`.

## Validation System (from `docs/VALIDATION-STATUS.md` + `docs/PHPX-VALIDATION.md`)

### Validation status gaps (explicit in `docs/VALIDATION-STATUS.md`)
- [ ] Add `deka-validation` integration for parser errors (use formatter instead of basic text).
- [ ] Extend `ParseError` to include `error_kind` and `help_text`.
- [ ] Add PHPX error kinds (Syntax/Type/Import/Export/Null/OOP/JSX/etc.) for validation output.
- [ ] Add compiler API: `compile_phpx(source, file_path) -> ValidationResult` with structured errors.
- [ ] Return `ValidationResult { errors, warnings, ast }` instead of generic CoreError.

### Validation plan checklist (from `docs/PHPX-VALIDATION.md`)
- [ ] PHPX-VALIDATION:  Unclosed braces, brackets, parentheses
- [ ] PHPX-VALIDATION:  Invalid tokens
- [ ] PHPX-VALIDATION:  Unexpected end of file
- [ ] PHPX-VALIDATION:  Malformed expressions
- [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/syntax.rs`
- [ ] PHPX-VALIDATION:  Implement `validate_syntax(source: &str) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Use php-rs parser error recovery
- [ ] PHPX-VALIDATION:  Map parser errors to validation errors
- [ ] PHPX-VALIDATION:  Add helpful suggestions for common mistakes
- [ ] PHPX-VALIDATION:  Import at top of file (before other code)
- [ ] PHPX-VALIDATION:  Valid import syntax
- [ ] PHPX-VALIDATION:  Named imports: `import { foo, bar } from 'module'`
- [ ] PHPX-VALIDATION:  WASM imports: `import { fn } from '@user/mod' as wasm`
- [ ] PHPX-VALIDATION:  Module path format (no relative paths with `../`)
- [ ] PHPX-VALIDATION:  Unused imports (warning)
- [ ] PHPX-VALIDATION:  Duplicate imports
- [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/imports.rs`
- [ ] PHPX-VALIDATION:  Implement `validate_imports(ast: &Ast) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Check import placement (AST position)
- [ ] PHPX-VALIDATION:  Validate module paths
- [ ] PHPX-VALIDATION:  Track used imports (mark on usage)
- [ ] PHPX-VALIDATION:  Detect duplicates
- [ ] PHPX-VALIDATION:  Export only functions, constants, types
- [ ] PHPX-VALIDATION:  No duplicate exports
- [ ] PHPX-VALIDATION:  Exported names actually exist
- [ ] PHPX-VALIDATION:  Re-export syntax validation
- [ ] PHPX-VALIDATION:  Template files: no explicit exports (auto-exported as Component)
- [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/exports.rs`
- [ ] PHPX-VALIDATION:  Implement `validate_exports(ast: &Ast) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Track exported names
- [ ] PHPX-VALIDATION:  Check for duplicates
- [ ] PHPX-VALIDATION:  Verify definitions exist
- [ ] PHPX-VALIDATION:  Special handling for template files
- [ ] PHPX-VALIDATION:  Valid type names
- [ ] PHPX-VALIDATION:  Generic syntax: `Option<T>`, `Result<T, E>`, `array<T>`
- [ ] PHPX-VALIDATION:  Object shape syntax: `Object<{ field: Type }>`
- [ ] PHPX-VALIDATION:  Type alias syntax: `type Name = ...`
- [ ] PHPX-VALIDATION:  Union types (limited): `int|float`
- [ ] PHPX-VALIDATION:  No nullable types (`?T`, `T|null` are banned)
- [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/type_syntax.rs`
- [ ] PHPX-VALIDATION:  Implement `validate_type_annotations(ast: &Ast) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Reject `null`, `?T`, `T|null` syntax
- [ ] PHPX-VALIDATION:  Validate generic parameter syntax
- [ ] PHPX-VALIDATION:  Check object shape syntax
- [ ] PHPX-VALIDATION:  Variable assignment type matches
- [ ] PHPX-VALIDATION:  Function parameter types match arguments
- [ ] PHPX-VALIDATION:  Return type matches returned value
- [ ] PHPX-VALIDATION:  Binary operation types compatible
- [ ] PHPX-VALIDATION:  Struct field types match literal values
- [ ] PHPX-VALIDATION:  Safe widening only (int → float allowed, not reverse)
- [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/type_checker.rs`
- [ ] PHPX-VALIDATION:  Implement type inference engine
- [ ] PHPX-VALIDATION:  Implement `check_types(ast: &Ast) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Build type environment (symbol table)
- [ ] PHPX-VALIDATION:  Infer types for expressions
- [ ] PHPX-VALIDATION:  Check compatibility at assignments/calls/returns
- [ ] PHPX-VALIDATION:  Track widening rules
- [ ] PHPX-VALIDATION:  Generic parameters are used
- [ ] PHPX-VALIDATION:  Generic constraints are satisfied
- [ ] PHPX-VALIDATION:  Type arguments provided where required
- [ ] PHPX-VALIDATION:  Constraint syntax: `T: Reader`
- [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/generics.rs`
- [ ] PHPX-VALIDATION:  Implement `validate_generics(ast: &Ast) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Track generic parameters
- [ ] PHPX-VALIDATION:  Check constraints
- [ ] PHPX-VALIDATION:  Infer type arguments
- [ ] PHPX-VALIDATION:  No `null` literals
- [ ] PHPX-VALIDATION:  No `=== null` or `!== null` comparisons
- [ ] PHPX-VALIDATION:  No `is_null()` calls
- [ ] PHPX-VALIDATION:  Suggest `Option<T>` instead
- [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/phpx_rules.rs`
- [ ] PHPX-VALIDATION:  Implement `validate_no_null(ast: &Ast) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Scan AST for null literals
- [ ] PHPX-VALIDATION:  Scan for null comparisons
- [ ] PHPX-VALIDATION:  Scan for is_null() calls
- [ ] PHPX-VALIDATION:  No `throw` statements
- [ ] PHPX-VALIDATION:  No `try/catch/finally` blocks
- [ ] PHPX-VALIDATION:  Suggest `Result<T, E>` instead
- [ ] PHPX-VALIDATION:  Allow `panic()` for unrecoverable errors
- [ ] PHPX-VALIDATION:  Add to `crates/modules_php/src/validation/phpx_rules.rs`
- [ ] PHPX-VALIDATION:  Implement `validate_no_exceptions(ast: &Ast) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Scan for throw statements
- [ ] PHPX-VALIDATION:  Scan for try/catch/finally
- [ ] PHPX-VALIDATION:  No `class` declarations
- [ ] PHPX-VALIDATION:  No `trait` declarations
- [ ] PHPX-VALIDATION:  No `extends` keyword
- [ ] PHPX-VALIDATION:  No `implements` keyword
- [ ] PHPX-VALIDATION:  No `new` keyword
- [ ] PHPX-VALIDATION:  No `interface` inheritance (structural interfaces only)
- [ ] PHPX-VALIDATION:  Suggest structs instead
- [ ] PHPX-VALIDATION:  Add to `crates/modules_php/src/validation/phpx_rules.rs`
- [ ] PHPX-VALIDATION:  Implement `validate_no_oop(ast: &Ast) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Scan for class/trait/interface declarations
- [ ] PHPX-VALIDATION:  Scan for extends/implements
- [ ] PHPX-VALIDATION:  Scan for new keyword
- [ ] PHPX-VALIDATION:  No `namespace` declarations
- [ ] PHPX-VALIDATION:  No top-level `use` statements
- [ ] PHPX-VALIDATION:  Suggest import/export instead
- [ ] PHPX-VALIDATION:  Add to `crates/modules_php/src/validation/phpx_rules.rs`
- [ ] PHPX-VALIDATION:  Implement `validate_no_namespace(ast: &Ast) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Scan for namespace declarations
- [ ] PHPX-VALIDATION:  Scan for top-level use statements
- [ ] PHPX-VALIDATION:  No `__construct` in PHPX structs
- [ ] PHPX-VALIDATION:  Field defaults are constant expressions
- [ ] PHPX-VALIDATION:  Field type annotations are valid
- [ ] PHPX-VALIDATION:  No duplicate field names
- [ ] PHPX-VALIDATION:  Struct composition (`use A`) is valid
- [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/structs.rs`
- [ ] PHPX-VALIDATION:  Implement `validate_struct_definitions(ast: &Ast) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Check for __construct
- [ ] PHPX-VALIDATION:  Validate field defaults are constants
- [ ] PHPX-VALIDATION:  Check for duplicate fields
- [ ] PHPX-VALIDATION:  Validate composition
- [ ] PHPX-VALIDATION:  All required fields provided
- [ ] PHPX-VALIDATION:  No extra fields
- [ ] PHPX-VALIDATION:  Field types match values
- [ ] PHPX-VALIDATION:  Shorthand syntax valid
- [ ] PHPX-VALIDATION:  Nested struct literals valid
- [ ] PHPX-VALIDATION:  Add to `crates/modules_php/src/validation/structs.rs`
- [ ] PHPX-VALIDATION:  Implement `validate_struct_literals(ast: &Ast) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Check required fields
- [ ] PHPX-VALIDATION:  Reject extra fields
- [ ] PHPX-VALIDATION:  Validate field types
- [ ] PHPX-VALIDATION:  Handle shorthand syntax
- [ ] PHPX-VALIDATION:  Valid tag names
- [ ] PHPX-VALIDATION:  Matching opening/closing tags
- [ ] PHPX-VALIDATION:  Valid attribute syntax
- [ ] PHPX-VALIDATION:  Fragment syntax: `<>...</>`
- [ ] PHPX-VALIDATION:  Self-closing tags
- [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/jsx.rs`
- [ ] PHPX-VALIDATION:  Implement `validate_jsx_syntax(ast: &Ast) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Check tag matching
- [ ] PHPX-VALIDATION:  Validate attribute names
- [ ] PHPX-VALIDATION:  Check self-closing vs paired tags
- [ ] PHPX-VALIDATION:  Expression syntax: `{$var}`, `{$obj.field}`
- [ ] PHPX-VALIDATION:  If blocks: `{if ($cond) { <p>yes</p> }}`
- [ ] PHPX-VALIDATION:  Foreach loops: `{foreach ($items as $item) { <li>{$item}</li> }}`
- [ ] PHPX-VALIDATION:  Object literals require double braces: `{{ key: 'value' }}`
- [ ] PHPX-VALIDATION:  No statements in expressions (only expressions)
- [ ] PHPX-VALIDATION:  Add to `crates/modules_php/src/validation/jsx.rs`
- [ ] PHPX-VALIDATION:  Implement `validate_jsx_expressions(ast: &Ast) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Check expression syntax
- [ ] PHPX-VALIDATION:  Validate if/foreach blocks
- [ ] PHPX-VALIDATION:  Detect statements in expressions
- [ ] PHPX-VALIDATION:  Check object literal braces
- [ ] PHPX-VALIDATION:  Component names are capitalized (or imported)
- [ ] PHPX-VALIDATION:  Built-in tags are lowercase
- [ ] PHPX-VALIDATION:  Component props match definition (if available)
- [ ] PHPX-VALIDATION:  Special components: `<Link>`, `<Hydration>`, `<ContextProvider>`
- [ ] PHPX-VALIDATION:  Add to `crates/modules_php/src/validation/jsx.rs`
- [ ] PHPX-VALIDATION:  Implement `validate_components(ast: &Ast) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Check component naming
- [ ] PHPX-VALIDATION:  Validate props (if type info available)
- [ ] PHPX-VALIDATION:  Track imported components
- [ ] PHPX-VALIDATION:  Frontmatter starts at beginning of file
- [ ] PHPX-VALIDATION:  Proper `---` delimiters
- [ ] PHPX-VALIDATION:  No explicit exports in template files (under `php_modules/`)
- [ ] PHPX-VALIDATION:  Template section is valid JSX
- [ ] PHPX-VALIDATION:  Imports in frontmatter only
- [ ] PHPX-VALIDATION:  Add to `crates/modules_php/src/validation/jsx.rs`
- [ ] PHPX-VALIDATION:  Implement `validate_frontmatter(ast: &Ast, file_path: &str) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Check frontmatter position
- [ ] PHPX-VALIDATION:  Validate delimiters
- [ ] PHPX-VALIDATION:  Check for exports (if in php_modules/)
- [ ] PHPX-VALIDATION:  Validate imports placement
- [ ] PHPX-VALIDATION:  Module exists in `php_modules/`
- [ ] PHPX-VALIDATION:  Module has valid entry point
- [ ] PHPX-VALIDATION:  Circular imports detected
- [ ] PHPX-VALIDATION:  Import/export names match
- [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/modules.rs`
- [ ] PHPX-VALIDATION:  Implement `validate_module_resolution(ast: &Ast, base_path: &str) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Scan php_modules/ for available modules
- [ ] PHPX-VALIDATION:  Build dependency graph
- [ ] PHPX-VALIDATION:  Detect cycles
- [ ] PHPX-VALIDATION:  Check export names
- [ ] PHPX-VALIDATION:  `@user/module` format
- [ ] PHPX-VALIDATION:  `deka.json` exists
- [ ] PHPX-VALIDATION:  `module.wasm` exists
- [ ] PHPX-VALIDATION:  `.d.phpx` stub file exists (suggest generating if missing)
- [ ] PHPX-VALIDATION:  Imported names exist in stubs
- [ ] PHPX-VALIDATION:  Add to `crates/modules_php/src/validation/modules.rs`
- [ ] PHPX-VALIDATION:  Implement `validate_wasm_imports(ast: &Ast) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Scan php_modules/@*/ for WASM modules
- [ ] PHPX-VALIDATION:  Check deka.json, module.wasm, .d.phpx
- [ ] PHPX-VALIDATION:  Parse .d.phpx for exported names
- [ ] PHPX-VALIDATION:  Suggest deka wasm commands
- [ ] PHPX-VALIDATION:  Enum match covers all cases
- [ ] PHPX-VALIDATION:  No unreachable match arms
- [ ] PHPX-VALIDATION:  Variable binding in match arms
- [ ] PHPX-VALIDATION:  Payload destructuring correct
- [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/patterns.rs`
- [ ] PHPX-VALIDATION:  Implement `validate_match_exhaustiveness(ast: &Ast) -> Vec<ValidationError>`
- [ ] PHPX-VALIDATION:  Build enum case registry
- [ ] PHPX-VALIDATION:  Check match coverage
- [ ] PHPX-VALIDATION:  Validate payload destructuring
- [ ] PHPX-VALIDATION:  Detect unreachable arms
- [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/compiler_api.rs`
- [ ] PHPX-VALIDATION:  Expose `compile_phpx(source: &str, file_path: &str) -> ValidationResult`
- [ ] PHPX-VALIDATION:  Run all validation passes in order:
- [ ] PHPX-VALIDATION:  Collect all errors and warnings
- [ ] PHPX-VALIDATION:  Format with `deka-validation`
- [ ] PHPX-VALIDATION:  Return structured result
- [ ] PHPX-VALIDATION:  Create test file for each validation rule
- [ ] PHPX-VALIDATION:  Positive tests (valid code passes)
- [ ] PHPX-VALIDATION:  Negative tests (invalid code caught)
- [ ] PHPX-VALIDATION:  Error message quality
- [ ] PHPX-VALIDATION:  Help text accuracy
- [ ] PHPX-VALIDATION:  Edge cases
- [ ] PHPX-VALIDATION:  Support for multiple errors in single file
- [ ] PHPX-VALIDATION:  Color coding by error kind
- [ ] PHPX-VALIDATION:  Code suggestions (auto-fix hints)
- [ ] PHPX-VALIDATION:  Reference links to PHPX docs
- [ ] PHPX-VALIDATION:  `format_multiple_errors()` - Format list of errors
- [ ] PHPX-VALIDATION:  Color codes by severity (error=red, warning=yellow, info=blue)
- [ ] PHPX-VALIDATION:  `format_with_suggestion()` - Include code fix suggestions
- [ ] PHPX-VALIDATION:  `format_with_docs_link()` - Add doc links

## DX / Tooling (from `docs/DX-TASKS.md`)
- [ ] DX-TASKS:  Create `tooling/tree-sitter-phpx/` directory
- [ ] DX-TASKS:  Clone `tree-sitter-php` as starting point
- [ ] DX-TASKS:  Rename project to `tree-sitter-phpx`
- [ ] DX-TASKS:  Update `package.json` metadata (name, description, repo)
- [ ] DX-TASKS:  Install tree-sitter CLI: `npm install -g tree-sitter-cli`
- [ ] DX-TASKS:  Verify build: `tree-sitter generate && tree-sitter test`
- [ ] DX-TASKS:  Add type annotation rules to `grammar.js`
48:  - [ ] Primitive types: `int`, `string`, `bool`, `float`, `mixed`
49:  - [ ] Generic types: `Option<T>`, `Result<T, E>`, `array<T>`
50:  - [ ] Object types: `Object<{ field: Type }>`
51:  - [ ] Type aliases: `type Name = ...`
- [ ] DX-TASKS:  Create `queries/highlights.scm` for type highlighting
- [ ] DX-TASKS:  Test with PHPX files containing type annotations
- [ ] DX-TASKS:  Verify types are highlighted differently from values
- [ ] DX-TASKS:  Add `import_statement` rule to `grammar.js`
66:  - [ ] Named imports: `import { foo, bar } from 'module'`
67:  - [ ] WASM imports: `import { fn } from '@user/mod' as wasm`
68:  - [ ] Default import (if needed)
- [ ] DX-TASKS:  Add `export_statement` rule
70:  - [ ] Export functions: `export function foo() {}`
71:  - [ ] Export constants: `export const X = 1`
72:  - [ ] Re-exports: `export { foo } from './bar'`
- [ ] DX-TASKS:  Add highlighting for `import`, `export`, `from`, `as` keywords
- [ ] DX-TASKS:  Test with module examples from `examples/php/modules/`
- [ ] DX-TASKS:  Add `struct_literal` rule to `grammar.js`
82:  - [ ] Type name: `Point`
83:  - [ ] Field list: `{ $x: 1, $y: 2 }`
84:  - [ ] Shorthand: `{ $x, $y }`
- [ ] DX-TASKS:  Add highlighting for struct names and fields
- [ ] DX-TASKS:  Test with struct examples
- [ ] DX-TASKS:  Verify nested struct literals work
- [ ] DX-TASKS:  Port JSX grammar from `tree-sitter-javascript`
101:  - [ ] Opening tags: `<Component>`
102:  - [ ] Self-closing tags: `<Component />`
103:  - [ ] Attributes: `<Component id={$val} />`
104:  - [ ] Children: `<div>text</div>`
105:  - [ ] Fragments: `<>...</>`
- [ ] DX-TASKS:  Add PHPX-specific JSX expressions
107:  - [ ] Variables: `{$user.name}`
108:  - [ ] If blocks: `{if ($x) { <p>yes</p> }}`
109:  - [ ] Foreach loops: `{foreach ($items as $item) { <li>{$item}</li> }}`
110:  - [ ] Object literals (double braces): `{{ host: 'localhost' }}`
- [ ] DX-TASKS:  Add highlighting for tags, attributes, expressions
- [ ] DX-TASKS:  Test with component examples from `examples/phpx-components/`
- [ ] DX-TASKS:  Add `frontmatter` rule to `grammar.js`
120:  - [ ] Detect `---` at start of file
121:  - [ ] Parse PHPX code section
122:  - [ ] Parse JSX template section
- [ ] DX-TASKS:  Add highlighting for frontmatter delimiters
- [ ] DX-TASKS:  Test with template examples
- [ ] DX-TASKS:  Verify code and template sections have correct highlighting
- [ ] DX-TASKS:  Create `extensions/phpx/` directory
- [ ] DX-TASKS:  Create `extension.toml` with PHPX language config
147:  - [ ] Set file suffixes: `["phpx"]`
148:  - [ ] Set comment syntax
149:  - [ ] Link to tree-sitter-phpx grammar
- [ ] DX-TASKS:  Add syntax highlighting theme overrides (if needed)
- [ ] DX-TASKS:  Install extension in Zed:
152:  - [ ] Symlink: `ln -s /path/to/deka/extensions/phpx ~/.config/zed/extensions/phpx`
- [ ] DX-TASKS:  Test with real PHPX files
- [ ] DX-TASKS:  Verify highlighting works for all features
- [ ] DX-TASKS:  Create `tooling/tree-sitter-phpx/README.md`
166:  - [ ] Installation instructions
167:  - [ ] Testing instructions
168:  - [ ] Editor integration guides (Zed, Neovim, Helix)
169:  - [ ] Grammar rules overview
- [ ] DX-TASKS:  Add examples to `test/corpus/`
- [ ] DX-TASKS:  Document known limitations
- [ ] DX-TASKS:  Add contributing guidelines
- [ ] DX-TASKS:  Create `crates/phpx_lsp/` directory
- [ ] DX-TASKS:  Initialize Cargo project: `cargo new phpx_lsp --bin`
- [ ] DX-TASKS:  Add dependencies to `Cargo.toml`:
188:  - [ ] `tower-lsp = "0.20"`
189:  - [ ] `tokio` (workspace)
190:  - [ ] `serde_json = "1"`
191:  - [ ] `anyhow = "1"`
192:  - [ ] `modules_php` (path dependency to existing PHPX compiler)
193:  - [ ] `deka-validation` (for error formatting)
- [ ] DX-TASKS:  Add to workspace members in root `Cargo.toml`
- [ ] DX-TASKS:  Verify build: `cargo build -p phpx_lsp`
- [ ] DX-TASKS:  Create `src/main.rs` with LSP boilerplate
- [ ] DX-TASKS:  Implement `initialize` method with server capabilities:
204:  - [ ] `textDocumentSync`: Full sync mode
205:  - [ ] `diagnosticProvider`: Report errors
206:  - [ ] (Others later: hover, completion, etc.)
- [ ] DX-TASKS:  Implement `initialized` method (log ready message)
- [ ] DX-TASKS:  Implement `shutdown` method
- [ ] DX-TASKS:  Implement `did_open` and `did_change` handlers (log only)
- [ ] DX-TASKS:  Test with manual stdio: `echo '{"jsonrpc":"2.0","method":"initialize",...}' | cargo run`
- [ ] DX-TASKS:  Create `crates/modules_php/src/compiler_api.rs`
- [ ] DX-TASKS:  Define public structs:
- [ ] DX-TASKS:  Implement `compile_phpx(source: &str, file_path: &str) -> CompilationResult`
235:  - [ ] Call existing PHPX parser/compiler
236:  - [ ] Collect syntax errors
237:  - [ ] Collect type errors
238:  - [ ] Return structured results
- [ ] DX-TASKS:  Add unit tests for error collection
- [ ] DX-TASKS:  Export from `crates/modules_php/src/lib.rs`
- [ ] DX-TASKS:  Add `deka-validation` dependency to `phpx_lsp`
- [ ] DX-TASKS:  Implement error formatting in LSP:
- [ ] DX-TASKS:  Convert formatted errors to LSP Diagnostic messages
- [ ] DX-TASKS:  Test with PHPX files containing errors
- [ ] DX-TASKS:  Verify beautiful error output in editor
- [ ] DX-TASKS:  Implement `validate_document` method in LSP server:
275:  - [ ] Call `compile_phpx` API
276:  - [ ] Convert `CompileError` to LSP `Diagnostic`
277:  - [ ] Map line/column positions
278:  - [ ] Set severity (Error vs Warning)
279:  - [ ] Include formatted message
- [ ] DX-TASKS:  Call `client.publish_diagnostics` on document open/change
- [ ] DX-TASKS:  Test with PHPX files containing:
282:  - [ ] Syntax errors
283:  - [ ] Type errors
284:  - [ ] Import errors
285:  - [ ] WIT import errors (missing stubs)
- [ ] DX-TASKS:  Verify errors appear in editor as red squiggles
- [ ] DX-TASKS:  Update `extensions/phpx/extension.toml` with LSP config:
- [ ] DX-TASKS:  Add LSP binary path to Zed settings:
- [ ] DX-TASKS:  Rebuild LSP: `cargo build --release -p phpx_lsp`
- [ ] DX-TASKS:  Restart Zed
- [ ] DX-TASKS:  Test with PHPX files
- [ ] DX-TASKS:  Verify diagnostics appear in problems panel
- [ ] DX-TASKS:  Extend compiler API to validate imports
- [ ] DX-TASKS:  Check PHPX module imports:
323:  - [ ] Verify module exists in `php_modules/`
324:  - [ ] Verify exported names exist
325:  - [ ] Detect unused imports
326:  - [ ] Detect circular imports
- [ ] DX-TASKS:  Check WASM imports:
328:  - [ ] Verify `deka.json` exists
329:  - [ ] Verify `module.wasm` exists
330:  - [ ] Check for `.d.phpx` stub file
331:  - [ ] Suggest running `deka wasm stubs` if missing
- [ ] DX-TASKS:  Add helpful error messages with fixes
- [ ] DX-TASKS:  Test with various import scenarios
- [ ] DX-TASKS:  Create `crates/phpx_lsp/README.md`
348:  - [ ] Installation instructions
349:  - [ ] Editor integration guides (Zed, VSCode, Neovim)
350:  - [ ] Configuration options
351:  - [ ] Debugging tips
- [ ] DX-TASKS:  Document compiler API in `crates/modules_php/src/compiler_api.rs`
- [ ] DX-TASKS:  Add troubleshooting section
- [ ] DX-TASKS:  List supported features and roadmap
- [ ] DX-TASKS:  Test and fix tight dot access: `$user.name.first`
- [ ] DX-TASKS:  Test and fix nested object literals in JSX: `{{ nested: { value: 1 } }}`
- [ ] DX-TASKS:  Test and fix multiline JSX expressions
- [ ] DX-TASKS:  Test and fix if/foreach blocks in JSX
- [ ] DX-TASKS:  Add error recovery rules for better partial highlighting
- [ ] DX-TASKS:  Test with malformed PHPX (ensure no crashes)
- [ ] DX-TASKS:  Create `queries/folds.scm`
- [ ] DX-TASKS:  Add folding for:
381:  - [ ] Function bodies
382:  - [ ] Struct definitions
383:  - [ ] JSX elements
384:  - [ ] If/foreach blocks
385:  - [ ] Frontmatter sections
- [ ] DX-TASKS:  Test in Zed (verify fold markers appear)
- [ ] DX-TASKS:  Create `queries/indents.scm`
- [ ] DX-TASKS:  Define indent increases for:
395:  - [ ] Function bodies
396:  - [ ] If/else/foreach blocks
397:  - [ ] JSX children
398:  - [ ] Struct/object literals
- [ ] DX-TASKS:  Test auto-indentation in Zed
- [ ] DX-TASKS:  Verify correct indent after newline
- [ ] DX-TASKS:  Create `queries/textobjects.scm`
- [ ] DX-TASKS:  Define textobjects for:
409:  - [ ] Functions (`@function.outer`, `@function.inner`)
410:  - [ ] Structs (`@struct.outer`, `@struct.inner`)
411:  - [ ] JSX elements (`@jsx.outer`, `@jsx.inner`)
412:  - [ ] Parameters (`@parameter.outer`)
- [ ] DX-TASKS:  Test in Neovim (via nvim-treesitter)
- [ ] DX-TASKS:  Document textobject usage
- [ ] DX-TASKS:  Add `hoverProvider` capability to LSP
- [ ] DX-TASKS:  Implement `hover` method:
429:  - [ ] Parse PHPX to AST
430:  - [ ] Find symbol at cursor position
431:  - [ ] Look up type information
432:  - [ ] Format hover contents (markdown)
- [ ] DX-TASKS:  Show hover info for:
434:  - [ ] Variables (show inferred type)
435:  - [ ] Functions (show signature)
436:  - [ ] Imports (show module path)
437:  - [ ] Struct fields (show type)
438:  - [ ] WASM imports (show WIT signature from `.d.phpx`)
- [ ] DX-TASKS:  Test with various PHPX constructs
- [ ] DX-TASKS:  Add `completionProvider` capability to LSP
- [ ] DX-TASKS:  Implement `completion` method:
448:  - [ ] Parse PHPX to AST
449:  - [ ] Determine completion context (import, variable, etc.)
450:  - [ ] Generate completion items
- [ ] DX-TASKS:  Provide completions for:
452:  - [ ] Import paths (scan `php_modules/`)
453:  - [ ] WASM modules (scan `php_modules/@*/`)
454:  - [ ] Exported functions from imports
455:  - [ ] Struct fields
456:  - [ ] Built-in types (`Option`, `Result`, `Object`)
457:  - [ ] PHPX stdlib functions
- [ ] DX-TASKS:  Add snippets for common patterns
- [ ] DX-TASKS:  Test in Zed
- [ ] DX-TASKS:  Add `definitionProvider` capability to LSP
- [ ] DX-TASKS:  Implement `goto_definition` method:
468:  - [ ] Find symbol at cursor
469:  - [ ] Resolve import paths
470:  - [ ] Find definition location
471:  - [ ] Return LSP `Location`
- [ ] DX-TASKS:  Support go-to-definition for:
473:  - [ ] Imported functions
474:  - [ ] Local variables
475:  - [ ] Struct definitions
476:  - [ ] WASM imports (jump to `.d.phpx` stub)
- [ ] DX-TASKS:  Test with multi-file projects
- [ ] DX-TASKS:  Extend compiler API to load `.d.phpx` stubs
- [ ] DX-TASKS:  Parse stub files for type information
- [ ] DX-TASKS:  Use stub types for:
487:  - [ ] Hover info on WASM imports
488:  - [ ] Autocomplete for WASM functions
489:  - [ ] Type checking WASM function calls
490:  - [ ] Go-to-definition (jump to stub)
- [ ] DX-TASKS:  Suggest generating stubs if missing
- [ ] DX-TASKS:  Test with WIT examples from `examples/wasm_hello_wit/`
- [ ] DX-TASKS:  Add `referencesProvider` capability to LSP
- [ ] DX-TASKS:  Implement `references` method:
501:  - [ ] Find all uses of symbol
502:  - [ ] Search across all files in workspace
503:  - [ ] Return LSP `Location` list
- [ ] DX-TASKS:  Support find-references for:
505:  - [ ] Functions
506:  - [ ] Variables
507:  - [ ] Imports
508:  - [ ] Struct types
- [ ] DX-TASKS:  Test with multi-file projects
- [ ] DX-TASKS:  Add `renameProvider` capability to LSP
- [ ] DX-TASKS:  Implement `rename` method:
518:  - [ ] Find all references to symbol
519:  - [ ] Generate `TextEdit` for each reference
520:  - [ ] Return `WorkspaceEdit`
- [ ] DX-TASKS:  Support renaming:
522:  - [ ] Variables
523:  - [ ] Functions
524:  - [ ] Imports (update import path)
525:  - [ ] Struct fields
- [ ] DX-TASKS:  Test rename across multiple files
- [ ] DX-TASKS:  Verify no broken references
- [ ] DX-TASKS:  Add `documentSymbolProvider` capability to LSP
- [ ] DX-TASKS:  Implement `document_symbol` method:
536:  - [ ] Parse PHPX to AST
537:  - [ ] Extract functions, structs, constants
538:  - [ ] Return LSP `DocumentSymbol` hierarchy
- [ ] DX-TASKS:  Show symbols in editor outline/breadcrumbs
- [ ] DX-TASKS:  Test with large PHPX files
- [ ] DX-TASKS:  Create `extensions/vscode-phpx/` directory
- [ ] DX-TASKS:  Initialize extension: `npm init` or `yo code`
- [ ] DX-TASKS:  Update `package.json` metadata
- [ ] DX-TASKS:  Create `syntaxes/phpx.tmLanguage.json` (TextMate grammar)
557:  - [ ] Port from tree-sitter grammar OR
558:  - [ ] Use tree-sitter WASM in extension
- [ ] DX-TASKS:  Add language configuration
- [ ] DX-TASKS:  Add file icon
- [ ] DX-TASKS:  Add `vscode-languageclient` dependency
- [ ] DX-TASKS:  Create `src/extension.ts`:
569:  - [ ] Start LSP server on activation
570:  - [ ] Configure server options
571:  - [ ] Handle server lifecycle
- [ ] DX-TASKS:  Bundle LSP binary with extension OR
- [ ] DX-TASKS:  Download binary on activation (GitHub releases)
- [ ] DX-TASKS:  Test extension locally: `code --extensionDevelopmentPath=.`
- [ ] DX-TASKS:  Option A: TextMate grammar in `syntaxes/`
- [ ] DX-TASKS:  Option B: tree-sitter WASM bundle
583:  - [ ] Compile tree-sitter grammar to WASM
584:  - [ ] Bundle in extension
585:  - [ ] Use `vscode-textmate` or `web-tree-sitter`
- [ ] DX-TASKS:  Test highlighting with PHPX files
- [ ] DX-TASKS:  Verify matches Zed highlighting
- [ ] DX-TASKS:  Create `.vsix` package: `vsce package`
- [ ] DX-TASKS:  Test installation: `code --install-extension phpx-0.1.0.vsix`
- [ ] DX-TASKS:  Create GitHub repository for extension
- [ ] DX-TASKS:  Write `README.md` with features and screenshots
- [ ] DX-TASKS:  Publish to VSCode Marketplace (optional):
599:  - [ ] Create publisher account
600:  - [ ] Run `vsce publish`
- [ ] DX-TASKS:  Add to Deka documentation
- [ ] DX-TASKS:  Create Neovim plugin structure: `nvim-phpx/`
- [ ] DX-TASKS:  Add tree-sitter grammar to nvim-treesitter:
616:  - [ ] Fork `nvim-treesitter`
617:  - [ ] Add parser config for PHPX
618:  - [ ] Submit PR to nvim-treesitter
- [ ] DX-TASKS:  Document installation (Lazy.nvim, Packer, etc.)
- [ ] DX-TASKS:  Test in Neovim
- [ ] DX-TASKS:  Document LSP setup with `nvim-lspconfig`:
- [ ] DX-TASKS:  Add autocommand for `.phpx` files
- [ ] DX-TASKS:  Test LSP features in Neovim
- [ ] DX-TASKS:  Document keybindings
- [ ] DX-TASKS:  Create LuaSnip snippets for PHPX
- [ ] DX-TASKS:  Add common patterns:
646:  - [ ] Function definition
647:  - [ ] Struct literal
648:  - [ ] Import statement
649:  - [ ] JSX component
650:  - [ ] Frontmatter template
- [ ] DX-TASKS:  Document snippet usage
- [ ] DX-TASKS:  Create `docs/editor-support.md`:
665:  - [ ] Overview of tree-sitter and LSP
666:  - [ ] Installation for each editor (Zed, VSCode, Neovim, Helix)
667:  - [ ] Feature comparison matrix
668:  - [ ] Troubleshooting guide
669:  - [ ] Known limitations
- [ ] DX-TASKS:  Update `CLAUDE.md` with editor support section
- [ ] DX-TASKS:  Add screenshots/GIFs to documentation
- [ ] DX-TASKS:  Create `scripts/install-phpx-lsp.sh`:
679:  - [ ] Build LSP binary
680:  - [ ] Install to `~/.local/bin/` or system path
681:  - [ ] Set up editor configs
- [ ] DX-TASKS:  Create `scripts/install-zed-extension.sh`
- [ ] DX-TASKS:  Create `scripts/install-vscode-extension.sh`
- [ ] DX-TASKS:  Test on clean systems (Linux, macOS)
- [ ] DX-TASKS:  Add GitHub Actions workflow:
692:  - [ ] Build tree-sitter grammar
693:  - [ ] Build LSP server
694:  - [ ] Run tests
695:  - [ ] Create releases with binaries
- [ ] DX-TASKS:  Build for multiple platforms:
697:  - [ ] Linux (x86_64)
698:  - [ ] macOS (x86_64, arm64)
699:  - [ ] Windows (x86_64)
- [ ] DX-TASKS:  Publish VSCode extension to marketplace (automated)
- [ ] DX-TASKS:  Write blog post/announcement:
708:  - [ ] Why PHPX needs editor support
709:  - [ ] What's included (tree-sitter, LSP)
710:  - [ ] How to install
711:  - [ ] Demo screenshots/GIFs
- [ ] DX-TASKS:  Create video tutorial (optional)
- [ ] DX-TASKS:  Post to appropriate channels
- [ ] DX-TASKS:  Update Deka website
- [ ] DX-TASKS:  `examples/strlen.phpx` - Simple type annotations
- [ ] DX-TASKS:  `examples/php/modules-import/index.php` - Import/export
- [ ] DX-TASKS:  `examples/bridge_array.phpx` - Struct literals
- [ ] DX-TASKS:  `examples/phpx-components/app.phpx` - JSX + frontmatter
- [ ] DX-TASKS:  `examples/wasm_hello_wit/` - WASM imports with WIT stubs
- [ ] DX-TASKS:  Create edge case files:
730:  - [ ] Nested JSX with PHPX expressions
731:  - [ ] Complex type annotations
732:  - [ ] Syntax errors
733:  - [ ] Type errors
734:  - [ ] Missing imports
- [ ] DX-TASKS:  **Prerequisite**: PHPX Validation System (See PHPX-VALIDATION.md)
801:  - [ ] Foundation (syntax, imports, PHPX rules)
802:  - [ ] Type system (type checking, generics)
803:  - [ ] Structs, JSX, modules, patterns
- [ ] DX-TASKS:  Phase 1: Tree-sitter Grammar (Not started)
- [ ] DX-TASKS:  Phase 2: LSP Server (Blocked by validation system)
- [ ] DX-TASKS:  Phase 3: Advanced Tree-sitter (Not started)
- [ ] DX-TASKS:  Phase 4: LSP Intelligence (Blocked by validation system)
- [ ] DX-TASKS:  Phase 5: VSCode Extension (Not started)
- [ ] DX-TASKS:  Phase 6: Neovim Support (Not started)
- [ ] DX-TASKS:  Phase 7: Documentation (Not started)
