# PHPX Editor Support - Development Tasks

This document tracks the implementation plan for first-class PHPX editor support across modern editors (Zed, VSCode, Neovim, etc.).

## Goals

1. **Syntax Highlighting**: PHPX files look beautiful with proper color coding
2. **Type Checking**: Real-time error detection as you type
3. **Autocomplete**: Intelligent suggestions for imports, functions, types
4. **Navigation**: Go-to-definition, find references, hover info
5. **Multi-Editor Support**: Works in Zed, VSCode, Neovim, and any LSP-compatible editor

## Strategy

We need **both** Tree-sitter (syntax highlighting) and LSP (intelligence):

- **Tree-sitter** → Visual feedback (syntax highlighting, code folding)
- **LSP** → Intelligence (diagnostics, autocomplete, navigation)

We can leverage the existing **`deka-validation`** crate for beautiful error formatting in the LSP.

---

## Phase 1: Tree-sitter Grammar (Syntax Highlighting)

**Goal**: Get PHPX syntax highlighting working in Zed and other tree-sitter-based editors

**Duration**: 1-2 weeks

### Task 1.1: Setup Tree-sitter Project
- [x] Create `tooling/tree-sitter-phpx/` directory
- [x] Clone `tree-sitter-php` as starting point
- [x] Rename project to `tree-sitter-phpx`
- [x] Update `package.json` metadata (name, description, repo)
- [x] Install tree-sitter CLI: `npm install -g tree-sitter-cli`
- [x] Verify build: `tree-sitter generate && tree-sitter test`

**Files to create**:
- `tooling/tree-sitter-phpx/grammar.js`
- `tooling/tree-sitter-phpx/package.json`

**Acceptance**: Can build and run `tree-sitter parse examples/hello.php`

---

### Task 1.2: Add PHPX Type Syntax
- [x] Add type annotation rules to `grammar.js`
  - [x] Primitive types: `int`, `string`, `bool`, `float`, `mixed`
  - [x] Generic types: `Option<T>`, `Result<T, E>`, `array<T>`
  - [x] Object types: `Object<{ field: Type }>`
  - [x] Type aliases: `type Name = ...`
- [x] Create `queries/highlights.scm` for type highlighting
- [x] Test with PHPX files containing type annotations
- [x] Verify types are highlighted differently from values

**Test files**:
- Create `test/corpus/types.txt` with PHPX type examples
- Run: `tree-sitter test`

**Acceptance**: Type annotations in `.phpx` files have correct syntax highlighting

---

### Task 1.3: Add Import/Export Syntax
- [x] Add `import_statement` rule to `grammar.js`
  - [x] Named imports: `import { foo, bar } from 'module'`
  - [x] WASM imports: `import { fn } from '@user/mod' as wasm`
  - [ ] Default import (if needed)
- [x] Add `export_statement` rule
  - [x] Export functions: `export function foo() {}`
  - [ ] Export constants: `export const X = 1`
  - [x] Re-exports: `export { foo } from './bar'`
- [x] Add highlighting for `import`, `export`, `from`, `as` keywords
- [x] Test with module examples from `examples/php/modules/`

**Acceptance**: Import/export statements have proper highlighting

---

### Task 1.4: Add Struct Literal Syntax
- [x] Add `struct_literal` rule to `grammar.js`
  - [x] Type name: `Point`
  - [x] Field list: `{ $x: 1, $y: 2 }`
  - [x] Shorthand: `{ $x, $y }`
- [x] Add highlighting for struct names and fields
- [x] Test with struct examples
- [x] Verify nested struct literals work

**Test case**:
```phpx
$p = Point { $x: 1, $y: 2 };
$user = User { $name: 'Sam', Profile { $bio: 'Dev' } };
```

**Acceptance**: Struct literals have proper highlighting, nested structs work

---

### Task 1.5: Add JSX Syntax Support
- [x] Port JSX grammar from `tree-sitter-javascript`
  - [x] Opening tags: `<Component>`
  - [x] Self-closing tags: `<Component />`
  - [x] Attributes: `<Component id={$val} />`
  - [x] Children: `<div>text</div>`
  - [x] Fragments: `<>...</>`
