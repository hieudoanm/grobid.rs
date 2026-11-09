# grobid-cli

Convert PDF articles to Markdown via GROBID TEI XML (PDF → TEI → MD). Runs GROBID in Docker, processes PDFs with the GROBID service, extracts metadata into YAML frontmatter, and renders clean Markdown.

## Quick Start

```bash
# From the repository root
cargo build --release
./target/release/grobid
```

Or use the example script:

```bash
./crates/grobid-cli/tests/e2e.sh
```

## Usage

1. Put PDFs in `pdf/` (relative to working directory or data dir)
2. Run `grobid`
3. Outputs: TEI XML in `tei/` and Markdown in `md/`

By default, data directories are resolved from `GROBID_DATA_DIR` or `ARTICLES_DATA_DIR`, falling back to current working directory. So `./pdf`, `./tei`, `./md` are used if unset.

Options:
```bash
grobid --version
grobid --enrich          # Crossref enrichment (or GROBID_ENRICH=1)

# Corpus commands (read <data-dir>/.grobid/grobid.db, offline)
grobid search <query> [--limit N]
grobid stats
grobid graph             # → .grobid/index.html dashboard
grobid mcp               # MCP server over stdio for AI agents
```

## Environment Variables

| Variable | Default | Description |
| --- | --- | --- |
| `GROBID_DATA_DIR` / `ARTICLES_DATA_DIR` | `.` | Base directory containing `pdf/`, `tei/`, `md/`, `.grobid/` |
| `GROBID_PORT` | `8070` | GROBID service port |
| `GROBID_URL` | — | Remote GROBID base URL (disables Docker auto-start) |
| `GROBID_IMAGE` | `grobid/grobid:0.9.1-crf` | Docker image for GROBID |
| `GROBID_CONTAINER` | `grobid-cli` | Docker container name |
| `GROBID_CONCURRENCY` | `4` | Max concurrent PDF→TEI requests |
| `GROBID_ENRICH` | — | `1` enables Crossref enrichment |

## Features

- **Auto-start GROBID**: Starts a Docker container if GROBID isn't running
- **Resumable**: Skips PDFs/TEIs/MDs that already exist
- **Metadata extraction**: YAML frontmatter with title, authors, year, journal, volume/issue/pages, DOI/DOI link
- **DOI-linked references**: Reference titles with DOIs render as `https://doi.org/…` links
- **Crossref enrichment**: `--enrich`/`GROBID_ENRICH` fills type, cited-by count, and abstract gaps
- **SQLite corpus**: Every work + citation edge indexed into `.grobid/grobid.db`
- **Corpus search & stats**: `grobid search` (unified substring search) and `grobid stats` (counts, year histogram, top authors, in/out-degree rankings)
- **Corpus dashboard**: `grobid graph` writes a self-contained `.grobid/index.html` — d3-force citation graph + stats + year chart + unified filter
- **MCP server**: `grobid mcp` exposes stats/search/works/graph tools to AI agents over stdio
- **Process manifest**: `.grobid/process.json` records every run (per-file status, counts, timing)
- **TEI rendering**: Headings, paragraphs, lists, tables, formulas, figures, references
- **Timestamped logging**: `info/warn/error` with timestamps

## Examples

See [`grobid-core/tests/fixtures/`](grobid-core/tests/fixtures/) for a sample
PDF, TEI XML, and expected Markdown output. Run
[`grobid-cli/tests/e2e.sh`](grobid-cli/tests/e2e.sh) to exercise the full
pipeline against them.

## Requirements

- Docker (for auto-starting GROBID)
- Rust toolchain for building from source

## License

MIT
