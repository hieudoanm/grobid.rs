# fixtures

Sample files for the `pdf/` → `tei/` → `md/` conversion pipeline, used by the
unit test in `grobid-core` and the end-to-end script
`crates/grobid-cli/tests/e2e.sh`.

## Directory Structure

```
fixtures/
├── pdf/   # Input PDF files
├── tei/   # TEI XML output (GROBID-generated)
├── md/    # Expected Markdown output with frontmatter
└── README.md
```

## Example Files

- **pdf/** - `Predicting language outcome after stroke using machine learning - in search of the big data benefit.pdf`
- **tei/** - `Predicting language outcome after stroke using machine learning - in search of the big data benefit.tei.xml`
- **md/** - `Predicting language outcome after stroke using machine learning - in search of the big data benefit.md`

## Running the Example

```bash
./crates/grobid-cli/tests/e2e.sh   # from the repository root
```

This builds the `grobid` binary if needed, then runs it against `fixtures/`
with `GROBID_DATA_DIR` pointed here. Existing `tei/` and `md/` outputs are
skipped, so re-runs are cheap.

## Requirements

- Docker (for running the GROBID server)
- Rust toolchain (to build the binary)