- [x] Add PHPX-specific JSX expressions
  - [x] Variables: `{$user->name}`
  - [x] Conditional expressions: `{$user->admin ? <Admin /> : null}`
  - [x] Object literals (double braces): `{{ host: 'localhost' }}`
  - [x] Statements are not allowed in JSX expressions (validation error; use expressions).
- [x] Add highlighting for tags, attributes, expressions
- [x] Test with component examples from `examples/phpx-components/`

**Acceptance**: JSX in `.phpx` files has proper highlighting, PHPX expressions work

---

### Task 1.6: Add Frontmatter Template Support
- [x] Add `frontmatter` rule to `grammar.js`
  - [x] Detect `---` at start of file
  - [x] Parse PHPX code section
  - [x] Parse JSX template section
- [x] Add highlighting for frontmatter delimiters
- [x] Test with template examples
- [x] Verify code and template sections have correct highlighting

**Test case**:
```phpx
---
import { Link } from 'component/dom';
$title = 'Home';
---

<html>
  <head><title>{$title}</title></head>
  <body><Link to="/about">About</Link></body>
</html>
```

**Acceptance**: Frontmatter templates have proper highlighting in both sections

---

### Task 1.7: Create Zed Extension
- [x] Create `extensions/phpx/` directory
- [x] Create `extension.toml` with PHPX language config
  - [x] Set file suffixes: `["phpx"]`
  - [x] Set comment syntax
  - [x] Link to tree-sitter-phpx grammar
- [ ] Add syntax highlighting theme overrides (if needed)
- [ ] Install extension in Zed:
  - [ ] Symlink: `ln -s /path/to/deka/extensions/phpx ~/.config/zed/extensions/phpx`
- [ ] Test with real PHPX files
- [ ] Verify highlighting works for all features

**Files to create**:
- `extensions/phpx/extension.toml`
- `extensions/phpx/languages/phpx/config.toml` (if needed)

**Acceptance**: PHPX files in Zed have full syntax highlighting

---

### Task 1.8: Document Tree-sitter Grammar
- [x] Create `tooling/tree-sitter-phpx/README.md`
  - [x] Installation instructions
  - [x] Testing instructions
  - [x] Editor integration guides (Zed, Neovim, Helix)
  - [x] Grammar rules overview
- [x] Add examples to `test/corpus/`
- [x] Document known limitations
- [x] Add contributing guidelines

**Acceptance**: Other developers can extend the grammar

---

## Phase 2: LSP Server (Type Checking & Diagnostics)

**Goal**: Get real-time error detection and diagnostics working

**Duration**: 2-3 weeks

### Task 2.1: Create LSP Crate
- [x] Create `crates/phpx_lsp/` directory
- [x] Initialize Cargo project: `cargo new phpx_lsp --bin`
- [x] Add dependencies to `Cargo.toml`:
  - [x] `tower-lsp = "0.20"`
  - [x] `tokio` (workspace)
  - [x] `serde_json = "1"`
  - [x] `anyhow = "1"`
  - [x] `modules_php` (path dependency to existing PHPX compiler)
  - [x] `deka-validation` (for error formatting)
- [x] Add to workspace members in root `Cargo.toml`
- [x] Verify build: `cargo build -p phpx_lsp`

**Acceptance**: LSP crate compiles successfully

---

### Task 2.2: Implement Basic LSP Server
- [x] Create `src/main.rs` with LSP boilerplate
- [x] Implement `initialize` method with server capabilities:
  - [x] `textDocumentSync`: Full sync mode
  - [x] `diagnosticProvider`: Report errors
  - [ ] (Others later: hover, completion, etc.)
- [x] Implement `initialized` method (log ready message)
- [x] Implement `shutdown` method
- [x] Implement `did_open` and `did_change` handlers (log only)
- [ ] Test with manual stdio: `echo '{"jsonrpc":"2.0","method":"initialize",...}' | cargo run`

**Acceptance**: LSP server responds to initialize and logs document events

