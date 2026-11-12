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

if [ "$WITH_GROBID" = 1 ]; then
  echo "==> Also installing the local GROBID JVM service (needs OpenJDK 21+)"
  curl -fsSL "https://raw.githubusercontent.com/hieudoanm/grobid.rs/master/scripts/setup-grobid-local.sh" | bash
else
  # No backend? Point the user at the two options.
  if ! command -v docker >/dev/null 2>&1 \
    && [ ! -x "$HOME/.local/share/grobid.rs/grobid/grobid-service/bin/grobid-service" ]; then
    echo "hint: no Docker or local GROBID found — pick a backend before first run:" >&2
    echo "  Docker:        install Docker, the CLI auto-starts the container" >&2
    echo "  JVM (no Docker): curl -fsSL https://raw.githubusercontent.com/hieudoanm/grobid.rs/master/scripts/setup-grobid-local.sh | bash" >&2
  fi
fi
