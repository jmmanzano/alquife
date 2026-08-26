#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
PKG_NAME="alquife"
MAINTAINER="Alquife Contributors <noreply@example.com>"

if ! command -v dpkg-deb >/dev/null 2>&1; then
  echo "Error: dpkg-deb is required. Install it with: sudo apt install dpkg-dev" >&2
  exit 1
fi

if ! command -v dpkg >/dev/null 2>&1; then
  echo "Error: dpkg is required." >&2
  exit 1
fi

VERSION="$(sed -n 's/^version\s*=\s*"\([^"]*\)"/\1/p' "$PROJECT_ROOT/Cargo.toml" | head -n1)"
if [[ -z "$VERSION" ]]; then
  echo "Error: could not parse version from Cargo.toml" >&2
  exit 1
fi

ARCH="$(dpkg --print-architecture)"
BUILD_BASE="$PROJECT_ROOT/target/deb"
PKG_ROOT="$BUILD_BASE/${PKG_NAME}_${VERSION}_${ARCH}"
DEBIAN_DIR="$PKG_ROOT/DEBIAN"

BIN_SRC="$PROJECT_ROOT/target/release/$PKG_NAME"
ICON_SRC="$PROJECT_ROOT/icons/alquife.png"
if [[ ! -f "$ICON_SRC" ]]; then
  ICON_SRC="$PROJECT_ROOT/screenshots/alquife.png"
fi

DESKTOP_SRC="$PROJECT_ROOT/packaging/debian/alquife.desktop"
DEB_OUTPUT="$BUILD_BASE/${PKG_NAME}_${VERSION}_${ARCH}.deb"

echo "Building release binary..."
cd "$PROJECT_ROOT"
if ! cargo build --release; then
  echo "Standard cargo build failed. Retrying with nightly toolchain..."
  if ! cargo +nightly build --release; then
    echo "Error: build failed with both stable and nightly cargo." >&2
    exit 1
  fi
fi

if [[ ! -f "$BIN_SRC" ]]; then
  echo "Error: binary not found at $BIN_SRC" >&2
  exit 1
fi

if [[ ! -f "$DESKTOP_SRC" ]]; then
  echo "Error: desktop file not found at $DESKTOP_SRC" >&2
  exit 1
fi

if [[ ! -f "$ICON_SRC" ]]; then
  echo "Error: icon not found (expected icons/alquife.png or screenshots/alquife.png)" >&2
  exit 1
fi

echo "Preparing package tree..."
rm -rf "$PKG_ROOT"
mkdir -p "$DEBIAN_DIR"
install -Dm755 "$BIN_SRC" "$PKG_ROOT/usr/bin/$PKG_NAME"
install -Dm644 "$DESKTOP_SRC" "$PKG_ROOT/usr/share/applications/$PKG_NAME.desktop"
install -Dm644 "$ICON_SRC" "$PKG_ROOT/usr/share/icons/hicolor/256x256/apps/$PKG_NAME.png"

cat > "$DEBIAN_DIR/control" <<EOF
Package: $PKG_NAME
Version: $VERSION
Section: sound
Priority: optional
Architecture: $ARCH
Maintainer: $MAINTAINER
Depends: mpv, pipewire, wireplumber, dbus
Description: Terminal-based Subsonic music client
 Alquife is a terminal-based Subsonic music client written in Rust.
 It supports gapless playback, MPRIS controls, and PipeWire integration.
EOF

echo "Building .deb package..."
if dpkg-deb --help | grep -q -- '--root-owner-group'; then
  dpkg-deb --root-owner-group --build "$PKG_ROOT" "$DEB_OUTPUT" >/dev/null
else
  dpkg-deb --build "$PKG_ROOT" "$DEB_OUTPUT" >/dev/null
fi

echo "Done: $DEB_OUTPUT"
echo "Install with: sudo apt install \"$DEB_OUTPUT\""