---

### Task 2.3: Expose PHPX Compiler API
- [x] Create `crates/modules_php/src/compiler_api.rs`
- [x] Define public structs:
  ```rust
  pub struct CompilationResult {
      pub errors: Vec<CompileError>,
      pub warnings: Vec<CompileWarning>,
      pub ast: Option<Ast>,
  }

  pub struct CompileError {
      pub line: usize,
      pub column: usize,
      pub message: String,
      pub error_kind: String,
      pub underline_length: usize,
  }
  ```
- [x] Implement `compile_phpx(source: &str, file_path: &str) -> ValidationResult`
  - [x] Call existing PHPX parser/compiler
  - [x] Collect syntax errors
  - [x] Collect type errors
  - [x] Return structured results
- [x] Add unit tests for error collection
- [x] Export from `crates/modules_php/src/lib.rs`

**Acceptance**: Can call compiler API and get structured errors

---

### Task 2.4: Integrate Validation Formatting
- [x] Add `deka-validation` dependency to `phpx_lsp`
- [x] Implement error formatting in LSP:
  ```rust
  use deka_validation::format_validation_error;

  fn format_lsp_error(error: &CompileError, source: &str, file_path: &str) -> String {
      format_validation_error(
          source,
          file_path,
          &error.error_kind,
          error.line,
          error.column,
          &error.message,
          &error.help_text,
          error.underline_length,
      )
  }
  ```
- [x] Convert formatted errors to LSP Diagnostic messages
- [ ] Test with PHPX files containing errors
- [ ] Verify beautiful error output in editor

**Acceptance**: LSP diagnostics show formatted errors with help text

---

### Task 2.5: Implement Diagnostics Publishing
- [x] Implement `validate_document` method in LSP server:
  - [x] Call `compile_phpx` API
  - [x] Convert `CompileError` to LSP `Diagnostic`
  - [x] Map line/column positions
  - [x] Set severity (Error vs Warning)
  - [x] Include formatted message
- [x] Call `client.publish_diagnostics` on document open/change
- [ ] Test with PHPX files containing:
  - [ ] Syntax errors
  - [ ] Type errors
  - [ ] Import errors
  - [ ] WIT import errors (missing stubs)
- [ ] Verify errors appear in editor as red squiggles

**Acceptance**: Editor shows red squiggles on PHPX errors

---

### Task 2.6: Configure LSP in Zed
- [x] Update `extensions/phpx/extension.toml` with LSP config:
  ```toml
  [language_servers.phpx-lsp]
  name = "PHPX Language Server"
  language = "phpx"
  ```
- [ ] Add LSP binary path to Zed settings:
  ```json
  {
    "lsp": {
      "phpx-lsp": {
        "binary": {
          "path": "/path/to/deka/target/release/phpx_lsp"
        }
      }
    }
  }
  ```
- [ ] Rebuild LSP: `cargo build --release -p phpx_lsp`
- [ ] Restart Zed
- [ ] Test with PHPX files
- [ ] Verify diagnostics appear in problems panel

**Acceptance**: LSP runs in Zed and reports errors

---

### Task 2.7: Add Import Validation
- [ ] Extend compiler API to validate imports
- [ ] Check PHPX module imports:
  - [ ] Verify module exists in `php_modules/`
  - [ ] Verify exported names exist
  - [ ] Detect unused imports
  - [ ] Detect circular imports
- [ ] Check WASM imports:
  - [ ] Verify `deka.json` exists
  - [ ] Verify `module.wasm` exists
  - [ ] Check for `.d.phpx` stub file
  - [ ] Suggest running `deka wasm stubs` if missing
- [ ] Add helpful error messages with fixes
- [ ] Test with various import scenarios

**Example error**:
```
Import Error: Missing type stubs for '@user/hello'

= help: Run `deka wasm stubs @user/hello` to generate type stubs
```

**Acceptance**: Import errors have actionable help messages

---

