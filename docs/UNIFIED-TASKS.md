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
1. [x] Fix namespaced struct coercion (type metadata should use fully-qualified struct names).
2. [x] Resolve type aliases for applied types (e.g., `type Maybe<T> = Option<T>`).
3. [x] Define `Result` array schema rules (`ok` boolean vs truthy) and align coercion.
4. [x] Decide/document missing struct fields behavior (defaults vs unset).
5. [x] Add match expression inference (union of arm types) so `match` participates in return/assignment typing.
6. [x] Infer type params for `array<T>` from array literals / `Type::Array` actuals.
7. [x] Infer type params for `Option<T>` / `Result<T,E>` when actuals are enum cases (`Option::Some`, `Result::Ok`, `Result::Err`).
8. [x] Add method-call type checking for structs/interfaces (arity + arg types).
9. [x] Add dot-access typing for promoted embedded fields in inference (if any gap remains).
10. [x] Add `unset($obj.field)` support for dot access (ObjectMap + struct).
11. [x] Support `->` property access on ObjectMap for PHP compatibility
12. [x] Decide how ObjectMap crosses PHP boundary: keep ObjectMap or coerce to stdClass.
13. [x] Audit core object helpers (`get_class`, `property_exists`, `method_exists`, `count`) for ObjectMap/Struct semantics and document/implement decisions.
14. [x] Define object-literal equality semantics (`==`/`===`) and implement deep comparison.
15. [x] Add tests for dot-unset + object-literal equality (value semantics).
16. [ ] Implement JSX validation pass (syntax/expressions/components) as outlined in `docs/PHPX-VALIDATION.md`.
17. [x] Enforce JSX expression rules (no statements; object literals require `{{ }}`).
18. [x] Add optional JSX/VNode type inference (e.g., `VNode` return type for components).
19. [x] Decide on JSX whitespace normalization (current renderer trims text nodes).
20. [x] Verify unused-import detection in presence of synthetic JSX/runtime imports (avoid false positives/negatives).
21. [x] Decide whether `import` in `.php` should allow additional PHP statements before it when `<?php` is present (currently must be first non-empty line).
22. [x] Add explicit tests for duplicate imports, duplicate export aliases, and cyclic imports.
23. [x] Clarify behavior of `phpx_import()` when module load fails (panic/trigger_error/echo).
24. [x] Allow unary +/− constant expressions in struct defaults (e.g. `$x: int = -1`).
25. [x] Decide whether object/struct literals should be permitted as struct defaults; if yes, extend constant-expr validation + runtime init.
26. [x] ContextProvider should push/pop context even when JSX passes callable (not string).
27. [x] Decide on `createRoot` `mode` support (implement or remove + document).
28. [x] Implement or remove `Link` prefetch option (currently unused in hydration).
29. [x] Add helper to emit partial JSON responses with proper headers (or document required headers in examples).
30. [x] Clarify layout semantics (where layout id is set and when partial navigation falls back).
31. [x] Fix PHPX eval frame depth so eval'd PHPX can call functions (no stack underflow).
32. [x] Read export signatures from `__PHPX_TYPES` registry (avoid namespace-scoped type vars).
33. [x] Auto-add `core/option` + `core/result` as deps when referenced in PHPX modules.

### Phase 7 Tests/Docs (from `docs/TASKS.md`, non-checkbox items)
1. [x] Add PHP<->PHPX bridge tests for `Option<T>` (null -> None, Some -> value, return mapping).
2. [x] Add PHP<->PHPX bridge tests for `Result<T,E>` (Ok/Err return mapping; array/stdClass coercions).
3. [x] Add PHP<->PHPX bridge tests for object/object-shape + struct param coercion (extra keys ignored).
4. [x] Add runtime coverage for module isolation + unused import errors.
5. [ ] Add tests/docs for frontmatter templates + `<Hydration />`.
6. [x] Deduplicate boundary coercion bullets in `docs/phpx-dx.md`.

## Validation System (from `docs/VALIDATION-STATUS.md` + `docs/PHPX-VALIDATION.md`)

### Validation status gaps (explicit in `docs/VALIDATION-STATUS.md`)
1. [ ] Add `deka-validation` integration for parser errors (use formatter instead of basic text).
2. [ ] Extend `ParseError` to include `error_kind` and `help_text`.
3. [ ] Add PHPX error kinds (Syntax/Type/Import/Export/Null/OOP/JSX/etc.) for validation output.
4. [ ] Add compiler API: `compile_phpx(source, file_path) -> ValidationResult` with structured errors.
5. [ ] Return `ValidationResult { errors, warnings, ast }` instead of generic CoreError.

