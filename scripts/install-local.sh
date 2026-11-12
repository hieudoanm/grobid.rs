#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
BINARY_NAME="grobid"
WITH_GROBID=0

for arg in "$@"; do
  case "$arg" in
    --with-grobid | --grobid) WITH_GROBID=1 ;;
    *) echo "error: unknown option $arg (supported: --with-grobid)" >&2; exit 1 ;;
  esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo not found — install Rust from https://rustup.rs" >&2
  exit 1
fi

echo "Building $BINARY_NAME from $REPO_ROOT"
cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml"

mkdir -p "$INSTALL_DIR"
cp "$REPO_ROOT/target/release/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo "Installed to $INSTALL_DIR/$BINARY_NAME"

if [ "$WITH_GROBID" = 1 ]; then
  "$SCRIPT_DIR/setup-grobid-local.sh"
elif ! command -v docker >/dev/null 2>&1 \
  && [ ! -x "$HOME/.local/share/grobid.rs/grobid/grobid-service/bin/grobid-service" ]; then
  echo "hint: no Docker or local GROBID found — pick a backend before first run:" >&2
  echo "  Docker:        install Docker, the CLI auto-starts the container" >&2
  echo "  JVM (no Docker): $SCRIPT_DIR/setup-grobid-local.sh" >&2
fi
