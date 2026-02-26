This is a [Next.js](https://nextjs.org) project bootstrapped with [`create-next-app`](https://nextjs.org/docs/app/api-reference/cli/create-next-app).

## Getting Started

First, run the development server:

```bash
npm run dev
# or
yarn dev
# or
pnpm dev
# or
bun dev
```

Open [http://localhost:3000](http://localhost:3000) with your browser to see the result.

You can start editing the page by modifying `app/page.tsx`. The page auto-updates as you edit the file.

This project uses [`next/font`](https://nextjs.org/docs/app/building-your-application/optimizing/fonts) to automatically optimize and load [Geist](https://vercel.com/font), a new font family for Vercel.

## Build & Translation Pipeline

This repo bundles markdown/MDX docs into JSON for runtime use and generates localized versions during build.

### High-level flow

1) `bundle:i18n` (translation + localized bundles)
   - Reads `.env` and requires `OPENAI_API_KEY`.
   - Iterates all languages in `i18n/i18n.ts` (except `en`).
   - For each language:
     - Translates:
       - `content/docs` -> `content-i18n/<lang>/docs`
       - `content/cli` -> `content-i18n/<lang>/cli`
       - `content/api` -> `content-i18n/<lang>/api`
     - Bundles localized output:
       - `lib/bundled-runtime.<lang>.json`
       - `lib/bundled-cli.<lang>.json`
       - `lib/bundled-api.<lang>.json`

2) `bundle:all` (English bundles)
   - Generates the default English bundles:
     - `lib/bundled-runtime.json`
     - `lib/bundled-cli.json`
     - `lib/bundled-api.json`
     - `lib/bundled-docs.json`
     - `lib/bundled-examples.json`

3) `next build`
   - Pages load the appropriate bundle based on the `deka-language` cookie.
   - Localized bundles merge over English so missing translations fall back to English.

### Commands

```bash
bun run bundle:i18n
bun run bundle:all
bun run build
```

### Notes for future agents

- Translation is driven by `scripts/build-i18n.ts` + `scripts/translate-docs.ts`.
- Bundlers are in `scripts/bundle-runtime.ts`, `scripts/bundle-cli.ts`, `scripts/bundle-api.ts`.
- Language selection comes from the `deka-language` cookie and `lib/i18n-server.ts`.
- If you add a new language, update `i18n/i18n.ts` and re-run `bun run bundle:i18n`.

## Learn More

To learn more about Next.js, take a look at the following resources:

- [Next.js Documentation](https://nextjs.org/docs) - learn about Next.js features and API.
- [Learn Next.js](https://nextjs.org/learn) - an interactive Next.js tutorial.

You can check out [the Next.js GitHub repository](https://github.com/vercel/next.js) - your feedback and contributions are welcome!

## Deploy on Vercel

The easiest way to deploy your Next.js app is to use the [Vercel Platform](https://vercel.com/new?utm_medium=default-template&filter=next.js&utm_source=create-next-app&utm_campaign=create-next-app-readme) from the creators of Next.js.

Check out our [Next.js deployment documentation](https://nextjs.org/docs/app/building-your-application/deploying) for more details.
