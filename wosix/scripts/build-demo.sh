#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

"$ROOT_DIR/scripts/build-wasm.sh"

cd "$ROOT_DIR/js"
if [ ! -d node_modules ]; then
  echo "node_modules missing; run 'npm install' in $ROOT_DIR/js first."
  exit 1
fi

npm run build

echo "Demo assets built. Serve $ROOT_DIR/examples/browser"
