#!/usr/bin/env bash
# Convenience launcher for the locally-built ALIV agent binary.
# Usage:
#   ./aliv-agent.sh                 -> opens the interactive agent UI
#   ./aliv-agent.sh agent status    -> forwards any args straight to `aliv`
#   ./aliv-agent.sh agent run "..." -> run a task headlessly

set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ALIV_BIN="$DIR/src-tauri/resources/bin/aliv"

if [ ! -x "$ALIV_BIN" ]; then
  echo "error: aliv binary not found or not executable at $ALIV_BIN" >&2
  exit 1
fi

cd "$DIR/src-tauri"

if [ "$#" -eq 0 ]; then
  exec "$ALIV_BIN" agent ui
else
  exec "$ALIV_BIN" "$@"
fi
