# AGENTS.md

## Project Overview

grobid.rs converts PDF articles to Markdown via GROBID TEI XML, making PDFs
AI-friendly. It is a pnpm/turbo monorepo plus a root-level Rust workspace:

- **`crates/`** — Rust workspace (`grobid`): converts all PDFs in `pdf/` to TEI
  XML in `tei/` and Markdown in `md/`, auto-starting GROBID in Docker.
  - `grobid-core` — GROBID HTTP client, Docker management, TEI parsing,
    metadata extraction, Markdown rendering
  - `grobid-cli` — `grobid` binary (clap + tokio) and batch orchestration
- **`packages/landing`** — plain static landing page (`index.html` +
  `styles.css`, no build step) deployed to GitHub Pages; markets the CLI
  only, no app functionality.

Pipeline: `pdf/` → GROBID `processFulltextDocument` → TEI XML → Markdown with
YAML frontmatter (title, authors, year, journal, volume/issue/pages, DOI).

## Repository Layout

```
Cargo.toml             # Rust workspace root (members: crates/grobid-*)
Cargo.lock
rust-toolchain.toml    # stable + rustfmt/clippy
Makefile               # cargo build/test/lint/fmt/clean at workspace root
crates/
├── package.json       # @grobid/cli turbo shim → make -C ..
├── grobid-core/       # grobid-core library
│   ├── src/
│   │   ├── config.rs       # env config, ports, image, META_FIELDS
│   │   ├── grobid_client.rs# Docker management + PDF→TEI HTTP calls
│   │   ├── metadata.rs     # TEI → Metadata (frontmatter fields)
│   │   ├── render.rs       # TEI → Markdown rendering + unit tests
│   │   ├── paths.rs        # pdf/tei/md dir resolution
│   │   ├── utils.rs        # logging, yaml_string, text helpers
│   │   └── version.rs      # version from binary mtime
│   └── tests/fixtures/ # sample pdf/tei/md fixture data
└── grobid-cli/        # grobid binary (clap + tokio)
    └── tests/e2e.sh   # end-to-end run against the fixtures
packages/
└── landing/              # static site: index.html + styles.css + svg/
    └── src/
        ├── app/               # layout, page, error/not-found/loading
        ├── components/        # Navbar, Hero, Features, DemoSection,
        │                      # Pipeline, InstallSection, Footer, etc.
        └── lib/site.ts        # BASE_PATH, assetPath, GITHUB_URL
```

## Commands

```bash
# Root (turbo across packages)
pnpm dev | lint | test | build | format

# CLI (Rust workspace at repo root)
cargo build --release        # binary at target/release/grobid
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
make test lint fmt build     # or via turbo: pnpm test / pnpm lint / pnpm build

# Landing (plain static files — no build)
# preview: python3 -m http.server 8000 -d packages/landing
```

Git hooks: `.husky/pre-commit` runs `pnpm turbo run format` + `lint`
(`pre-commit install` to enable).

## Environment Variables

| Variable               | Default                  | Used by |
| ---------------------- | ------------------------ | ------- |
| `GROBID_DATA_DIR` / `ARTICLES_DATA_DIR` | cwd | CLI: base dir for `pdf/`, `tei/`, `md/` |
| `GROBID_PORT`          | `8070`                   | CLI + compose |
| `GROBID_URL`           | —                        | CLI: remote GROBID base URL (disables Docker auto-start) |
| `GROBID_IMAGE`         | `grobid/grobid:0.9.1-crf`| CLI Docker auto-start |
| `GROBID_CONTAINER`     | `grobid-cli`             | CLI Docker auto-start |
| `GROBID_CONCURRENCY`   | `4`                      | CLI: max concurrent PDF→TEI requests |
| `GROBID_LOG`           | `info`                   | CLI Logger level (`fatal`..`trace`); `-v`/`-vv` flags override |
| `RUST_LOG`             | info                     | CLI tracing filter |
| `GROBID_CONCURRENCY`   | `4`                      | CLI: concurrent PDF→TEI requests |

## Conventions

- **Rust**: edition 2021, exact-pinned deps (`=`) managed by renovate,
  `anyhow` for errors, `tracing` + custom timestamped `Logger`, clippy
  `-D warnings`, `cargo fmt`.
- **Landing**: dependency-free `index.html` + `styles.css`; relative asset
  paths so it works under the `/grobid.rs/` Pages base path; light theme
  with dark terminal mockups; follow the anti-slop design rules (no filler
  badges, monospace only for code, one teal accent).
- **Versions**: pnpm pinned via `packageManager` in root `package.json`
  (corepack); Node `26.13.0`; Rust deps pinned exactly in the root
  `Cargo.toml` workspace.
- **Supply chain**: `pnpm-workspace.yaml` sets `minimumReleaseAge: 1440`
  (1 day — longer windows break installs since renovate pins fresh
  versions); dependency build scripts gated by `allowBuilds`.

## Docker & Release

- `docker-compose.yaml` (repo root): `grobid` service only (the landing
  page is static and needs no container).
- CI (`.github/workflows/ci.yaml`): CLI lint/test/build on ubuntu; landing
  is deployed to GitHub Pages directly from `packages/landing` on master (requires Pages source = "GitHub Actions"), then a
  `latest` GitHub Release with the Linux binary (`scripts/install.sh`
  downloads it to `~/.local/bin`; `scripts/install-local.sh` builds from
  source).

## Gotchas

- The TEI parser matches elements by **local name** (namespace-agnostic);
  `render.rs` helpers `descendant_named`/`descendants_named`/`tei_root` are
  the canonical way to query the tree — do not reintroduce xpath-string
  matching.
- Metadata author/DOI/journal lookups are scoped to `teiHeader` on purpose —
  document-wide search picks up reference-list authors.
- Resumable pipeline: existing `tei/*.tei.xml` and `md/*.md` are skipped;
  the CLI exits non-zero if any file failed.
- The landing page must stay static — no API routes or server logic.
