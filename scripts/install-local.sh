#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
BINARY_NAME="grobid"

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
