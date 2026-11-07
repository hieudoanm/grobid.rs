# CLI reference

The `grobid` binary converts every PDF in the data directory's `pdf/` folder
to TEI XML (`tei/`) and Markdown (`md/`). There is one command and one flag.

## Usage

```bash
grobid [--version]
```

| Flag        | Short | Description                                  |
| ----------- | ----- | -------------------------------------------- |
| `--version` | `-v`  | Print the build-timestamp version and exit. |

There are no positional arguments — the data directory comes from the
environment (see below).

## Environment variables

| Variable               | Default                  | Description                                            |
| ---------------------- | ------------------------ | ------------------------------------------------------ |
| `GROBID_DATA_DIR`      | `ARTICLES_DATA_DIR`, then cwd | Base directory containing `pdf/`, `tei/`, `md/`  |
| `ARTICLES_DATA_DIR`    | —                        | Legacy alias for `GROBID_DATA_DIR`                     |
| `GROBID_PORT`          | `8070`                   | Port the GROBID service listens on                     |
| `GROBID_IMAGE`         | `grobid/grobid:0.9.1-crf`| Docker image used when auto-starting GROBID            |
| `GROBID_CONTAINER`     | `grobid-cli`             | Docker container name used when auto-starting GROBID   |
| `RUST_LOG`             | `info`                   | `tracing` filter for CLI/GROBID-client logs            |

`GROBID_DATA_DIR` takes precedence over `ARTICLES_DATA_DIR`. When neither is
set, the current working directory is used, so `./pdf`, `./tei`, and `./md`
are expected relative to where you run the command.

## Behavior

1. `paths::init_dirs()` creates `pdf/`, `tei/`, and `md/` under the data dir.
2. If `pdf/` is empty, the run logs a warning and exits `0`.
3. GROBID is started if needed (see [03-grobid-service.md](./03-grobid-service.md)).
   If it cannot be reached, the run exits `1`.
4. For each PDF, in sorted order:
   - If `tei/<name>.tei.xml` exists it is reused; otherwise the PDF is POSTed
     to GROBID (`processFulltextDocument`, `consolidateHeader=1`).
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

# Verbose logs
RUST_LOG=debug grobid

# Version
$ grobid --version
grobid 2026.11.07.22.40.32
```

The version string is a build timestamp (`YYYY.MM.DD.HH.MM.SS`) captured when
the binary was compiled — it identifies the build, not a semver release.

## Output layout

```text
<data dir>/
├── pdf/   # input PDFs (you provide these)
├── tei/   # <name>.tei.xml — GROBID TEI XML intermediates
└── md/    # <name>.md — Markdown with YAML frontmatter
```
