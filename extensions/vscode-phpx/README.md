# PHPX VS Code Extension

This extension provides:

- PHPX syntax highlighting (`.phpx`)
- PHPX language server integration (`phpx_lsp`) for diagnostics, go-to-definition, references, rename, and more

## Install

Use the helper script from repo root:

```sh
scripts/install-vscode-extension.sh
```

## LSP startup behavior

Resolution order:
1. `phpx.lsp.path` (explicit setting)
2. Bundled binary in extension `bin/` (if present)
3. `deka lsp` from PATH
4. `phpx_lsp` from PATH

Override in VS Code settings if needed:

```json
{
  "phpx.lsp.path": "/absolute/path/to/phpx_lsp",
  "phpx.lsp.args": []
}
```

## Commands

- `PHPX: Restart Language Server`

## Troubleshooting

- Open `Output` panel and select `PHPX` to inspect server launch logs.
- If VS Code PATH differs from shell, set:
  - `"phpx.lsp.path": "/Users/<you>/Projects/deka/deka/target/release/phpx_lsp"`
- After rebuilding LSP, run `PHPX: Restart Language Server`.

## Development

1. Open `extensions/vscode-phpx` in VS Code
2. Run `npm install`
3. Press `F5` to launch Extension Development Host
4. Open a `.phpx` file