### Task 2.8: Document LSP Server
- [ ] Create `crates/phpx_lsp/README.md`
  - [ ] Installation instructions
  - [ ] Editor integration guides (Zed, VSCode, Neovim)
  - [ ] Configuration options
  - [ ] Debugging tips
- [ ] Document compiler API in `crates/modules_php/src/compiler_api.rs`
- [ ] Add troubleshooting section
- [ ] List supported features and roadmap

**Acceptance**: Developers can set up and debug the LSP

---

## Phase 3: Advanced Tree-sitter Features

**Goal**: Improve tree-sitter grammar with edge cases

**Duration**: 1 week

### Task 3.1: Handle PHPX-Specific Edge Cases
- [ ] Test and fix tight dot access: `$user.name.first`
- [ ] Test and fix nested object literals in JSX: `{{ nested: { value: 1 } }}`
- [ ] Test and fix multiline JSX expressions
- [ ] Test and fix if/foreach blocks in JSX
- [ ] Add error recovery rules for better partial highlighting
- [ ] Test with malformed PHPX (ensure no crashes)

**Acceptance**: Tree-sitter handles edge cases gracefully

---

### Task 3.2: Add Code Folding
- [ ] Create `queries/folds.scm`
- [ ] Add folding for:
  - [ ] Function bodies
  - [ ] Struct definitions
  - [ ] JSX elements
  - [ ] If/foreach blocks
  - [ ] Frontmatter sections
- [ ] Test in Zed (verify fold markers appear)

**Acceptance**: PHPX code can be folded at logical boundaries

---

### Task 3.3: Add Indentation Rules
- [ ] Create `queries/indents.scm`
- [ ] Define indent increases for:
  - [ ] Function bodies
  - [ ] If/else/foreach blocks
  - [ ] JSX children
  - [ ] Struct/object literals
- [ ] Test auto-indentation in Zed
- [ ] Verify correct indent after newline

**Acceptance**: Auto-indentation works correctly for PHPX

---

### Task 3.4: Add Textobjects
- [ ] Create `queries/textobjects.scm`
- [ ] Define textobjects for:
  - [ ] Functions (`@function.outer`, `@function.inner`)
  - [ ] Structs (`@struct.outer`, `@struct.inner`)
  - [ ] JSX elements (`@jsx.outer`, `@jsx.inner`)
  - [ ] Parameters (`@parameter.outer`)
- [ ] Test in Neovim (via nvim-treesitter)
- [ ] Document textobject usage

**Acceptance**: Textobjects work in Neovim for PHPX code

---

## Phase 4: LSP Intelligence Features

**Goal**: Add autocomplete, hover, go-to-definition

**Duration**: 2-4 weeks

### Task 4.1: Implement Hover Provider
- [ ] Add `hoverProvider` capability to LSP
- [ ] Implement `hover` method:
  - [ ] Parse PHPX to AST
  - [ ] Find symbol at cursor position
  - [ ] Look up type information
  - [ ] Format hover contents (markdown)
- [ ] Show hover info for:
  - [ ] Variables (show inferred type)
  - [ ] Functions (show signature)
  - [ ] Imports (show module path)
  - [ ] Struct fields (show type)
  - [ ] WASM imports (show WIT signature from `.d.phpx`)
- [ ] Test with various PHPX constructs

**Acceptance**: Hover shows useful type info

---

### Task 4.2: Implement Completion Provider
- [ ] Add `completionProvider` capability to LSP
- [ ] Implement `completion` method:
  - [ ] Parse PHPX to AST
  - [ ] Determine completion context (import, variable, etc.)
  - [ ] Generate completion items
- [ ] Provide completions for:
  - [ ] Import paths (scan `php_modules/`)
  - [ ] WASM modules (scan `php_modules/@*/`)
  - [ ] Exported functions from imports
  - [ ] Struct fields
  - [ ] Built-in types (`Option`, `Result`, `Object`)
  - [ ] PHPX stdlib functions
- [ ] Add snippets for common patterns
- [ ] Test in Zed

**Acceptance**: Autocomplete suggests relevant items

---

