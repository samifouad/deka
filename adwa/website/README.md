# Adwa Browser Playground

Run from `adwa/` with Deka as the dev server:

```sh
deka run dev
```

Open:

```text
http://localhost:8530
```

The page is a 3-pane layout:

1. Left: source editor (`main.phpx` input)
2. Right top (70%): rendered result
3. Right bottom (30%): terminal/console log

The browser runtime path is PHPX-first and goes through the adapter boundary:

`main.js` -> `@deka/adwa-js` adapter -> `php_runtime.js` wasm.

Default runtime bootstrap is Linux-like:

- `cwd`: `/home/user`
- dirs: `/bin`, `/usr/bin`, `/home/user`, `/tmp`, `/etc`, `/var/tmp`
- env: `HOME`, `USER`, `PATH`, `TMPDIR`, `PWD`

## Runtime vs editor assets

By default, `scripts/build-demo.sh` builds a runtime-focused bundle and excludes browser LSP wasm assets.

- default: runtime assets only (`vendor/adwa_js`, `vendor/adwa_wasm`)
- optional: include browser LSP/editor wasm assets with:

```sh
ADWA_DEMO_INCLUDE_EDITOR=1 ./scripts/build-demo.sh
```

When editor wasm assets are not present, diagnostics/completion fall back to sidecar mode.

## Config split

This folder has two separate configs:

- `website/deka.json`: static serving for the website shell (`index.html`).
- `website/project/deka.json`: bundled as `/deka.json` for the in-browser PHPX demo runtime.
