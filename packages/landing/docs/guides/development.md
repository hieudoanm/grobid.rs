# Development guide

The repository has two build systems: Cargo for the Rust workspace (manifest
at the repo root) and pnpm/Turborepo for the Node packages.

## Toolchain

- Rust stable (see `rust-toolchain.toml`: stable + `rustfmt`, `clippy`)
- Node `26.13.0` (pnpm/turbo toolchain only — the landing page needs none)
- pnpm `12.18.0` via `packageManager` + corepack
- Docker (only needed to actually run conversions)

## Install

```bash
corepack enable
pnpm install
```

`pnpm-workspace.yaml` sets `minimumReleaseAge: 1440` (1 day) — long enough to
catch same-day supply-chain attacks without breaking installs, since renovate
pins fresh versions constantly.

## Rust workspace

From the repository root:

```bash
cargo build --release              # binary at target/release/grobid
cargo test --workspace             # unit tests incl. TEI fixture render
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

Short aliases via the root `Makefile`:

```bash
make build test lint fmt fmt-check clean   # fmt writes, fmt-check verifies
```

The same tasks run through turbo thanks to the `crates/package.json` shim
(`@grobid/cli` → `make -C .. <task>`):

```bash
pnpm lint     # cargo clippy (via @grobid/cli shim)
pnpm test     # cargo test
pnpm build    # cargo build --release
```

## Landing page

`packages/landing` is plain `index.html` + `styles.css` — no build, no
dependencies. Preview with any static server:

```bash
python3 -m http.server 8000 -d packages/landing
```

## Fixtures and e2e

`crates/grobid-core/tests/fixtures/` contains one real GROBID conversion
(sample PDF, generated TEI XML, expected Markdown). The unit test renders the
TEI fixture; the e2e script exercises the full pipeline:

```bash
./crates/grobid-cli/tests/e2e.sh
```

It builds `target/release/grobid` if missing, sets
`GROBID_DATA_DIR=crates/grobid-core/tests/fixtures`, and runs the binary —
existing `tei/`/`md/` outputs are skipped. Requires Docker for GROBID.

## Git hooks

`.husky/pre-commit` runs `turbo format` + `lint` on staged work
(`pnpm exec husky` / `pre-commit install` to enable locally).
