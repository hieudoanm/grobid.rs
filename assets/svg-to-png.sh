#!/usr/bin/env bash
# Regenerate raster/landing assets from the canonical SVG sources.
#
#   assets/branding/svg/*.svg  ->  assets/branding/png/*.png   (sibling PNGs)
#   assets/branding/svg/logo.svg -> packages/landing/logo.svg  (copied verbatim)
#   assets/branding/svg/og.svg   -> packages/landing/og-image.png (1280x640)
#   packages/landing/logo.svg    -> packages/landing/favicon.ico (16/32/48/256)
#
# Usage: assets/svg-to-png.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ASSETS_DIR="$REPO_ROOT/assets"
BRANDING_SVG="$ASSETS_DIR/branding/svg"
LANDING_DIR="$REPO_ROOT/packages/landing"
JOBS="${JOBS:-4}"

command -v rsvg-convert >/dev/null 2>&1 || {
  echo "rsvg-convert is required but not installed" >&2
  echo "  macOS:  brew install librsvg" >&2
  echo "  Debian: sudo apt-get install librsvg2-bin" >&2
  echo "  Arch:   sudo pacman -S librsvg" >&2
  exit 1
}

pids=()
count=0
while IFS= read -r -d '' svg; do
  (
    png="$(dirname "$svg" | sed 's|/svg$|/png|')/$(basename "${svg%.svg}").png"

    if [ ! -s "$svg" ]; then
      echo "skip (empty): $svg" >&2
      exit 0
    fi

    mkdir -p "$(dirname "$png")"
    if rsvg-convert -o "$png" "$svg"; then
      echo "converted $svg -> $png"
    else
      echo "failed: $svg" >&2
      exit 1
    fi
  ) &
  pids+=("$!")
  count=$((count + 1))
  if [ "${#pids[@]}" -ge "$JOBS" ]; then
    wait "${pids[0]}" || true
    pids=("${pids[@]:1}")
  fi
done < <(find "$ASSETS_DIR" -type f -path '*/svg/*.svg' -print0)

status=0
for pid in "${pids[@]}"; do
  wait "$pid" || status=1
done

if [ "$count" -eq 0 ]; then
  echo "no svg files found under $ASSETS_DIR/*/svg" >&2
fi

# --- Landing page assets (flat layout, relative paths for GitHub Pages) ---

if [ -f "$BRANDING_SVG/logo.svg" ]; then
  cp "$BRANDING_SVG/logo.svg" "$LANDING_DIR/logo.svg"
  echo "copied $BRANDING_SVG/logo.svg -> $LANDING_DIR/logo.svg"
fi

if [ -f "$BRANDING_SVG/og.svg" ]; then
  rsvg-convert -w 1280 -h 640 -o "$LANDING_DIR/og-image.png" "$BRANDING_SVG/og.svg"
  echo "rendered $BRANDING_SVG/og.svg -> $LANDING_DIR/og-image.png"
fi

if command -v magick >/dev/null 2>&1 && [ -f "$LANDING_DIR/logo.svg" ]; then
  magick -background none "$LANDING_DIR/logo.svg" \
    \( -clone 0 -resize 16x16 \) \
    \( -clone 0 -resize 32x32 \) \
    \( -clone 0 -resize 48x48 \) \
    \( -clone 0 -resize 256x256 \) \
    -delete 0 "$LANDING_DIR/favicon.ico"
  echo "rendered $LANDING_DIR/favicon.ico (16/32/48/256)"
elif [ -f "$LANDING_DIR/logo.svg" ]; then
  echo "note: 'magick' not found — favicon.ico left unchanged" >&2
fi

exit "$status"
