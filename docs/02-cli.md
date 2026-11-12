# CLI reference

The `grobid` binary converts every PDF in the data directory's `pdf/` folder
to TEI XML (`tei/`) and Markdown (`md/`), indexes the corpus into a SQLite
database, and can search, report stats on, or visualize that corpus.

## Usage

```bash
grobid [--enrich] [-v] [--version]
grobid graph [--db PATH] [--out PATH]
grobid search <query> [--limit N] [--db PATH]
grobid stats [--db PATH]
grobid mcp [--db PATH]
```

### Convert (default, no subcommand)

| Flag        | Short | Description                                                        |
| ----------- | ----- | ------------------------------------------------------------------ |
| `--version` | `-V`  | Print the version and exit.                                        |
| `--verbose` | `-v`  | Increase log verbosity; repeat for more (`-vv` = trace).          |
| `--enrich`  |       | Enrich metadata via the Crossref API (see below).                  |

### `graph`

Regenerates the corpus dashboard at
`<data-dir>/.grobid/index.html` (conversion already writes it on every
run; use this to rebuild without re-converting). The page is
Obsidian-style, with a
`d3-force` citation graph (pan/zoom, drag, per-node metadata panel, nodes
colored by work type) plus a stats sidebar: corpus counts, a year
distribution chart, and top in-degree/out-degree/author rankings. One
search box filters the graph, the rankings, and the chart together;
untitled stub works are excluded from the graph. Large corpora render
the top 400 nodes by in-degree first, with a "show all" toggle and a
minimum-degree filter.

### `search <query>`

Case-insensitive substring search across title, authors, journal, DOI,
abstract, and type — the same unified-filter semantics as a corpus search
box. `--limit` caps results (default 20).

### `stats`

Prints corpus statistics: work/reference counts, unresolved and
self-citation counts, distinct authors, year histogram, top individual
authors, most-cited works (Crossref `cited_by`), and corpus-derived
in-degree (most cited within corpus) / out-degree (most citing)
rankings.

### `mcp`