### Task 4.3: Implement Go-to-Definition
- [ ] Add `definitionProvider` capability to LSP
- [ ] Implement `goto_definition` method:
  - [ ] Find symbol at cursor
  - [ ] Resolve import paths
  - [ ] Find definition location
  - [ ] Return LSP `Location`
- [ ] Support go-to-definition for:
  - [ ] Imported functions
  - [ ] Local variables
  - [ ] Struct definitions
  - [ ] WASM imports (jump to `.d.phpx` stub)
- [ ] Test with multi-file projects

**Acceptance**: Can jump to definitions across files

---

### Task 4.4: Implement WIT Stub Integration
- [ ] Extend compiler API to load `.d.phpx` stubs
- [ ] Parse stub files for type information
- [ ] Use stub types for:
  - [ ] Hover info on WASM imports
  - [ ] Autocomplete for WASM functions
  - [ ] Type checking WASM function calls
  - [ ] Go-to-definition (jump to stub)
- [ ] Suggest generating stubs if missing
- [ ] Test with WIT examples from `examples/wasm_hello_wit/`

**Acceptance**: LSP uses WIT stubs for WASM imports

---

### Task 4.5: Add Find References
- [ ] Add `referencesProvider` capability to LSP
- [ ] Implement `references` method:
  - [ ] Find all uses of symbol
  - [ ] Search across all files in workspace
  - [ ] Return LSP `Location` list
- [ ] Support find-references for:
  - [ ] Functions
  - [ ] Variables
  - [ ] Imports
  - [ ] Struct types
- [ ] Test with multi-file projects

**Acceptance**: Can find all references to a symbol

---

### Task 4.6: Add Rename Support
- [ ] Add `renameProvider` capability to LSP
- [ ] Implement `rename` method:
  - [ ] Find all references to symbol
  - [ ] Generate `TextEdit` for each reference
  - [ ] Return `WorkspaceEdit`
- [ ] Support renaming:
  - [ ] Variables
  - [ ] Functions
  - [ ] Imports (update import path)
  - [ ] Struct fields
- [ ] Test rename across multiple files
- [ ] Verify no broken references

**Acceptance**: Rename updates all references correctly

---

### Task 4.7: Add Document Symbols
- [ ] Add `documentSymbolProvider` capability to LSP
- [ ] Implement `document_symbol` method:
  - [ ] Parse PHPX to AST
  - [ ] Extract functions, structs, constants
  - [ ] Return LSP `DocumentSymbol` hierarchy
- [ ] Show symbols in editor outline/breadcrumbs
- [ ] Test with large PHPX files

**Acceptance**: Editor outline shows PHPX symbols

---

## Phase 5: VSCode Extension

**Goal**: Package for VSCode users

**Duration**: 1 week

### Task 5.1: Create VSCode Extension Scaffold
- [ ] Create `extensions/vscode-phpx/` directory
- [ ] Initialize extension: `npm init` or `yo code`
- [ ] Update `package.json` metadata
- [ ] Create `syntaxes/phpx.tmLanguage.json` (TextMate grammar)
  - [ ] Port from tree-sitter grammar OR
  - [ ] Use tree-sitter WASM in extension
- [ ] Add language configuration
- [ ] Add file icon

**Acceptance**: Basic VSCode extension structure exists

---

### Task 5.2: Integrate LSP Client
- [ ] Add `vscode-languageclient` dependency
- [ ] Create `src/extension.ts`:
  - [ ] Start LSP server on activation
  - [ ] Configure server options
  - [ ] Handle server lifecycle
- [ ] Bundle LSP binary with extension OR
- [ ] Download binary on activation (GitHub releases)
- [ ] Test extension locally: `code --extensionDevelopmentPath=.`

**Acceptance**: LSP runs in VSCode

---

### Task 5.3: Add Syntax Highlighting
- [ ] Option A: TextMate grammar in `syntaxes/`
- [ ] Option B: tree-sitter WASM bundle
  - [ ] Compile tree-sitter grammar to WASM
  - [ ] Bundle in extension
  - [ ] Use `vscode-textmate` or `web-tree-sitter`
