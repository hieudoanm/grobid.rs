#!/usr/bin/env bash
set -euo pipefail

# Directory of this script: crates/grobid-cli/tests
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
BINARY_PATH="$REPO_ROOT/target/release/grobid"
DATA_DIR="$REPO_ROOT/crates/grobid-core/tests/fixtures"

# Check if binary exists, if not build it
if [[ ! -f "$BINARY_PATH" ]]; then
    echo "Binary not found at $BINARY_PATH"
    echo "Building..."
    cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml"
fi

# Verify binary exists after build
if [[ ! -f "$BINARY_PATH" ]]; then
    echo "ERROR: Binary still not found after build"
    exit 1
fi

# Run the binary on the fixture PDF directory
echo "Running grobid on PDFs in $DATA_DIR/pdf"
export GROBID_DATA_DIR="$DATA_DIR"
cd "$DATA_DIR"
"$BINARY_PATH" -v
"$BINARY_PATH"
