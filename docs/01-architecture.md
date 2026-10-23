# Architecture

grobid.rs is a pnpm/Turborepo monorepo with a root-level Rust workspace. The
Rust CLI does all conversion work; the landing page is a static marketing site
with no runtime role in the pipeline.

## Repository layers

### Rust workspace (`crates/`)

The Cargo workspace manifest lives at the repository root (`Cargo.toml`),
with `Cargo.lock`, `rust-toolchain.toml`, and a root `Makefile`. Crate
directories are named after their packages.

- `grobid-core` is the library crate. It owns:
  - `config.rs` — environment-driven configuration (port, image, container,
    data dir) and constants (TEI namespace, metadata fields, service name)
  - `grobid_client.rs` — GROBID health checks, Docker container lifecycle, and
    the `processFulltextDocument` HTTP call (PDF → TEI)
  - `metadata.rs` — TEI `teiHeader` traversal producing `Metadata` and YAML
    frontmatter
  - `render.rs` — TEI → Markdown rendering (inline markup, blocks, lists,
    tables, figures, references) plus unit tests
  - `paths.rs` — `pdf/`/`tei/`/`md/` directory resolution and file-name mapping
  - `utils.rs` — timestamped logging and YAML/text helpers
  - `version.rs` — build timestamp used by `grobid --version`
- `grobid-cli` is the binary crate. `main.rs` parses args with clap and calls
  `cli::convert_pdf_to_md()`, which orchestrates the batch pipeline and maps
  per-file failures to a non-zero exit code.

### Node workspace (`packages/`)

- `packages/landing` is a dependency-free static site (`index.html` +
  `styles.css`) deployed to GitHub Pages as-is. It documents the CLI only —
  no build step, no server logic.
- `crates/package.json` (`@grobid/cli`) is a thin shim so `pnpm lint`, `pnpm
  test`, and `pnpm build` at the root still drive the Rust workspace through
  the root `Makefile`.

## Conversion pipeline

```text
pdf/*.pdf
   |
   v  grobid_client::pdf_to_tei  (POST /api/processFulltextDocument)
GROBID service (Docker, :8070)
   |
   v  TEI XML
tei/*.tei.xml          (skipped if already present)
   |
   v  metadata::extract_metadata_from_tei  +  render::convert_tei_to_md
md/*.md                (skipped if already present)
   |
   +--> YAML frontmatter: title, authors, year, journal,
        volume, issue, pages, doi, doi_link
```

Each PDF is processed independently: TEI conversion failures and Markdown
render/write failures are counted, logged, and skipped — the run continues and
exits `1` if any file failed.

## Data flow notes

- **Resumable**: existing `tei/*.tei.xml` and `md/*.md` are never regenerated;
  adding PDFs to `pdf/` and re-running only processes the new files.
- **Single parse**: `convert_tei_xml` parses the TEI document once and shares
  the parsed tree between metadata extraction and Markdown rendering.
- **Namespace-agnostic**: TEI elements are matched by local name via
  `descendant_named`/`descendants_named`/`tei_root` helpers in `render.rs`, so
  documents with or without the TEI namespace both parse.
- **Header scoping**: metadata lookups (authors, DOI, journal, date) are
  restricted to `teiHeader` so reference-list authors cannot leak into
  frontmatter.

## Landing page

`packages/landing` uses relative asset paths, so the same files work locally
and under the `/grobid.rs/` Pages base path; the page is light-themed with
dark terminal mockups. CI uploads the directory to GitHub Pages on pushes to
`master` — there is no build step.
