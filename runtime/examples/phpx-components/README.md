# PHPX Components Demo (client-side navigation)

This example shows JSX rendering + client-side navigation (replace mode).

## Run
```
deka serve examples/phpx-components/app.phpx
```

Then open `http://localhost:8530`.

## Notes
- Uses frontmatter template mode (Astro-style) in `app.phpx`.
- Uses `component/dom` for server rendering + `<Link>` helpers.
- Hydration is provided via `<Hydration />` (inline client runtime).
- Client-side navigation is driven by the `<Hydration />` component.
- Demo includes a frontmatter component module: `php_modules/ui/card.phpx`.
- Omit `<Hydration />` for a fully static site (full page loads only).
- Partial requests are returned as JSON when `?partial=1` or
  `Accept: text/x-phpx-fragment` is present.
- Layout-aware nav is enabled: `#app` has `data-layout="users"` and
  the client only swaps when layouts match.
