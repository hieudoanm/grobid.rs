#!/usr/bin/env bash
# Install GROBID as a local JVM service — no Docker required.
#
# Downloads the GROBID source release, builds the standalone service +
# grobid-home distributions with Gradle (needs OpenJDK 21), and installs
# them under ~/.local/share/grobid.rs/grobid/ where the grobid CLI
# auto-detects them (override with GROBID_LOCAL_BIN).
#
# Usage: scripts/setup-grobid-local.sh [version]   (default: 0.9.1)

set -euo pipefail

VERSION="${1:-0.9.1}"
INSTALL_DIR="${GROBID_LOCAL_HOME:-$HOME/.local/share/grobid.rs/grobid}"
WORK_DIR="$(mktemp -d -t grobid-build)"
trap 'rm -rf "$WORK_DIR"' EXIT

echo "==> GROBID $VERSION → $INSTALL_DIR"

# --- Prerequisites ---
if ! command -v java >/dev/null 2>&1; then
  echo "ERROR: java not found. Install OpenJDK 21+ first." >&2
  exit 1
fi
JAVA_MAJOR="$(java -version 2>&1 | awk -F'"' '/version/ {print $2}' | cut -d. -f1)"
if [ "$JAVA_MAJOR" != "21" ] && [ "$JAVA_MAJOR" != "22" ] && [ "$JAVA_MAJOR" != "23" ] && [ "$JAVA_MAJOR" != "24" ]; then
  echo "WARN: java reports major version $JAVA_MAJOR; GROBID 0.9.x wants OpenJDK 21" >&2
fi

# --- Download source ---
cd "$WORK_DIR"
echo "==> Downloading grobid-$VERSION source"
curl -fsSL --retry 3 -o "grobid-$VERSION.zip" \
  "https://github.com/grobidOrg/grobid/archive/refs/tags/$VERSION.zip"
unzip -q "grobid-$VERSION.zip"
cd "grobid-$VERSION"
chmod +x gradlew

# --- Build distributions ---
echo "==> Building grobid-service + grobid-home distributions (Gradle, may take a while)"
./gradlew clean assemble --console=plain -q

SERVICE_ZIP="grobid-service/build/distributions/grobid-service-$VERSION.zip"
HOME_ZIP="grobid-home/build/distributions/grobid-home-$VERSION.zip"
[ -f "$SERVICE_ZIP" ] || { echo "ERROR: $SERVICE_ZIP not produced" >&2; exit 1; }
[ -f "$HOME_ZIP" ] || { echo "ERROR: $HOME_ZIP not produced" >&2; exit 1; }

# --- Install ---
echo "==> Installing to $INSTALL_DIR"
mkdir -p "$INSTALL_DIR"
rm -rf "$INSTALL_DIR/grobid-service" "$INSTALL_DIR/grobid-home"
unzip -q "$SERVICE_ZIP" -d "$INSTALL_DIR"
unzip -q "$HOME_ZIP" -d "$INSTALL_DIR"
mv "$INSTALL_DIR/grobid-service-$VERSION" "$INSTALL_DIR/grobid-service"
mv "$INSTALL_DIR/grobid-home-$VERSION" "$INSTALL_DIR/grobid-home"
chmod +x "$INSTALL_DIR/grobid-service/bin/grobid-service"

cat <<EOF

Done. The grobid CLI auto-detects:
  $INSTALL_DIR/grobid-service/bin/grobid-service

First run starts the JVM service automatically (models load lazily).
Check it with:  curl -s http://localhost:8070/api/isalive
Logs:           $INSTALL_DIR/grobid-service/grobid-service.log

To point at a non-default location, set GROBID_LOCAL_BIN=/path/to/grobid-service.
EOF
