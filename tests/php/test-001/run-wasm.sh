#!/usr/bin/env bash
set -euo pipefail

wasmtime target/wasm32-wasip1/debug/php-wasm.wasm -- tests/php/test-001/hello.php