- [ ] Test highlighting with PHPX files
- [ ] Verify matches Zed highlighting

**Acceptance**: PHPX has syntax highlighting in VSCode

---

### Task 5.4: Package and Publish
- [ ] Create `.vsix` package: `vsce package`
- [ ] Test installation: `code --install-extension phpx-0.1.0.vsix`
- [ ] Create GitHub repository for extension
- [ ] Write `README.md` with features and screenshots
- [ ] Publish to VSCode Marketplace (optional):
  - [ ] Create publisher account
  - [ ] Run `vsce publish`
- [ ] Add to Deka documentation

**Acceptance**: VSCode extension is installable and functional

---

## Phase 6: Neovim Support

**Goal**: Enable tree-sitter and LSP in Neovim

**Duration**: 2-3 days

### Task 6.1: Register Tree-sitter Grammar
- [ ] Create Neovim plugin structure: `nvim-phpx/`
- [ ] Add tree-sitter grammar to nvim-treesitter:
  - [ ] Fork `nvim-treesitter`
  - [ ] Add parser config for PHPX
  - [ ] Submit PR to nvim-treesitter
  - OR create standalone plugin
- [ ] Document installation (Lazy.nvim, Packer, etc.)
- [ ] Test in Neovim

**Acceptance**: PHPX has syntax highlighting in Neovim

---

### Task 6.2: Configure LSP
- [ ] Document LSP setup with `nvim-lspconfig`:
  ```lua
  require('lspconfig').phpx_lsp.setup({
    cmd = { '/path/to/deka/target/release/phpx_lsp' },
    filetypes = { 'phpx' },
  })
  ```
- [ ] Add autocommand for `.phpx` files
- [ ] Test LSP features in Neovim
- [ ] Document keybindings

**Acceptance**: PHPX LSP works in Neovim

---

### Task 6.3: Add Neovim Snippets
- [ ] Create LuaSnip snippets for PHPX
- [ ] Add common patterns:
  - [ ] Function definition
  - [ ] Struct literal
  - [ ] Import statement
  - [ ] JSX component
  - [ ] Frontmatter template
- [ ] Document snippet usage

**Acceptance**: Snippets available in Neovim

---

## Phase 7: Documentation and Distribution

**Goal**: Make it easy for developers to use PHPX editor support

**Duration**: 3-5 days

### Task 7.1: Write Comprehensive Documentation
- [ ] Create `docs/editor-support.md`:
  - [ ] Overview of tree-sitter and LSP
  - [ ] Installation for each editor (Zed, VSCode, Neovim, Helix)
  - [ ] Feature comparison matrix
  - [ ] Troubleshooting guide
  - [ ] Known limitations
- [ ] Update `CLAUDE.md` with editor support section
- [ ] Add screenshots/GIFs to documentation

**Acceptance**: Developers can set up any supported editor

---

### Task 7.2: Create Installation Scripts
- [ ] Create `scripts/install-phpx-lsp.sh`:
  - [ ] Build LSP binary
  - [ ] Install to `~/.local/bin/` or system path
  - [ ] Set up editor configs
- [ ] Create `scripts/install-zed-extension.sh`
- [ ] Create `scripts/install-vscode-extension.sh`
- [ ] Test on clean systems (Linux, macOS)

**Acceptance**: One-command installation for each editor

---

### Task 7.3: Set Up CI/CD
- [ ] Add GitHub Actions workflow:
  - [ ] Build tree-sitter grammar
  - [ ] Build LSP server
  - [ ] Run tests
  - [ ] Create releases with binaries
- [ ] Build for multiple platforms:
  - [ ] Linux (x86_64)
  - [ ] macOS (x86_64, arm64)
  - [ ] Windows (x86_64)
- [ ] Publish VSCode extension to marketplace (automated)

**Acceptance**: CI builds and tests on every commit

---

### Task 7.4: Create Announcement and Tutorial
- [ ] Write blog post/announcement:
  - [ ] Why PHPX needs editor support
  - [ ] What's included (tree-sitter, LSP)
  - [ ] How to install
  - [ ] Demo screenshots/GIFs
