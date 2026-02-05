# PHPX VS Code Extension (Local)

This provides basic PHPX syntax highlighting via a TextMate grammar.

## Local install

Option A (development host):

1. Open `extensions/vscode-phpx` in VS Code.
2. Press `F5` to run the Extension Development Host.
3. Open a `.phpx` file to verify highlighting.

Option B (vsix):

1. Install vsce: `npm install -g @vscode/vsce`
2. From `extensions/vscode-phpx/`, run: `vsce package`
3. Install the generated `.vsix` via `code --install-extension`.

## LSP

This extension only provides highlighting. Once `phpx_lsp` is packaged we can
add the language client to provide diagnostics and navigation.
