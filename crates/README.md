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
```

## Environment Variables

| Variable | Default | Description |
| --- | --- | --- |
| `GROBID_DATA_DIR` / `ARTICLES_DATA_DIR` | `.` | Base directory containing `pdf/`, `tei/`, `md/` |
| `GROBID_PORT` | `8070` | GROBID service port |
| `GROBID_IMAGE` | `grobid/grobid:0.9.1-crf` | Docker image for GROBID |
| `GROBID_CONTAINER` | `grobid-cli` | Docker container name |

## Features

- **Auto-start GROBID**: Starts a Docker container if GROBID isn't running
- **Resumable**: Skips PDFs/TEIs/MDs that already exist
- **Metadata extraction**: YAML frontmatter with title, authors, year, journal, volume/issue/pages, DOI/DOI link
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
