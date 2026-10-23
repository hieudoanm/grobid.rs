# CLI reference

The `grobid` binary converts every PDF in the data directory's `pdf/` folder
to TEI XML (`tei/`) and Markdown (`md/`). There is one command and two flags.

## Usage

```bash
grobid [--version] [-v]
```

| Flag          | Short | Description                                             |
| ------------- | ----- | ------------------------------------------------------- |
| `--version`   | `-V`  | Print the version and exit.                             |
| `--verbose`   | `-v`  | Increase log verbosity; repeat for more (`-vv` = trace). |

There are no positional arguments — the data directory comes from the
environment (see below).

## Environment variables

| Variable               | Default                  | Description                                            |
| ---------------------- | ------------------------ | ------------------------------------------------------ |
| `GROBID_DATA_DIR`      | `ARTICLES_DATA_DIR`, then cwd | Base directory containing `pdf/`, `tei/`, `md/`  |
| `ARTICLES_DATA_DIR`    | —                        | Legacy alias for `GROBID_DATA_DIR`                     |
| `GROBID_URL`           | —                        | Full base URL of a GROBID instance (e.g. `http://grobid:8070`); when set, the CLI never manages a local Docker container and `GROBID_PORT` is ignored |
| `GROBID_PORT`          | `8070`                   | Port the GROBID service listens on                     |
| `GROBID_IMAGE`         | `grobid/grobid:0.9.1-crf`| Docker image used when auto-starting GROBID            |
| `GROBID_CONTAINER`     | `grobid-cli`             | Docker container name used when auto-starting GROBID   |
| `GROBID_CONCURRENCY`   | `4`                      | Max concurrent PDF→TEI requests                        |
| `GROBID_LOG`           | `info`                   | Logger level: `fatal`, `error`, `warn`, `info`, `debug`, `trace` |
| `RUST_LOG`             | `info`                   | `tracing` filter for CLI/GROBID-client logs            |

`GROBID_DATA_DIR` takes precedence over `ARTICLES_DATA_DIR`. When neither is
set, the current working directory is used, so `./pdf`, `./tei`, and `./md`
are expected relative to where you run the command.

## Behavior

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
     Markdown and written.
5. Exit code is `0` when every file succeeded (or there was nothing to do)
   and `1` if GROBID was unavailable or any file failed.

The HTTP client used for GROBID calls has a 300-second timeout.

## Examples

```bash
# Convert ./pdf in the current directory
mkdir -p pdf && cp paper.pdf pdf/
grobid

# Use a dedicated data directory
GROBID_DATA_DIR=~/papers grobid

# Point at a non-default GROBID port
GROBID_PORT=8071 grobid

# Verbose logs (-v debug, -vv trace; or GROBID_LOG=debug)
grobid -v

# Limit concurrent GROBID requests
grobid  # or: GROBID_CONCURRENCY=2 grobid

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
├── pdf/   # input PDFs (you provide these)
├── tei/   # <name>.tei.xml — GROBID TEI XML intermediates
└── md/    # <name>.md — Markdown with YAML frontmatter
```
