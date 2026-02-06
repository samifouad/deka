# phpx_lsp

PHPX language server (LSP) binary used by editor integrations.

## Build

```sh
cargo build -p phpx_lsp
```

Release build:

```sh
cargo build --release -p phpx_lsp
```

## Run (stdio)

```sh
cargo run -p phpx_lsp
```

The server reads JSON-RPC/LSP from stdin and writes responses to stdout.

## Editor Integration

### Zed

1. Install the PHPX Zed extension from `extensions/phpx`.
2. Point Zed at the release binary:

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

3. Ensure `extensions/phpx/extension.toml` includes the `phpx-lsp` server binding.

### VS Code

1. Install `extensions/vscode-phpx`.
2. Configure binary path in VS Code settings:

```json
{
  "phpx.lsp.path": "/absolute/path/to/deka/target/release/phpx_lsp"
}
```

3. Reload window after binary updates.

### Neovim (`nvim-lspconfig`)

```lua
require('lspconfig').phpx_lsp.setup({
  cmd = { "/absolute/path/to/deka/target/release/phpx_lsp" },
  filetypes = { "phpx" },
  root_dir = require('lspconfig.util').root_pattern("deka.lock", "php_modules", ".git"),
})
```

Add filetype detection for `.phpx` if your setup does not already provide it.

## Configuration Options

- `PHPX_MODULE_ROOT`: optional module root override for import resolution.
  - If set, `phpx_lsp` resolves modules from `$PHPX_MODULE_ROOT/php_modules`.
- Workspace root discovery:
  - Uses `workspaceFolders` or `rootUri` from client initialize request.
  - Falls back to current working directory when not provided.

## Debugging Tips

- Check binary wiring:
  - `which phpx_lsp`
  - `/absolute/path/to/deka/target/release/phpx_lsp --help` (if supported by launcher)
- Run manually over stdio to validate startup:
  - `cargo run -p phpx_lsp`
- Rebuild after parser/typechecker changes:
  - `cargo build --release -p phpx_lsp`
- For editor-side issues:
  - Confirm filetype is `phpx`.
  - Confirm extension language id matches the LSP registration.
  - Open editor logs and verify `phpx_lsp` process is launched.
