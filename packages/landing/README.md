# @grobid/landing

Static landing page for grobid.rs — a Next.js 16 app exported to plain HTML
(`output: 'export'`) and deployed to GitHub Pages.

## Commands

```bash
pnpm dev            # next dev on :3000
pnpm build          # static export to out/
pnpm start          # serve the production build
pnpm lint           # eslint
pnpm typecheck      # tsc --noEmit (run via repo root turbo)
pnpm test:run       # vitest run
pnpm format         # prettier --write
```

## Notes

- `basePath` is `/grobid.rs` in production builds (GitHub Pages project
  site); empty in development.
- Theme is daisyUI `grobid` / `grobid-dark`, persisted in
  `localStorage['grobid-theme']`.
- Brand assets live in `public/svg/` (sourced from the repo-root `assets/`).