- [ ] Create video tutorial (optional)
- [ ] Post to appropriate channels
- [ ] Update Deka website

**Acceptance**: Community knows about PHPX editor support

---

## Testing Checklist

For each phase, test with these PHPX files:

- [ ] `examples/strlen.phpx` - Simple type annotations
- [ ] `examples/php/modules-import/index.php` - Import/export
- [ ] `examples/bridge_array.phpx` - Struct literals
- [ ] `examples/phpx-components/app.phpx` - JSX + frontmatter
- [ ] `examples/wasm_hello_wit/` - WASM imports with WIT stubs
- [ ] Create edge case files:
  - [ ] Nested JSX with PHPX expressions
  - [ ] Complex type annotations
  - [ ] Syntax errors
  - [ ] Type errors
  - [ ] Missing imports

---

## Success Metrics

We'll know we've succeeded when:

1. **Syntax Highlighting**: PHPX files look beautiful in Zed, VSCode, Neovim
2. **Error Detection**: Typos and type errors show red squiggles immediately
3. **Autocomplete**: Typing `import { ` suggests available modules
4. **Navigation**: Cmd+Click jumps to function definitions
5. **WIT Integration**: WASM imports show type hints from `.d.phpx` stubs
6. **Community Adoption**: Developers report PHPX feels like a first-class language

---

## Resources

### Tree-sitter
- Docs: https://tree-sitter.github.io/tree-sitter/
- tree-sitter-php: https://github.com/tree-sitter/tree-sitter-php
- tree-sitter-javascript (JSX): https://github.com/tree-sitter/tree-sitter-javascript

### LSP
- Tower LSP (Rust): https://github.com/ebkalderon/tower-lsp
- LSP Spec: https://microsoft.github.io/language-server-protocol/
- VSCode Language Extensions: https://code.visualstudio.com/api/language-extensions/overview

### Editor Integration
- Zed Extensions: https://zed.dev/docs/extensions
- nvim-treesitter: https://github.com/nvim-treesitter/nvim-treesitter
- nvim-lspconfig: https://github.com/neovim/nvim-lspconfig

### Existing Deka Code
- `deka-validation`: `/Users/samifouad/Projects/deka/deka-validation/src/lib.rs`
- PHPX compiler: `crates/modules_php/`
- PHPX examples: `examples/phpx-components/`, `examples/php/`

---

## Notes

- **Leverage `deka-validation`**: Use the existing validation crate for beautiful error messages in the LSP
- **Reuse PHPX compiler**: Don't rewrite the parser; expose a clean API from `modules_php`
- **Start small**: Get basic highlighting and error detection working first, then iterate
- **Test continuously**: Use real PHPX files from examples for testing
- **Document as you go**: Good docs prevent confusion later

---

## Prerequisites

**IMPORTANT**: Before implementing the LSP (Phase 2), we need to build out PHPX validation.

See **`PHPX-VALIDATION.md`** for the complete validation implementation plan. The validation system will:
- Catch syntax errors, type errors, and PHPX rule violations
- Use `deka-validation` for beautiful error formatting
- Power both the compiler (CLI) and LSP (editor)

**Recommended**: Complete Phases 1-2 of PHPX-VALIDATION.md before starting LSP implementation (Phase 2 of this document).

---

## Current Status

- [ ] **Prerequisite**: PHPX Validation System (See PHPX-VALIDATION.md)
  - [ ] Foundation (syntax, imports, PHPX rules)
  - [ ] Type system (type checking, generics)
  - [ ] Structs, JSX, modules, patterns
- [ ] Phase 1: Tree-sitter Grammar (Not started)
- [ ] Phase 2: LSP Server (Blocked by validation system)
- [ ] Phase 3: Advanced Tree-sitter (Not started)
- [ ] Phase 4: LSP Intelligence (Blocked by validation system)
- [ ] Phase 5: VSCode Extension (Not started)
- [ ] Phase 6: Neovim Support (Not started)
- [ ] Phase 7: Documentation (Not started)

**Last Updated**: 2026-02-01
