# tree-sitter-phpx

Tree-sitter grammar for PHPX (and PHP).

This grammar ships two language entries:
- `phpx` (used for `.php` and related PHP extensions)
- `phpx_only` (used for `.phpx`)

The PHPX grammar adds:
- Type annotations and type aliases
- `import` / `export` module syntax
- `struct` literals
- JSX (`<Component />`)
- Frontmatter templates (`---` ... `---`)

## Installation

### Local development

```sh
npm install
```

Generate parsers after editing the grammar:

```sh
cd tooling/tree-sitter-phpx/php

tree-sitter generate

cd ../php_only

tree-sitter generate
```

## Testing

Run the full corpus and highlight tests:

```sh
cd tooling/tree-sitter-phpx

tree-sitter test
```

Update a single corpus file:

```sh
tree-sitter test -u --file-name phpx_jsx.txt
```

## Editor integration

### Zed

A scaffold extension lives in `extensions/phpx/`.

Symlink it into Zed (Linux default config path):

```sh
ln -s /path/to/deka/extensions/phpx ~/.config/zed/extensions/work/phpx
```

On macOS, Zed uses a different config path:

```sh
ln -s /path/to/deka/extensions/phpx "$HOME/Library/Application Support/Zed/extensions/work/phpx"
```

Or run the helper script from the repo root:

```sh
scripts/install-zed-extension.sh
```

Configure the PHPX language server in Zed settings (example uses a local
release build):

```json
{
  "language_servers": {
    "phpx-lsp": {
      "command": "/path/to/deka/target/release/phpx_lsp",
      "args": []
    }
  },
  "languages": {
    "PHPX": {
      "language_servers": ["phpx-lsp"]
    }
  }
}
```

### Neovim (nvim-treesitter)

Register the grammar in your `nvim-treesitter` config:

```lua
local parser_config = require("nvim-treesitter.parsers").get_parser_configs()
parser_config.phpx = {
  install_info = {
    url = "https://github.com/samifouad/deka",
    files = {
      "tooling/tree-sitter-phpx/php/src/parser.c",
      "tooling/tree-sitter-phpx/php/src/scanner.c",
    },
  },
  filetype = "phpx",
}
```

### Helix

Add a language entry and tree-sitter grammar in `languages.toml`:

```toml
[[language]]
name = "phpx"
scope = "source.phpx"
injection-regex = "phpx"
file-types = ["phpx"]
comment-token = "//"
grammar = "phpx_only"
```

## Known limitations

- Frontmatter templates are parsed as JSX, not a full HTML grammar.
- `<!doctype ...>` and HTML comments are supported, but full HTML parsing is not.
- JSX expressions require spaces around `<`/`>` comparisons to avoid ambiguity.
- Statements are not allowed inside JSX expressions (use expressions instead).

## Contributing

- Update `common/define-grammar.js` and regenerate both parsers (`php/` and `php_only/`).
- Add/adjust corpus tests in `test/corpus/`.
- Run `tree-sitter test` before committing.
