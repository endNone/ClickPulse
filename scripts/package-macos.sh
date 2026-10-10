#!/bin/sh
set -eu
PROJECT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$PROJECT_DIR"
exec python3 "$PROJECT_DIR/scripts/build.py" --platform macos "$@"