Serves the corpus over the
[Model Context Protocol](https://modelcontextprotocol.io) on stdio, so AI
agents (Claude Desktop, Devin, etc.) can query it directly. Tools:

| Tool             | Description                                             |
| ---------------- | ------------------------------------------------------- |
| `grobid_stats`   | Corpus statistics as JSON                               |
| `grobid_search`  | Substring search (`query`, optional `limit`)            |
| `grobid_works`   | List works (`limit`, default 50)                        |
| `grobid_graph`   | Regenerate the dashboard HTML (`out` path optional)     |

Example client configuration (`.mcp.json`):

```json
{
  "mcpServers": {
    "grobid": {
      "command": "grobid",
      "args": ["mcp"],
      "env": { "GROBID_DATA_DIR": "/path/to/data" }
    }
  }
}
```

Like the other corpus commands, `mcp` reads
`<data-dir>/.grobid/grobid.db` and never contacts GROBID or the network.

## Environment variables

| Variable               | Default                  | Description                                            |
| ---------------------- | ------------------------ | ------------------------------------------------------ |
| `GROBID_DATA_DIR`      | `ARTICLES_DATA_DIR`, then cwd | Base directory containing `pdf/`, `tei/`, `md/`  |
| `ARTICLES_DATA_DIR`    | —                        | Legacy alias for `GROBID_DATA_DIR`                     |
| `GROBID_URL`           | —                        | Full base URL of a GROBID instance (e.g. `http://grobid:8070`); when set, the CLI never manages a local backend and `GROBID_PORT` is ignored |
| `GROBID_PORT`          | `8070`                   | Port the GROBID service listens on                     |
| `GROBID_LOCAL_BIN`     | `~/.local/share/grobid.rs/grobid/grobid-service/bin/grobid-service` | Path to a locally installed `grobid-service` launcher; when present the CLI spawns it instead of Docker and stops it on exit. Install with `scripts/setup-grobid-local.sh` (needs OpenJDK 21+, no Docker) |
| `GROBID_LOCAL_KEEP`    | —                        | `1` keeps the spawned local service running after the CLI exits |
| `GROBID_IMAGE`         | `grobid/grobid:0.9.1-crf`| Docker image used when auto-starting GROBID (fallback when no local service) |
| `GROBID_CONTAINER`     | `grobid-cli`             | Docker container name used when auto-starting GROBID   |
| `GROBID_CONCURRENCY`   | `4`                      | Max concurrent PDF→TEI requests                        |
| `GROBID_ENRICH`        | —                        | `1` enables Crossref enrichment (same as `--enrich`)   |
| `GROBID_LOG`           | `info`                   | Logger level: `fatal`, `error`, `warn`, `info`, `debug`, `trace` |
| `RUST_LOG`             | `info`                   | `tracing` filter for CLI/GROBID-client logs            |

`GROBID_DATA_DIR` takes precedence over `ARTICLES_DATA_DIR`. When neither is
set, the current working directory is used, so `./pdf`, `./tei`, and `./md`
are expected relative to where you run the command.

## Behavior

### Convert

1. `tei/` and `md/` are created under the data dir (`pdf/`, `tei/`, `md/`
   paths resolve lazily from the environment on first use).
2. If `pdf/` is empty, the run logs a warning and exits `0`.
3. GROBID is started if needed (see [03-grobid-service.md](./03-grobid-service.md)) —
   but only when at least one PDF still lacks its `.tei.xml`; fully cached
   runs never contact the server. If it cannot be reached when needed, the
   run exits `1`.
4. For each PDF (processed with up to `GROBID_CONCURRENCY` parallel requests):
   - If `tei/<name>.tei.xml` exists it is reused; otherwise the PDF is POSTed
     to GROBID (`processFulltextDocument`, `consolidateHeader=1`), with one
     retry on server errors or empty responses.
   - If `md/<name>.md` exists it is skipped; otherwise the TEI is rendered to
     Markdown and written. Reference entries that carry a DOI render their
     title as a `https://doi.org/…` link.
   - With `--enrich`/`GROBID_ENRICH`, works with a DOI are additionally
     resolved against `api.crossref.org` while their Markdown is (re)written.
     Crossref fills `type` and `cited_by` (emitted in frontmatter), plus
     abstract/year/journal fallbacks for the corpus index. Cached Markdown
     files are not re-enriched — delete `md/<name>.md` to refresh.
5. Every processed work is upserted into `<data-dir>/.grobid/grobid.db`
   (SQLite `works` + `refs` tables — work metadata and citation edges,
   including DOIs of referenced works that are not in the corpus), and
   `<data-dir>/.grobid/index.json` is exported as a portable search index. Each
   work's full record — all frontmatter fields plus the parsed reference
   list — is also written as `<data-dir>/.grobid/metadata/<pdf-stem>.json`.
   Each run is also appended to `<data-dir>/.grobid/process.json` (a
   manifest of the last 50 runs: per-file `converted`/`cached`/`failed`
   statuses, counts, duration, concurrency, enrich flag, and GROBID URL).
   The run also regenerates `<data-dir>/.grobid/index.html`, the corpus
   dashboard described under [`graph`](#graph) below.
6. Exit code is `0` when every file succeeded (or there was nothing to do)
   and `1` if GROBID was unavailable or any file failed. Corpus indexing
   failures are warnings, not fatal.

The HTTP client used for GROBID calls has a 300-second timeout; Crossref
calls use a 15-second timeout.

### Corpus commands

`graph`, `search`, `stats`, and `mcp` all read
`<data-dir>/.grobid/grobid.db` (override with `--db`). They never contact
GROBID or the network — they work offline on whatever the last convert run
indexed. `search`/`stats` exit `1` with a hint when no database exists yet.

## Examples

```bash
# Convert ./pdf in the current directory
mkdir -p pdf && cp paper.pdf pdf/
grobid

# Convert with Crossref enrichment
GROBID_ENRICH=1 grobid        # or: grobid --enrich

# Use a dedicated data directory
GROBID_DATA_DIR=~/papers grobid

# Point at a non-default GROBID port
GROBID_PORT=8071 grobid

# Verbose logs (-v debug, -vv trace; or GROBID_LOG=debug)
grobid -v

# Limit concurrent GROBID requests
grobid  # or: GROBID_CONCURRENCY=2 grobid

# Corpus: search, stats, interactive dashboard
GROBID_DATA_DIR=~/papers grobid search "transformer"
GROBID_DATA_DIR=~/papers grobid stats
GROBID_DATA_DIR=~/papers grobid graph   # → ~/papers/.grobid/index.html
open ~/papers/.grobid/index.html

# MCP server for AI agents (stdio)
grobid mcp

# Version
$ grobid --version
grobid 0.0.1 (build 2026.11.07.22.40.32)
```

The version string combines the crate semver with a build timestamp
(`YYYY.MM.DD.HH.MM.SS`) captured from the binary's mtime — the timestamp
identifies the exact build.

## Output layout

```text
<data dir>/
├── pdf/        # input PDFs (you provide these)
├── tei/        # <name>.tei.xml — GROBID TEI XML intermediates
├── md/         # <name>.md — Markdown with YAML frontmatter
└── .grobid/    # all generated corpus artifacts:
    ├── grobid.db     # SQLite corpus: works + citation edges (-wal/-shm)
    ├── index.json    # portable JSON search index of all works
    ├── metadata/     # <pdf-stem>.json — full per-work metadata + references
    ├── process.json  # run manifest (last 50 runs)
    └── index.html    # corpus dashboard: graph + stats + year chart (`grobid graph`)
```
