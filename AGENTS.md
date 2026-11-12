# AGENTS.md

## Project Overview

grobid.rs converts PDF articles to Markdown via GROBID TEI XML, making PDFs
AI-friendly. It is a pnpm/turbo monorepo plus a root-level Rust workspace:

- **`crates/`** — Rust workspace (`grobid`): converts all PDFs in `pdf/` to TEI
  XML in `tei/` and Markdown in `md/`, auto-starting GROBID in Docker.
  - `grobid-core` — GROBID HTTP client, local JVM/Docker backend management, TEI parsing,
    metadata extraction, Markdown rendering
  - `grobid-cli` — `grobid` binary (clap + tokio) and batch orchestration
- **`packages/landing`** — plain static landing page (`index.html` +
  `styles.css`, no build step) deployed to GitHub Pages; markets the CLI
  only, no app functionality.

Pipeline: `pdf/` → GROBID `processFulltextDocument` → TEI XML → Markdown with
YAML frontmatter (title, authors, year, journal, volume/issue/pages, DOI,
optional Crossref `type`/`cited_by`). Each converted work is also indexed
into `<data-dir>/grobid.db` (SQLite `works` + `refs` citation edges) with a
JSON export at `index.json`, per-work files under `metadata/`, and a run
manifest at `process.json` — all inside `<data-dir>/.grobid/` next to
`pdf/`/`tei/`/`md/`; `grobid search|stats|graph|mcp` query that corpus
offline, `grobid graph` emits a self-contained dashboard
`.grobid/index.html` (d3-force graph + stats sidebar + year chart +
unified filter), and `grobid mcp` serves the corpus to AI agents over
stdio MCP.

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
│   │   ├── grobid_client.rs# local JVM/Docker backend management + PDF→TEI HTTP calls
│   │   ├── metadata.rs     # TEI → Metadata (frontmatter fields)
│   │   ├── render.rs       # TEI → Markdown + ParsedReference extraction
│   │   ├── corpus.rs       # SQLite works/refs index, search, stats, graph HTML
│   │   ├── enrich.rs       # optional Crossref metadata enrichment
│   │   ├── paths.rs        # pdf/tei/md dirs + .grobid/ artifact paths
│   │   ├── utils.rs        # logging, yaml_string, text helpers
│   │   ├── version.rs      # version from binary mtime
│   │   └── templates/graph.html  # Obsidian-style dashboard page (include_str!)
│   └── tests/fixtures/ # sample pdf/tei/md fixture data + committed .grobid/ examples
└── grobid-cli/        # grobid binary (clap + tokio)
    ├── src/mcp.rs     # stdio MCP server: stats/search/works/graph tools
    └── tests/e2e.sh   # end-to-end run against the fixtures
packages/
└── landing/              # flat static site deployed to GitHub Pages:
                          # index.html, 404.html, styles.css, logo.svg,
                          # og-image.png, favicon.ico, robots/sitemap/humans.txt
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

| Variable                                | Default                   | Used by                                                                                                                                                                      |
| --------------------------------------- | ------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `GROBID_DATA_DIR` / `ARTICLES_DATA_DIR` | cwd                       | CLI: base dir for `pdf/`, `tei/`, `md/`                                                                                                                                      |
| `GROBID_PORT`                           | `8070`                    | CLI + compose                                                                                                                                                                |
| `GROBID_URL`                            | —                         | CLI: remote GROBID base URL (disables local backend management)                                                                                                              |
| `GROBID_LOCAL_BIN`                      | —                         | CLI: path to local `grobid-service` launcher (default `~/.local/share/grobid.rs/grobid/grobid-service/bin/grobid-service` if present); spawned on demand and stopped on exit |
| `GROBID_LOCAL_KEEP`                     | —                         | CLI: `1` leaves the spawned local service running after exit                                                                                                                 |
| `GROBID_IMAGE`                          | `grobid/grobid:0.9.1-crf` | CLI Docker auto-start (fallback when no local service)                                                                                                                       |
| `GROBID_CONTAINER`                      | `grobid-cli`              | CLI Docker auto-start                                                                                                                                                        |
| `GROBID_CONCURRENCY`                    | `4`                       | CLI: max concurrent PDF→TEI requests                                                                                                                                         |
| `GROBID_ENRICH`                         | —                         | CLI: `1` enables Crossref enrichment (same as `--enrich`)                                                                                                                    |
| `GROBID_LOG`                            | `info`                    | CLI Logger level (`fatal`..`trace`); `-v`/`-vv` flags override                                                                                                               |
| `RUST_LOG`                              | info                      | CLI tracing filter                                                                                                                                                           |

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
- **Supply chain**: `pnpm-workspace.yaml` sets `minimumReleaseAge: 0`;
  dependency build scripts gated by `allowBuilds`.

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
