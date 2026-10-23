#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
BINARY_NAME="grobid"

OS="$(uname -s)"
ARCH="$(uname -m)"

# CI only publishes a Linux x86_64 binary (release asset: "grobid").
if [[ "$OS" != "Linux" || "$ARCH" != "x86_64" ]]; then
  echo "error: prebuilt releases are Linux x86_64 only (detected ${OS} ${ARCH})." >&2
  echo "Build from source instead:" >&2
  echo "  ./scripts/install-local.sh" >&2
  exit 1
fi

echo "Installing $BINARY_NAME to $INSTALL_DIR"
mkdir -p "$INSTALL_DIR"

URL="https://github.com/hieudoanm/grobid.rs/releases/download/latest/grobid"
curl -fsSL "$URL" -o "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo "Installed to $INSTALL_DIR/$BINARY_NAME"
