#!/usr/bin/env bash
# Convert SVG sources in assets/svg/ to PNGs in assets/images/ (output size
# comes from each SVG's intrinsic dimensions), then refresh the flat
# landing page assets: packages/landing/ (logo.svg, og-image.png,
# favicon.ico).
#
# Usage: assets/svg-to-png.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SVG_DIR="$REPO_ROOT/assets/svg"
PNG_DIR="$REPO_ROOT/assets/images"
LANDING_DIR="$REPO_ROOT/packages/landing"

if ! command -v rsvg-convert >/dev/null 2>&1; then
  echo "rsvg-convert not found. Install librsvg:"
  echo "  macOS:  brew install librsvg"
  echo "  Debian: sudo apt-get install librsvg2-bin"
  echo "  Arch:   sudo pacman -S librsvg"
  exit 1
fi

mkdir -p "$PNG_DIR"

for svg in "$SVG_DIR"/*.svg; do
  name="$(basename "$svg" .svg)"
  png="$PNG_DIR/$name.png"
  echo "Converting $svg -> $png"
  rsvg-convert -o "$png" "$svg"
done

# Landing page assets (flat static site in packages/landing).
if [[ -f "$SVG_DIR/logo.svg" ]]; then
  cp "$SVG_DIR/logo.svg" "$LANDING_DIR/logo.svg"
fi
if [[ -f "$SVG_DIR/og.svg" ]]; then
  echo "Converting $SVG_DIR/og.svg -> $LANDING_DIR/og-image.png"
  rsvg-convert -o "$LANDING_DIR/og-image.png" "$SVG_DIR/og.svg"
fi

# favicon.ico: multi-size icon rendered from the logo.
if [[ -f "$SVG_DIR/logo.svg" ]]; then
  if command -v magick >/dev/null 2>&1; then
    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT
    for size in 16 32 48 256; do
      rsvg-convert -w "$size" -h "$size" "$SVG_DIR/logo.svg" -o "$tmpdir/favicon-$size.png"
    done
    magick "$tmpdir"/favicon-*.png "$LANDING_DIR/favicon.ico"
    echo "Wrote $LANDING_DIR/favicon.ico"
  else
    echo "warning: magick not found — skipping favicon.ico (install ImageMagick)" >&2
  fi
fi

echo "Done."
