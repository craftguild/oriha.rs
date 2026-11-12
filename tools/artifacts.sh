#!/bin/sh
# Forward the existing library selector to the shared execution engine.
set -eu
project_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project_root"
exec python3 tools/library-entry.py "$@"
