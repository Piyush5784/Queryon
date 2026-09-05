#!/usr/bin/env bash
set -euo pipefail

if ! command -v strip >/dev/null 2>&1; then
  echo "strip-duckdb-lib: no strip on PATH, skipping"
  exit 0
fi

lib=$(find target/duckdb-download -name "libduckdb.so" 2>/dev/null | head -n1)

if [[ -z "$lib" ]]; then
  echo "strip-duckdb-lib: no libduckdb.so found under target/duckdb-download, skipping"
  exit 0
fi

echo "strip-duckdb-lib: stripping $lib"
strip --strip-unneeded "$lib"
