# Docs System

This repo ships documentation into the website by combining:
1) hand-written MD/MDX in `docs/php` and `docs/phpx`
2) inline doc comments from source files (any extension)
3) examples pulled from `examples/`

The combined output is written into `deka-website/content/docs/`, then bundled into JSON for the runtime docs UI.

## Doc comment format
- A doc block starts with a `/// docid:` line and continues with `///` lines that follow it.
- The body is treated as Markdown with MDX tags preserved.
- You can place these comment blocks in any file type (Rust, JS, TS, etc.).

Example:
```text
/// docid: php/string/get_str_func()
/// <Function name="get_str_func">
///   <Description>Returns a string representation.</Description>
///   <Parameter name="$value" type="mixed" typeLink="/docs/php/types/mixed">
///     The value to stringify.
///   </Parameter>
///   <ReturnType type="string" typeLink="/docs/php/types/string" />
/// </Function>
```

## Doc IDs and routing
- `docs/docmap.json` maps doc IDs to slugs.
- If a doc ID is not explicitly mapped, it is routed as:
  - `section/category/name()` → `/section/category/name`
  - `section/name()` → uses the default category for that section (from `docmap.json`).
- `name()` loses the trailing parentheses when building the slug.

## Manual docs
- Hand-written pages live in `docs/php` and `docs/phpx` and are copied as-is.
- Frontmatter gets `category`, `categoryLabel`, and `categoryOrder` injected based on `docs/docmap.json`.

## Examples
- Examples are pulled from `examples/` using file-based routing:
  - `examples/php/string/get_str_func.example.php`
  - `examples/js/runtime/serve.example.ts`
- If multiple `*.example.*` files exist for the same doc, they are appended in filename order.

## Publish pipeline
From this repo:
```bash
node scripts/publish-docs.js --manual docs/php,docs/phpx --scan . --out ../deka-website/content/docs --force
```

Contributor test workflow (auto-publishes docs at end of PHPX test scripts):
```bash
scripts/test-runtime-dev.sh
```

Options:
- `DEKA_TEST_SKIP_DOCS=1` to skip docs publish in contributor test scripts
- `DEKA_DOCS_OUT=/custom/path` to override docs output
- `DEKA_RUN_DB_E2E=1 scripts/test-runtime-dev.sh` to include DB E2E checks

From `deka-website/`:
```bash
bun run bundle:runtime
```

## Translation system (i18n)
Translations are generated at build time in `deka-website/` using OpenAI:
- Source docs live in `content/docs`, `content/cli`, and `content/api`.
- Translated copies are written to `content-i18n/<lang>/{docs,cli,api}`.
- Bundled JSON is created per language:
  - `lib/bundled-runtime.<lang>.json`
  - `lib/bundled-cli.<lang>.json`
  - `lib/bundled-api.<lang>.json`

Key scripts:
- `scripts/translate-docs.ts` → translates MD/MDX into `content-i18n`.
- `scripts/build-i18n.ts` → runs translation + bundling for all languages.
- `bundle:i18n` (package script) → wraps `build-i18n.ts`.

Requirements:
- `.env` must include `OPENAI_API_KEY`.
- Run `bun run bundle:i18n` in `deka-website/` to generate translated bundles.

Runtime behavior:
- The server reads `deka-language` from cookies.
- Localized bundles are loaded and merged over English (English is the fallback).
- If a translation is missing, the English content is shown for that field.

Troubleshooting (when translations appear in English):
- Verify `content-i18n/<lang>/docs` exists.
- Verify `lib/bundled-runtime.<lang>.json` exists and is up to date.
- Re-run `bun run bundle:i18n` after updating source content.
- If switching languages does nothing, the cookie may not be set; check for `deka-language`.

## MDX function blocks
The runtime bundler (`deka-website/scripts/bundle-runtime.ts`) converts these MDX tags into a structured function block:
- `<Function>`
- `<Description>`
- `<Parameter>`
- `<ReturnType>` (required)

The output order is: Signature → Description → Parameters → Return type.

## PHPX internals audit
The internal bridge inventory and migration checklist lives in `PHPX_INTERNALS.md`.
The goal is to keep only low-level hooks in Rust and move algorithmic helpers into phpx.
