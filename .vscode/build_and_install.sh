#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
DEST_DIR="/usr/bin"
BIN_NAME="${1:-alquife}"
BIN_PATH="$PROJECT_ROOT/target/release/$BIN_NAME"

echo "Building release binary for '$BIN_NAME'..."
cd "$PROJECT_ROOT"
cargo build --release

if [[ ! -f "$BIN_PATH" ]]; then
  echo "Error: binary not found at '$BIN_PATH'" >&2
  exit 1
fi

echo "Installing '$BIN_NAME' to '$DEST_DIR'..."
if [[ -w "$DEST_DIR" ]]; then
  install -m 755 "$BIN_PATH" "$DEST_DIR/$BIN_NAME"
else
  sudo install -m 755 "$BIN_PATH" "$DEST_DIR/$BIN_NAME"
fi

echo "Done: $DEST_DIR/$BIN_NAME"
