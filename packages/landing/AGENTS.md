# AGENTS.md - @grobid/landing

## Overview

Static marketing/landing page for grobid.rs. Next.js 16 App Router, React 19,
TypeScript strict, Tailwind CSS 4 + daisyUI, exported as static HTML
(`output: 'export'`) for GitHub Pages.

## Structure

```
src/
├── app/                  # layout, page, error/not-found/loading
├── components/           # Navbar, Hero, Features, DemoSection,
│                         # Pipeline, InstallSection, Footer,
│                         # TerminalWindow, ThemeToggle
└── lib/site.ts           # BASE_PATH, assetPath, GITHUB_URL
public/svg/               # logo + og image (from repo-root assets/)
```

## Conventions

- Server components by default; only `ThemeToggle`, `error.tsx`, and
  `global-error.tsx` are client components.
- `assetPath()` must be used for every static asset URL so the
  `/grobid.rs` basePath is applied in production.
- Theme values are `grobid` (light) and `grobid-dark`; storage key
  `grobid-theme`, `data-theme` attribute on `<html>`.
- Keep it a landing page — no API routes, no upload/convert UI (that was
  the old `packages/web` app).

## Commands

| Command         | Purpose                                      |
| --------------- | -------------------------------------------- |
| `pnpm dev`      | Dev server on :3000                          |
| `pnpm build`    | Static export to `out/`                      |
| `pnpm lint`     | ESLint (next + typescript + prettier compat) |
| `pnpm test:run` | Vitest single run (jsdom + RTL)              |
| `pnpm format`   | Prettier write                               |
