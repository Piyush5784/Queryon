#!/usr/bin/env bash
set -euo pipefail

if ! command -v strip >/dev/null 2>&1; then
  echo "strip-duckdb-lib: no strip on PATH, skipping"
  exit 0
fi

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
src_tauri_dir=$(dirname "$script_dir")
download_dir="$src_tauri_dir/target/duckdb-download"

if [[ ! -d "$download_dir" ]]; then
  echo "strip-duckdb-lib: $download_dir does not exist, skipping"
  exit 0
fi

lib=$(find "$download_dir" -name "libduckdb.so" 2>/dev/null | head -n1)

if [[ -z "$lib" ]]; then
  echo "strip-duckdb-lib: no libduckdb.so found under target/duckdb-download, skipping"
  exit 0
fi

echo "strip-duckdb-lib: stripping $lib"
strip --strip-unneeded "$lib"
