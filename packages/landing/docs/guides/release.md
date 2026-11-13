# Release guide

grobid.rs ships one artifact: a statically-linked-style Linux binary named
`grobid`, published to a rolling GitHub Release tagged `latest`. The landing
page deploys separately to GitHub Pages.

## CI pipeline (`.github/workflows/ci.yaml`)

Runs on push/PR to `master` (and `main` for release jobs):

| Job              | What it does                                                                                                                                                                                                                                                                                                                                                        |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cli`            | `cargo fmt --all --check`, `clippy --workspace --all-targets -D warnings`, `cargo test --verbose`, `cargo build --release`; uploads `target/release/grobid` as artifact `grobid-linux-amd64` (7-day retention). On master/main it also deletes the old `latest` release + tag and recreates it with the binary and README as body via `softprops/action-gh-release` |
| `landing-deploy` | On master/main: uploads `packages/landing` (plain HTML/CSS) to GitHub Pages (requires repo Settings → Pages → Source = "GitHub Actions")                                                                                                                                                                                                                            |

All Cargo commands run at the repository root — the workspace manifest is
`Cargo.toml` there, so `target/release/grobid` is the artifact path.

## Installing the release

```bash
# Install script — downloads linux/amd64 binary, warns if Docker is missing
curl -fsSL https://raw.githubusercontent.com/hieudoanm/grobid.rs/master/scripts/install.sh | bash
```

The script installs to `~/.local/bin` (`INSTALL_DIR` overrides) and checks that
`docker` is on `PATH`, since the CLI auto-starts GROBID through it.

Manual equivalent:

```bash
curl -fsSL https://github.com/hieudoanm/grobid.rs/releases/download/latest/grobid \
  -o ~/.local/bin/grobid && chmod +x ~/.local/bin/grobid
```

> Only a `linux/amd64` binary is published. On macOS or other platforms, build
> from source.

## Building from source

```bash
./scripts/install-local.sh          # cargo build --release → ~/.local/bin
# or manually
cargo build --release               # binary at target/release/grobid
```

`install-local.sh` fails early with a clear message if `cargo` is not installed
and honors `INSTALL_DIR`.

## Release checklist

Before tagging or merging to `master`:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release
./target/release/grobid --version

pnpm install --frozen-lockfile
pnpm lint && pnpm test && pnpm build
```

After merge, verify:

- The `latest` release contains `dist/grobid` and its body renders the README.
- `scripts/install.sh` downloads and runs on a clean Linux machine.
- The Pages site serves the new `packages/landing` files.

## Known limitations

- Single-platform release (`linux/amd64`); a build matrix for macOS/arm64 is the
  obvious next step.
- `grobid --version` reports a build timestamp, not semver — releases are not
  versioned.
- Docker is required at runtime for the GROBID service; there is no bundled or
  remote fallback.
