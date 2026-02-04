# phpx_lsp

Minimal PHPX language server (LSP) implementation.

## Build

```sh
cargo build -p phpx_lsp
```

## Run (stdio)

```sh
cargo run -p phpx_lsp
```

The server reads LSP messages from stdin and writes responses to stdout.

## Zed setup

1) Build the LSP:

```sh
cargo build --release -p phpx_lsp
```

2) Add a language server entry in your Zed settings:

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

3) Ensure `extensions/phpx/extension.toml` is installed in Zed.
