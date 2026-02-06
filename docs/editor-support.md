# PHPX Editor Support

## Overview
PHPX editor support is split into two layers:
- Syntax layer: grammar/highlighting for `.phpx` files.
- Semantic layer: `phpx_lsp` for diagnostics, hover, completion, navigation, references, and rename.

## Install

### Zed
1. Install the local dev extension from `extensions/phpx` (Install Dev Extension).
2. Build binaries:
```sh
cargo build --release -p cli -p phpx_lsp
```
3. Add in `~/.config/zed/settings.json`:
```json
{
  "lsp": {
    "phpx-lsp": {
      "binary": {
        "path": "/absolute/path/to/deka/target/release/phpx_lsp"
      }
    }
  }
}
```

### VS Code
1. Install extension from `extensions/vscode-phpx`:
```sh
cd extensions/vscode-phpx
npm install
npm run package
code --install-extension vscode-phpx-0.2.0.vsix
```
2. Build binaries:
```sh
cargo build --release -p cli -p phpx_lsp
```
3. Optional settings (`settings.json`):
```json
{
  "phpx.lsp.path": "/absolute/path/to/deka/target/release/phpx_lsp",
  "phpx.lsp.args": []
}
```
If `phpx.lsp.path` is empty, the extension runs `deka lsp`.

### Neovim
With `nvim-lspconfig`:
```lua
require('lspconfig').phpx_lsp.setup({
  cmd = { '/absolute/path/to/deka/target/release/phpx_lsp' },
  filetypes = { 'phpx' },
  root_dir = require('lspconfig.util').root_pattern('deka.lock', 'php_modules', '.git'),
})
```

### Helix
Configure language server in `languages.toml`:
```toml
[language-server.phpx-lsp]
command = "/absolute/path/to/deka/target/release/phpx_lsp"

[[language]]
name = "phpx"
scope = "source.phpx"
file-types = ["phpx"]
language-servers = ["phpx-lsp"]
```

## Feature Matrix
| Feature | Zed | VS Code | Neovim | Helix |
|---|---|---|---|---|
| Syntax highlighting | Yes | Yes | Via grammar plugin | Via grammar |
| Diagnostics | Yes | Yes | Yes | Yes |
| Hover | Yes | Yes | Yes | Yes |
| Completion | Yes | Yes | Yes | Yes |
| Go to definition | Yes | Yes | Yes | Yes |
| References | Yes | Yes | Yes | Yes |
| Rename | Yes | Yes | Yes | Yes |

## Troubleshooting
- No diagnostics:
  - Rebuild `phpx_lsp`: `cargo build --release -p phpx_lsp`
  - Ensure editor points to the rebuilt binary.
  - Restart editor.
- Unknown language for `.phpx`:
  - Verify extension is installed and active.
  - Confirm file association includes `.phpx`.
- Imports unresolved in LSP:
  - Ensure workspace has `php_modules/`.
  - If using custom root, set `PHPX_MODULE_ROOT`.
- CLI/runtime mismatch:
  - Rebuild CLI: `cargo build --release -p cli`
  - Use `target/release/cli` (wired local `deka`).

## Known Limitations
- Some editor-specific visual behaviors (fold markers, indent behavior) still require manual verification.
- VS Code extension packaging currently includes unbundled `node_modules`.
- Cross-platform release packaging for `phpx_lsp` is not automated yet.
