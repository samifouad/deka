#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEFAULT_ADWA_DIR="$ROOT_DIR/../adwa"
ADWA_DIR="${ADWA_DIR:-$DEFAULT_ADWA_DIR}"
if [ ! -d "$ADWA_DIR" ]; then
  ADWA_DIR="$ROOT_DIR/adwa"
fi

exec "$ADWA_DIR/scripts/test-playground-e2e.sh" "$@"
