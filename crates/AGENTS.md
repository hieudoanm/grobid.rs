# AGENTS.md

## Overview

Rust workspace to convert PDFs to Markdown via GROBID TEI XML. The workspace
manifest lives at the repository root (`Cargo.toml`); crates live in `crates/`.

## Structure

- `grobid-core/src/config.rs` - Configuration and constants
- `grobid-core/src/grobid_client.rs` - GROBID HTTP client + Docker container management
- `grobid-core/src/metadata.rs` - Metadata extraction and YAML frontmatter
- `grobid-core/src/render.rs` - TEI XML to Markdown rendering + reference parsing
- `grobid-core/src/corpus.rs` - SQLite corpus index (works + citation edges), search/stats/graph/dashboard HTML
- `grobid-core/src/enrich.rs` - optional Crossref metadata enrichment
- `grobid-core/src/templates/graph.html` - Obsidian-style dashboard (d3-force graph + stats + year chart + unified filter)
- `grobid-core/src/paths.rs` - Path resolution for pdf/tei/md + `.grobid/` artifacts
- `grobid-core/src/utils.rs` - Logging and helpers
- `grobid-core/src/version.rs` - Version from file mtime
- `grobid-core/tests/fixtures/` - Sample pdf/tei/md fixture data + committed `.grobid/` examples
- `grobid-cli/src/main.rs` / `cli.rs` - `grobid` binary entry point (clap + tokio)
- `grobid-cli/src/mcp.rs` - stdio MCP server (`grobid mcp`): `grobid_stats`, `grobid_search`, `grobid_works`, `grobid_graph` tools (rmcp)
- `grobid-cli/tests/e2e.sh` - End-to-end script against the fixtures

## Build

From the repository root:

```bash
cargo build --release   # binary at target/release/grobid
```

## Test

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

## Environment

- `GROBID_DATA_DIR` or `ARTICLES_DATA_DIR`: base directory for pdf/tei/md + `.grobid/` corpus artifacts
- `GROBID_PORT`: GROBID port (default 8070)
- `GROBID_IMAGE`: Docker image (default grobid/grobid:0.9.1-crf)
- `GROBID_CONTAINER`: Container name (default grobid-cli)
- `GROBID_ENRICH`: `1` enables Crossref enrichment (same as `--enrich`)