### Validation plan checklist (from `docs/PHPX-VALIDATION.md`)
1. [ ] PHPX-VALIDATION:  Unclosed braces, brackets, parentheses
2. [ ] PHPX-VALIDATION:  Invalid tokens
3. [ ] PHPX-VALIDATION:  Unexpected end of file
4. [ ] PHPX-VALIDATION:  Malformed expressions
5. [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/syntax.rs`
6. [ ] PHPX-VALIDATION:  Implement `validate_syntax(source: &str) -> Vec<ValidationError>`
7. [ ] PHPX-VALIDATION:  Use php-rs parser error recovery
8. [ ] PHPX-VALIDATION:  Map parser errors to validation errors
9. [ ] PHPX-VALIDATION:  Add helpful suggestions for common mistakes
10. [ ] PHPX-VALIDATION:  Import at top of file (before other code)
11. [ ] PHPX-VALIDATION:  Valid import syntax
12. [ ] PHPX-VALIDATION:  Named imports: `import { foo, bar } from 'module'`
13. [ ] PHPX-VALIDATION:  WASM imports: `import { fn } from '@user/mod' as wasm`
14. [ ] PHPX-VALIDATION:  Module path format (no relative paths with `../`)
15. [ ] PHPX-VALIDATION:  Unused imports (warning)
16. [ ] PHPX-VALIDATION:  Duplicate imports
17. [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/imports.rs`
18. [ ] PHPX-VALIDATION:  Implement `validate_imports(ast: &Ast) -> Vec<ValidationError>`
19. [ ] PHPX-VALIDATION:  Check import placement (AST position)
20. [ ] PHPX-VALIDATION:  Validate module paths
21. [ ] PHPX-VALIDATION:  Track used imports (mark on usage)
22. [ ] PHPX-VALIDATION:  Detect duplicates
23. [ ] PHPX-VALIDATION:  Export only functions, constants, types
24. [ ] PHPX-VALIDATION:  No duplicate exports
25. [ ] PHPX-VALIDATION:  Exported names actually exist
26. [ ] PHPX-VALIDATION:  Re-export syntax validation
27. [ ] PHPX-VALIDATION:  Template files: no explicit exports (auto-exported as Component)
28. [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/exports.rs`
29. [ ] PHPX-VALIDATION:  Implement `validate_exports(ast: &Ast) -> Vec<ValidationError>`
30. [ ] PHPX-VALIDATION:  Track exported names
31. [ ] PHPX-VALIDATION:  Check for duplicates
32. [ ] PHPX-VALIDATION:  Verify definitions exist
33. [ ] PHPX-VALIDATION:  Special handling for template files
34. [ ] PHPX-VALIDATION:  Valid type names
35. [ ] PHPX-VALIDATION:  Generic syntax: `Option<T>`, `Result<T, E>`, `array<T>`
36. [ ] PHPX-VALIDATION:  Object shape syntax: `Object<{ field: Type }>`
37. [ ] PHPX-VALIDATION:  Type alias syntax: `type Name = ...`
38. [ ] PHPX-VALIDATION:  Union types (limited): `int|float`
39. [ ] PHPX-VALIDATION:  No nullable types (`?T`, `T|null` are banned)
40. [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/type_syntax.rs`
41. [ ] PHPX-VALIDATION:  Implement `validate_type_annotations(ast: &Ast) -> Vec<ValidationError>`
42. [ ] PHPX-VALIDATION:  Reject `null`, `?T`, `T|null` syntax
43. [ ] PHPX-VALIDATION:  Validate generic parameter syntax
44. [ ] PHPX-VALIDATION:  Check object shape syntax
45. [ ] PHPX-VALIDATION:  Variable assignment type matches
46. [ ] PHPX-VALIDATION:  Function parameter types match arguments
47. [ ] PHPX-VALIDATION:  Return type matches returned value
48. [ ] PHPX-VALIDATION:  Binary operation types compatible
49. [ ] PHPX-VALIDATION:  Struct field types match literal values
50. [ ] PHPX-VALIDATION:  Safe widening only (int → float allowed, not reverse)
51. [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/type_checker.rs`
52. [ ] PHPX-VALIDATION:  Implement type inference engine
53. [ ] PHPX-VALIDATION:  Implement `check_types(ast: &Ast) -> Vec<ValidationError>`
54. [ ] PHPX-VALIDATION:  Build type environment (symbol table)
55. [ ] PHPX-VALIDATION:  Infer types for expressions
56. [ ] PHPX-VALIDATION:  Check compatibility at assignments/calls/returns
57. [ ] PHPX-VALIDATION:  Track widening rules
58. [ ] PHPX-VALIDATION:  Generic parameters are used
59. [ ] PHPX-VALIDATION:  Generic constraints are satisfied
60. [ ] PHPX-VALIDATION:  Type arguments provided where required
61. [ ] PHPX-VALIDATION:  Constraint syntax: `T: Reader`
62. [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/generics.rs`
63. [ ] PHPX-VALIDATION:  Implement `validate_generics(ast: &Ast) -> Vec<ValidationError>`
64. [ ] PHPX-VALIDATION:  Track generic parameters
65. [ ] PHPX-VALIDATION:  Check constraints
66. [ ] PHPX-VALIDATION:  Infer type arguments
67. [ ] PHPX-VALIDATION:  No `null` literals
68. [ ] PHPX-VALIDATION:  No `=== null` or `!== null` comparisons
69. [ ] PHPX-VALIDATION:  No `is_null()` calls
70. [ ] PHPX-VALIDATION:  Suggest `Option<T>` instead
71. [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/phpx_rules.rs`
72. [ ] PHPX-VALIDATION:  Implement `validate_no_null(ast: &Ast) -> Vec<ValidationError>`
73. [ ] PHPX-VALIDATION:  Scan AST for null literals
74. [ ] PHPX-VALIDATION:  Scan for null comparisons
75. [ ] PHPX-VALIDATION:  Scan for is_null() calls
76. [ ] PHPX-VALIDATION:  No `throw` statements
77. [ ] PHPX-VALIDATION:  No `try/catch/finally` blocks
78. [ ] PHPX-VALIDATION:  Suggest `Result<T, E>` instead
79. [ ] PHPX-VALIDATION:  Allow `panic()` for unrecoverable errors
80. [ ] PHPX-VALIDATION:  Add to `crates/modules_php/src/validation/phpx_rules.rs`
81. [ ] PHPX-VALIDATION:  Implement `validate_no_exceptions(ast: &Ast) -> Vec<ValidationError>`
82. [ ] PHPX-VALIDATION:  Scan for throw statements
83. [ ] PHPX-VALIDATION:  Scan for try/catch/finally
84. [ ] PHPX-VALIDATION:  No `class` declarations
85. [ ] PHPX-VALIDATION:  No `trait` declarations
86. [ ] PHPX-VALIDATION:  No `extends` keyword
87. [ ] PHPX-VALIDATION:  No `implements` keyword
88. [ ] PHPX-VALIDATION:  No `new` keyword
89. [ ] PHPX-VALIDATION:  No `interface` inheritance (structural interfaces only)
90. [ ] PHPX-VALIDATION:  Suggest structs instead
91. [ ] PHPX-VALIDATION:  Add to `crates/modules_php/src/validation/phpx_rules.rs`
92. [ ] PHPX-VALIDATION:  Implement `validate_no_oop(ast: &Ast) -> Vec<ValidationError>`
93. [ ] PHPX-VALIDATION:  Scan for class/trait/interface declarations
94. [ ] PHPX-VALIDATION:  Scan for extends/implements
95. [ ] PHPX-VALIDATION:  Scan for new keyword
96. [ ] PHPX-VALIDATION:  No `namespace` declarations
97. [ ] PHPX-VALIDATION:  No top-level `use` statements
98. [ ] PHPX-VALIDATION:  Suggest import/export instead
99. [ ] PHPX-VALIDATION:  Add to `crates/modules_php/src/validation/phpx_rules.rs`
100. [ ] PHPX-VALIDATION:  Implement `validate_no_namespace(ast: &Ast) -> Vec<ValidationError>`
101. [ ] PHPX-VALIDATION:  Scan for namespace declarations
102. [ ] PHPX-VALIDATION:  Scan for top-level use statements
103. [ ] PHPX-VALIDATION:  No `__construct` in PHPX structs
104. [ ] PHPX-VALIDATION:  Field defaults are constant expressions
105. [ ] PHPX-VALIDATION:  Field type annotations are valid
106. [ ] PHPX-VALIDATION:  No duplicate field names
107. [ ] PHPX-VALIDATION:  Struct composition (`use A`) is valid
108. [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/structs.rs`
109. [ ] PHPX-VALIDATION:  Implement `validate_struct_definitions(ast: &Ast) -> Vec<ValidationError>`
110. [ ] PHPX-VALIDATION:  Check for __construct
111. [ ] PHPX-VALIDATION:  Validate field defaults are constants
112. [ ] PHPX-VALIDATION:  Check for duplicate fields
113. [ ] PHPX-VALIDATION:  Validate composition
114. [ ] PHPX-VALIDATION:  All required fields provided
115. [ ] PHPX-VALIDATION:  No extra fields
116. [ ] PHPX-VALIDATION:  Field types match values
117. [ ] PHPX-VALIDATION:  Shorthand syntax valid
118. [ ] PHPX-VALIDATION:  Nested struct literals valid
119. [ ] PHPX-VALIDATION:  Add to `crates/modules_php/src/validation/structs.rs`
120. [ ] PHPX-VALIDATION:  Implement `validate_struct_literals(ast: &Ast) -> Vec<ValidationError>`
121. [ ] PHPX-VALIDATION:  Check required fields
122. [ ] PHPX-VALIDATION:  Reject extra fields
123. [ ] PHPX-VALIDATION:  Validate field types
124. [ ] PHPX-VALIDATION:  Handle shorthand syntax
125. [ ] PHPX-VALIDATION:  Valid tag names
126. [ ] PHPX-VALIDATION:  Matching opening/closing tags
127. [ ] PHPX-VALIDATION:  Valid attribute syntax
128. [ ] PHPX-VALIDATION:  Fragment syntax: `<>...</>`
129. [ ] PHPX-VALIDATION:  Self-closing tags
130. [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/jsx.rs`
131. [ ] PHPX-VALIDATION:  Implement `validate_jsx_syntax(ast: &Ast) -> Vec<ValidationError>`
132. [ ] PHPX-VALIDATION:  Check tag matching
133. [ ] PHPX-VALIDATION:  Validate attribute names
134. [ ] PHPX-VALIDATION:  Check self-closing vs paired tags
135. [ ] PHPX-VALIDATION:  Expression syntax: `{$var}`, `{$obj.field}`
136. [ ] PHPX-VALIDATION:  If blocks: `{if ($cond) { <p>yes</p> }}`
137. [ ] PHPX-VALIDATION:  Foreach loops: `{foreach ($items as $item) { <li>{$item}</li> }}`
138. [ ] PHPX-VALIDATION:  Object literals require double braces: `{{ key: 'value' }}`
139. [ ] PHPX-VALIDATION:  No statements in expressions (only expressions)
140. [ ] PHPX-VALIDATION:  Add to `crates/modules_php/src/validation/jsx.rs`
141. [ ] PHPX-VALIDATION:  Implement `validate_jsx_expressions(ast: &Ast) -> Vec<ValidationError>`
142. [ ] PHPX-VALIDATION:  Check expression syntax
143. [ ] PHPX-VALIDATION:  Validate if/foreach blocks
144. [ ] PHPX-VALIDATION:  Detect statements in expressions
145. [ ] PHPX-VALIDATION:  Check object literal braces
146. [ ] PHPX-VALIDATION:  Component names are capitalized (or imported)
147. [ ] PHPX-VALIDATION:  Built-in tags are lowercase
148. [ ] PHPX-VALIDATION:  Component props match definition (if available)
149. [ ] PHPX-VALIDATION:  Special components: `<Link>`, `<Hydration>`, `<ContextProvider>`
150. [ ] PHPX-VALIDATION:  Add to `crates/modules_php/src/validation/jsx.rs`
151. [ ] PHPX-VALIDATION:  Implement `validate_components(ast: &Ast) -> Vec<ValidationError>`
152. [ ] PHPX-VALIDATION:  Check component naming
153. [ ] PHPX-VALIDATION:  Validate props (if type info available)
154. [ ] PHPX-VALIDATION:  Track imported components
155. [ ] PHPX-VALIDATION:  Frontmatter starts at beginning of file
156. [ ] PHPX-VALIDATION:  Proper `---` delimiters
157. [ ] PHPX-VALIDATION:  No explicit exports in template files (under `php_modules/`)
158. [ ] PHPX-VALIDATION:  Template section is valid JSX
159. [ ] PHPX-VALIDATION:  Imports in frontmatter only
160. [ ] PHPX-VALIDATION:  Add to `crates/modules_php/src/validation/jsx.rs`
161. [ ] PHPX-VALIDATION:  Implement `validate_frontmatter(ast: &Ast, file_path: &str) -> Vec<ValidationError>`
162. [ ] PHPX-VALIDATION:  Check frontmatter position
163. [ ] PHPX-VALIDATION:  Validate delimiters
164. [ ] PHPX-VALIDATION:  Check for exports (if in php_modules/)
165. [ ] PHPX-VALIDATION:  Validate imports placement
166. [ ] PHPX-VALIDATION:  Module exists in `php_modules/`
167. [ ] PHPX-VALIDATION:  Module has valid entry point
168. [ ] PHPX-VALIDATION:  Circular imports detected
169. [ ] PHPX-VALIDATION:  Import/export names match
170. [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/modules.rs`
171. [ ] PHPX-VALIDATION:  Implement `validate_module_resolution(ast: &Ast, base_path: &str) -> Vec<ValidationError>`
172. [ ] PHPX-VALIDATION:  Scan php_modules/ for available modules
173. [ ] PHPX-VALIDATION:  Build dependency graph
174. [ ] PHPX-VALIDATION:  Detect cycles
175. [ ] PHPX-VALIDATION:  Check export names
176. [ ] PHPX-VALIDATION:  `@user/module` format
177. [ ] PHPX-VALIDATION:  `deka.json` exists
178. [ ] PHPX-VALIDATION:  `module.wasm` exists
179. [ ] PHPX-VALIDATION:  `.d.phpx` stub file exists (suggest generating if missing)
180. [ ] PHPX-VALIDATION:  Imported names exist in stubs
181. [ ] PHPX-VALIDATION:  Add to `crates/modules_php/src/validation/modules.rs`
182. [ ] PHPX-VALIDATION:  Implement `validate_wasm_imports(ast: &Ast) -> Vec<ValidationError>`
183. [ ] PHPX-VALIDATION:  Scan php_modules/@*/ for WASM modules
184. [ ] PHPX-VALIDATION:  Check deka.json, module.wasm, .d.phpx
185. [ ] PHPX-VALIDATION:  Parse .d.phpx for exported names
186. [ ] PHPX-VALIDATION:  Suggest deka wasm commands
187. [ ] PHPX-VALIDATION:  Enum match covers all cases
188. [ ] PHPX-VALIDATION:  No unreachable match arms
189. [ ] PHPX-VALIDATION:  Variable binding in match arms
190. [ ] PHPX-VALIDATION:  Payload destructuring correct
191. [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/validation/patterns.rs`
192. [ ] PHPX-VALIDATION:  Implement `validate_match_exhaustiveness(ast: &Ast) -> Vec<ValidationError>`
193. [ ] PHPX-VALIDATION:  Build enum case registry
194. [ ] PHPX-VALIDATION:  Check match coverage
195. [ ] PHPX-VALIDATION:  Validate payload destructuring
196. [ ] PHPX-VALIDATION:  Detect unreachable arms
197. [ ] PHPX-VALIDATION:  Create `crates/modules_php/src/compiler_api.rs`
198. [ ] PHPX-VALIDATION:  Expose `compile_phpx(source: &str, file_path: &str) -> ValidationResult`
199. [ ] PHPX-VALIDATION:  Run all validation passes in order:
200. [ ] PHPX-VALIDATION:  Collect all errors and warnings
201. [ ] PHPX-VALIDATION:  Format with `deka-validation`
202. [ ] PHPX-VALIDATION:  Return structured result
203. [ ] PHPX-VALIDATION:  Create test file for each validation rule
204. [ ] PHPX-VALIDATION:  Positive tests (valid code passes)
205. [ ] PHPX-VALIDATION:  Negative tests (invalid code caught)
206. [ ] PHPX-VALIDATION:  Error message quality
207. [ ] PHPX-VALIDATION:  Help text accuracy
208. [ ] PHPX-VALIDATION:  Edge cases
209. [ ] PHPX-VALIDATION:  Support for multiple errors in single file
210. [ ] PHPX-VALIDATION:  Color coding by error kind
211. [ ] PHPX-VALIDATION:  Code suggestions (auto-fix hints)
212. [ ] PHPX-VALIDATION:  Reference links to PHPX docs
213. [ ] PHPX-VALIDATION:  `format_multiple_errors()` - Format list of errors
214. [ ] PHPX-VALIDATION:  Color codes by severity (error=red, warning=yellow, info=blue)
215. [ ] PHPX-VALIDATION:  `format_with_suggestion()` - Include code fix suggestions
216. [ ] PHPX-VALIDATION:  `format_with_docs_link()` - Add doc links

## DX / Tooling (from `docs/DX-TASKS.md`)
1. [ ] DX-TASKS:  Create `tooling/tree-sitter-phpx/` directory
2. [ ] DX-TASKS:  Clone `tree-sitter-php` as starting point
3. [ ] DX-TASKS:  Rename project to `tree-sitter-phpx`
4. [ ] DX-TASKS:  Update `package.json` metadata (name, description, repo)
5. [ ] DX-TASKS:  Install tree-sitter CLI: `npm install -g tree-sitter-cli`
6. [ ] DX-TASKS:  Verify build: `tree-sitter generate && tree-sitter test`
7. [ ] DX-TASKS:  Add type annotation rules to `grammar.js`
48:  - [ ] Primitive types: `int`, `string`, `bool`, `float`, `mixed`
49:  - [ ] Generic types: `Option<T>`, `Result<T, E>`, `array<T>`
50:  - [ ] Object types: `Object<{ field: Type }>`
51:  - [ ] Type aliases: `type Name = ...`
8. [ ] DX-TASKS:  Create `queries/highlights.scm` for type highlighting
9. [ ] DX-TASKS:  Test with PHPX files containing type annotations
10. [ ] DX-TASKS:  Verify types are highlighted differently from values
11. [ ] DX-TASKS:  Add `import_statement` rule to `grammar.js`
66:  - [ ] Named imports: `import { foo, bar } from 'module'`
67:  - [ ] WASM imports: `import { fn } from '@user/mod' as wasm`
68:  - [ ] Default import (if needed)
12. [ ] DX-TASKS:  Add `export_statement` rule
70:  - [ ] Export functions: `export function foo() {}`
71:  - [ ] Export constants: `export const X = 1`
72:  - [ ] Re-exports: `export { foo } from './bar'`
13. [ ] DX-TASKS:  Add highlighting for `import`, `export`, `from`, `as` keywords
14. [ ] DX-TASKS:  Test with module examples from `examples/php/modules/`
15. [ ] DX-TASKS:  Add `struct_literal` rule to `grammar.js`
82:  - [ ] Type name: `Point`
83:  - [ ] Field list: `{ $x: 1, $y: 2 }`
84:  - [ ] Shorthand: `{ $x, $y }`
16. [ ] DX-TASKS:  Add highlighting for struct names and fields
17. [ ] DX-TASKS:  Test with struct examples
18. [ ] DX-TASKS:  Verify nested struct literals work
19. [ ] DX-TASKS:  Port JSX grammar from `tree-sitter-javascript`
101:  - [ ] Opening tags: `<Component>`
102:  - [ ] Self-closing tags: `<Component />`
103:  - [ ] Attributes: `<Component id={$val} />`
104:  - [ ] Children: `<div>text</div>`
105:  - [ ] Fragments: `<>...</>`
20. [ ] DX-TASKS:  Add PHPX-specific JSX expressions
107:  - [ ] Variables: `{$user.name}`
108:  - [ ] If blocks: `{if ($x) { <p>yes</p> }}`
109:  - [ ] Foreach loops: `{foreach ($items as $item) { <li>{$item}</li> }}`
110:  - [ ] Object literals (double braces): `{{ host: 'localhost' }}`
21. [ ] DX-TASKS:  Add highlighting for tags, attributes, expressions
22. [ ] DX-TASKS:  Test with component examples from `examples/phpx-components/`
23. [ ] DX-TASKS:  Add `frontmatter` rule to `grammar.js`
120:  - [ ] Detect `---` at start of file
121:  - [ ] Parse PHPX code section
122:  - [ ] Parse JSX template section
24. [ ] DX-TASKS:  Add highlighting for frontmatter delimiters
25. [ ] DX-TASKS:  Test with template examples
26. [ ] DX-TASKS:  Verify code and template sections have correct highlighting
27. [ ] DX-TASKS:  Create `extensions/phpx/` directory
28. [ ] DX-TASKS:  Create `extension.toml` with PHPX language config
147:  - [ ] Set file suffixes: `["phpx"]`
148:  - [ ] Set comment syntax
149:  - [ ] Link to tree-sitter-phpx grammar
29. [ ] DX-TASKS:  Add syntax highlighting theme overrides (if needed)
30. [ ] DX-TASKS:  Install extension in Zed:
152:  - [ ] Symlink: `ln -s /path/to/deka/extensions/phpx ~/.config/zed/extensions/phpx`
31. [ ] DX-TASKS:  Test with real PHPX files
32. [ ] DX-TASKS:  Verify highlighting works for all features
33. [ ] DX-TASKS:  Create `tooling/tree-sitter-phpx/README.md`
166:  - [ ] Installation instructions
167:  - [ ] Testing instructions
168:  - [ ] Editor integration guides (Zed, Neovim, Helix)
169:  - [ ] Grammar rules overview
34. [ ] DX-TASKS:  Add examples to `test/corpus/`
35. [ ] DX-TASKS:  Document known limitations
36. [ ] DX-TASKS:  Add contributing guidelines
37. [ ] DX-TASKS:  Create `crates/phpx_lsp/` directory
38. [ ] DX-TASKS:  Initialize Cargo project: `cargo new phpx_lsp --bin`
39. [ ] DX-TASKS:  Add dependencies to `Cargo.toml`:
188:  - [ ] `tower-lsp = "0.20"`
189:  - [ ] `tokio` (workspace)
190:  - [ ] `serde_json = "1"`
191:  - [ ] `anyhow = "1"`
192:  - [ ] `modules_php` (path dependency to existing PHPX compiler)
193:  - [ ] `deka-validation` (for error formatting)
40. [ ] DX-TASKS:  Add to workspace members in root `Cargo.toml`
41. [ ] DX-TASKS:  Verify build: `cargo build -p phpx_lsp`
42. [ ] DX-TASKS:  Create `src/main.rs` with LSP boilerplate
43. [ ] DX-TASKS:  Implement `initialize` method with server capabilities:
204:  - [ ] `textDocumentSync`: Full sync mode
205:  - [ ] `diagnosticProvider`: Report errors
206:  - [ ] (Others later: hover, completion, etc.)
44. [ ] DX-TASKS:  Implement `initialized` method (log ready message)
45. [ ] DX-TASKS:  Implement `shutdown` method
46. [ ] DX-TASKS:  Implement `did_open` and `did_change` handlers (log only)
47. [ ] DX-TASKS:  Test with manual stdio: `echo '{"jsonrpc":"2.0","method":"initialize",...}' | cargo run`
48. [ ] DX-TASKS:  Create `crates/modules_php/src/compiler_api.rs`
49. [ ] DX-TASKS:  Define public structs:
50. [ ] DX-TASKS:  Implement `compile_phpx(source: &str, file_path: &str) -> CompilationResult`
235:  - [ ] Call existing PHPX parser/compiler
236:  - [ ] Collect syntax errors
237:  - [ ] Collect type errors
238:  - [ ] Return structured results
51. [ ] DX-TASKS:  Add unit tests for error collection
52. [ ] DX-TASKS:  Export from `crates/modules_php/src/lib.rs`
53. [ ] DX-TASKS:  Add `deka-validation` dependency to `phpx_lsp`
54. [ ] DX-TASKS:  Implement error formatting in LSP:
55. [ ] DX-TASKS:  Convert formatted errors to LSP Diagnostic messages
56. [ ] DX-TASKS:  Test with PHPX files containing errors
57. [ ] DX-TASKS:  Verify beautiful error output in editor
58. [ ] DX-TASKS:  Implement `validate_document` method in LSP server:
275:  - [ ] Call `compile_phpx` API
276:  - [ ] Convert `CompileError` to LSP `Diagnostic`
277:  - [ ] Map line/column positions
278:  - [ ] Set severity (Error vs Warning)
279:  - [ ] Include formatted message
59. [ ] DX-TASKS:  Call `client.publish_diagnostics` on document open/change
60. [ ] DX-TASKS:  Test with PHPX files containing:
282:  - [ ] Syntax errors
283:  - [ ] Type errors
284:  - [ ] Import errors
285:  - [ ] WIT import errors (missing stubs)
61. [ ] DX-TASKS:  Verify errors appear in editor as red squiggles
62. [ ] DX-TASKS:  Update `extensions/phpx/extension.toml` with LSP config:
63. [ ] DX-TASKS:  Add LSP binary path to Zed settings:
64. [ ] DX-TASKS:  Rebuild LSP: `cargo build --release -p phpx_lsp`
65. [ ] DX-TASKS:  Restart Zed
66. [ ] DX-TASKS:  Test with PHPX files
67. [ ] DX-TASKS:  Verify diagnostics appear in problems panel
68. [ ] DX-TASKS:  Extend compiler API to validate imports
69. [ ] DX-TASKS:  Check PHPX module imports:
323:  - [ ] Verify module exists in `php_modules/`
324:  - [ ] Verify exported names exist
325:  - [ ] Detect unused imports
326:  - [ ] Detect circular imports
70. [ ] DX-TASKS:  Check WASM imports:
328:  - [ ] Verify `deka.json` exists
329:  - [ ] Verify `module.wasm` exists
330:  - [ ] Check for `.d.phpx` stub file
331:  - [ ] Suggest running `deka wasm stubs` if missing
71. [ ] DX-TASKS:  Add helpful error messages with fixes
72. [ ] DX-TASKS:  Test with various import scenarios
73. [ ] DX-TASKS:  Create `crates/phpx_lsp/README.md`
348:  - [ ] Installation instructions
349:  - [ ] Editor integration guides (Zed, VSCode, Neovim)
350:  - [ ] Configuration options
351:  - [ ] Debugging tips
74. [ ] DX-TASKS:  Document compiler API in `crates/modules_php/src/compiler_api.rs`
75. [ ] DX-TASKS:  Add troubleshooting section
76. [ ] DX-TASKS:  List supported features and roadmap
77. [ ] DX-TASKS:  Test and fix tight dot access: `$user.name.first`
78. [ ] DX-TASKS:  Test and fix nested object literals in JSX: `{{ nested: { value: 1 } }}`
79. [ ] DX-TASKS:  Test and fix multiline JSX expressions
80. [ ] DX-TASKS:  Test and fix if/foreach blocks in JSX
81. [ ] DX-TASKS:  Add error recovery rules for better partial highlighting
82. [ ] DX-TASKS:  Test with malformed PHPX (ensure no crashes)
83. [ ] DX-TASKS:  Create `queries/folds.scm`
84. [ ] DX-TASKS:  Add folding for:
381:  - [ ] Function bodies
382:  - [ ] Struct definitions
383:  - [ ] JSX elements
384:  - [ ] If/foreach blocks
385:  - [ ] Frontmatter sections
85. [ ] DX-TASKS:  Test in Zed (verify fold markers appear)
86. [ ] DX-TASKS:  Create `queries/indents.scm`
87. [ ] DX-TASKS:  Define indent increases for:
395:  - [ ] Function bodies
396:  - [ ] If/else/foreach blocks
397:  - [ ] JSX children
398:  - [ ] Struct/object literals
88. [ ] DX-TASKS:  Test auto-indentation in Zed
89. [ ] DX-TASKS:  Verify correct indent after newline
90. [ ] DX-TASKS:  Create `queries/textobjects.scm`
91. [ ] DX-TASKS:  Define textobjects for:
409:  - [ ] Functions (`@function.outer`, `@function.inner`)
410:  - [ ] Structs (`@struct.outer`, `@struct.inner`)
411:  - [ ] JSX elements (`@jsx.outer`, `@jsx.inner`)
412:  - [ ] Parameters (`@parameter.outer`)
92. [ ] DX-TASKS:  Test in Neovim (via nvim-treesitter)
93. [ ] DX-TASKS:  Document textobject usage
94. [ ] DX-TASKS:  Add `hoverProvider` capability to LSP
95. [ ] DX-TASKS:  Implement `hover` method:
429:  - [ ] Parse PHPX to AST
430:  - [ ] Find symbol at cursor position
431:  - [ ] Look up type information
432:  - [ ] Format hover contents (markdown)
96. [ ] DX-TASKS:  Show hover info for:
434:  - [ ] Variables (show inferred type)
435:  - [ ] Functions (show signature)
436:  - [ ] Imports (show module path)
437:  - [ ] Struct fields (show type)
438:  - [ ] WASM imports (show WIT signature from `.d.phpx`)
97. [ ] DX-TASKS:  Test with various PHPX constructs
98. [ ] DX-TASKS:  Add `completionProvider` capability to LSP
99. [ ] DX-TASKS:  Implement `completion` method:
448:  - [ ] Parse PHPX to AST
449:  - [ ] Determine completion context (import, variable, etc.)
450:  - [ ] Generate completion items
100. [ ] DX-TASKS:  Provide completions for:
452:  - [ ] Import paths (scan `php_modules/`)
453:  - [ ] WASM modules (scan `php_modules/@*/`)
454:  - [ ] Exported functions from imports
455:  - [ ] Struct fields
456:  - [ ] Built-in types (`Option`, `Result`, `Object`)
457:  - [ ] PHPX stdlib functions
101. [ ] DX-TASKS:  Add snippets for common patterns
102. [ ] DX-TASKS:  Test in Zed
103. [ ] DX-TASKS:  Add `definitionProvider` capability to LSP
104. [ ] DX-TASKS:  Implement `goto_definition` method:
468:  - [ ] Find symbol at cursor
469:  - [ ] Resolve import paths
470:  - [ ] Find definition location
471:  - [ ] Return LSP `Location`
105. [ ] DX-TASKS:  Support go-to-definition for:
473:  - [ ] Imported functions
474:  - [ ] Local variables
475:  - [ ] Struct definitions
476:  - [ ] WASM imports (jump to `.d.phpx` stub)
106. [ ] DX-TASKS:  Test with multi-file projects
107. [ ] DX-TASKS:  Extend compiler API to load `.d.phpx` stubs
108. [ ] DX-TASKS:  Parse stub files for type information
109. [ ] DX-TASKS:  Use stub types for:
487:  - [ ] Hover info on WASM imports
488:  - [ ] Autocomplete for WASM functions
489:  - [ ] Type checking WASM function calls
490:  - [ ] Go-to-definition (jump to stub)
110. [ ] DX-TASKS:  Suggest generating stubs if missing
111. [ ] DX-TASKS:  Test with WIT examples from `examples/wasm_hello_wit/`
112. [ ] DX-TASKS:  Add `referencesProvider` capability to LSP
113. [ ] DX-TASKS:  Implement `references` method:
501:  - [ ] Find all uses of symbol
502:  - [ ] Search across all files in workspace
503:  - [ ] Return LSP `Location` list
114. [ ] DX-TASKS:  Support find-references for:
505:  - [ ] Functions
506:  - [ ] Variables
507:  - [ ] Imports
508:  - [ ] Struct types
115. [ ] DX-TASKS:  Test with multi-file projects
116. [ ] DX-TASKS:  Add `renameProvider` capability to LSP
117. [ ] DX-TASKS:  Implement `rename` method:
518:  - [ ] Find all references to symbol
519:  - [ ] Generate `TextEdit` for each reference
520:  - [ ] Return `WorkspaceEdit`
118. [ ] DX-TASKS:  Support renaming:
522:  - [ ] Variables
523:  - [ ] Functions
524:  - [ ] Imports (update import path)
525:  - [ ] Struct fields
119. [ ] DX-TASKS:  Test rename across multiple files
120. [ ] DX-TASKS:  Verify no broken references
121. [ ] DX-TASKS:  Add `documentSymbolProvider` capability to LSP
122. [ ] DX-TASKS:  Implement `document_symbol` method:
536:  - [ ] Parse PHPX to AST
537:  - [ ] Extract functions, structs, constants
538:  - [ ] Return LSP `DocumentSymbol` hierarchy
123. [ ] DX-TASKS:  Show symbols in editor outline/breadcrumbs
124. [ ] DX-TASKS:  Test with large PHPX files
125. [ ] DX-TASKS:  Create `extensions/vscode-phpx/` directory
126. [ ] DX-TASKS:  Initialize extension: `npm init` or `yo code`
127. [ ] DX-TASKS:  Update `package.json` metadata
128. [ ] DX-TASKS:  Create `syntaxes/phpx.tmLanguage.json` (TextMate grammar)
557:  - [ ] Port from tree-sitter grammar OR
558:  - [ ] Use tree-sitter WASM in extension
129. [ ] DX-TASKS:  Add language configuration
130. [ ] DX-TASKS:  Add file icon
131. [ ] DX-TASKS:  Add `vscode-languageclient` dependency
132. [ ] DX-TASKS:  Create `src/extension.ts`:
569:  - [ ] Start LSP server on activation
570:  - [ ] Configure server options
571:  - [ ] Handle server lifecycle
133. [ ] DX-TASKS:  Bundle LSP binary with extension OR
134. [ ] DX-TASKS:  Download binary on activation (GitHub releases)
135. [ ] DX-TASKS:  Test extension locally: `code --extensionDevelopmentPath=.`
136. [ ] DX-TASKS:  Option A: TextMate grammar in `syntaxes/`
137. [ ] DX-TASKS:  Option B: tree-sitter WASM bundle
583:  - [ ] Compile tree-sitter grammar to WASM
584:  - [ ] Bundle in extension
585:  - [ ] Use `vscode-textmate` or `web-tree-sitter`
138. [ ] DX-TASKS:  Test highlighting with PHPX files
139. [ ] DX-TASKS:  Verify matches Zed highlighting
140. [ ] DX-TASKS:  Create `.vsix` package: `vsce package`
141. [ ] DX-TASKS:  Test installation: `code --install-extension phpx-0.1.0.vsix`
142. [ ] DX-TASKS:  Create GitHub repository for extension
143. [ ] DX-TASKS:  Write `README.md` with features and screenshots
144. [ ] DX-TASKS:  Publish to VSCode Marketplace (optional):
599:  - [ ] Create publisher account
600:  - [ ] Run `vsce publish`
145. [ ] DX-TASKS:  Add to Deka documentation
146. [ ] DX-TASKS:  Create Neovim plugin structure: `nvim-phpx/`
147. [ ] DX-TASKS:  Add tree-sitter grammar to nvim-treesitter:
616:  - [ ] Fork `nvim-treesitter`
617:  - [ ] Add parser config for PHPX
618:  - [ ] Submit PR to nvim-treesitter
148. [ ] DX-TASKS:  Document installation (Lazy.nvim, Packer, etc.)
149. [ ] DX-TASKS:  Test in Neovim
150. [ ] DX-TASKS:  Document LSP setup with `nvim-lspconfig`:
151. [ ] DX-TASKS:  Add autocommand for `.phpx` files
152. [ ] DX-TASKS:  Test LSP features in Neovim
153. [ ] DX-TASKS:  Document keybindings
154. [ ] DX-TASKS:  Create LuaSnip snippets for PHPX
155. [ ] DX-TASKS:  Add common patterns:
646:  - [ ] Function definition
647:  - [ ] Struct literal
648:  - [ ] Import statement
649:  - [ ] JSX component
650:  - [ ] Frontmatter template
156. [ ] DX-TASKS:  Document snippet usage
157. [ ] DX-TASKS:  Create `docs/editor-support.md`:
665:  - [ ] Overview of tree-sitter and LSP
666:  - [ ] Installation for each editor (Zed, VSCode, Neovim, Helix)
667:  - [ ] Feature comparison matrix
668:  - [ ] Troubleshooting guide
669:  - [ ] Known limitations
158. [ ] DX-TASKS:  Update `CLAUDE.md` with editor support section
159. [ ] DX-TASKS:  Add screenshots/GIFs to documentation
160. [ ] DX-TASKS:  Create `scripts/install-phpx-lsp.sh`:
679:  - [ ] Build LSP binary
680:  - [ ] Install to `~/.local/bin/` or system path
681:  - [ ] Set up editor configs
161. [ ] DX-TASKS:  Create `scripts/install-zed-extension.sh`
162. [ ] DX-TASKS:  Create `scripts/install-vscode-extension.sh`
163. [ ] DX-TASKS:  Test on clean systems (Linux, macOS)
164. [ ] DX-TASKS:  Add GitHub Actions workflow:
692:  - [ ] Build tree-sitter grammar
693:  - [ ] Build LSP server
694:  - [ ] Run tests
695:  - [ ] Create releases with binaries
165. [ ] DX-TASKS:  Build for multiple platforms:
697:  - [ ] Linux (x86_64)
698:  - [ ] macOS (x86_64, arm64)
699:  - [ ] Windows (x86_64)
166. [ ] DX-TASKS:  Publish VSCode extension to marketplace (automated)
167. [ ] DX-TASKS:  Write blog post/announcement:
708:  - [ ] Why PHPX needs editor support
709:  - [ ] What's included (tree-sitter, LSP)
710:  - [ ] How to install
711:  - [ ] Demo screenshots/GIFs
168. [ ] DX-TASKS:  Create video tutorial (optional)
169. [ ] DX-TASKS:  Post to appropriate channels
170. [ ] DX-TASKS:  Update Deka website
171. [ ] DX-TASKS:  `examples/strlen.phpx` - Simple type annotations
172. [ ] DX-TASKS:  `examples/php/modules-import/index.php` - Import/export
173. [ ] DX-TASKS:  `examples/bridge_array.phpx` - Struct literals
174. [ ] DX-TASKS:  `examples/phpx-components/app.phpx` - JSX + frontmatter
175. [ ] DX-TASKS:  `examples/wasm_hello_wit/` - WASM imports with WIT stubs
176. [ ] DX-TASKS:  Create edge case files:
730:  - [ ] Nested JSX with PHPX expressions
731:  - [ ] Complex type annotations
732:  - [ ] Syntax errors
733:  - [ ] Type errors
734:  - [ ] Missing imports
177. [ ] DX-TASKS:  **Prerequisite**: PHPX Validation System (See PHPX-VALIDATION.md)
801:  - [ ] Foundation (syntax, imports, PHPX rules)
802:  - [ ] Type system (type checking, generics)
803:  - [ ] Structs, JSX, modules, patterns
178. [ ] DX-TASKS:  Phase 1: Tree-sitter Grammar (Not started)
179. [ ] DX-TASKS:  Phase 2: LSP Server (Blocked by validation system)
180. [ ] DX-TASKS:  Phase 3: Advanced Tree-sitter (Not started)
181. [ ] DX-TASKS:  Phase 4: LSP Intelligence (Blocked by validation system)
182. [ ] DX-TASKS:  Phase 5: VSCode Extension (Not started)
183. [ ] DX-TASKS:  Phase 6: Neovim Support (Not started)
184. [ ] DX-TASKS:  Phase 7: Documentation (Not started)
