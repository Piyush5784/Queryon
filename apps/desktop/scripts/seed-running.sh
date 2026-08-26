#!/usr/bin/env bash
set -euo pipefail

N="${1:-}"
if [[ -z "$N" || ! "$N" =~ ^[0-9]+$ || "$N" -lt 1 ]]; then
  echo "Usage: $0 <n>   (n = a positive integer row count, e.g. 10000)" >&2
  echo "Seeds whichever dev database containers are currently running. DuckDB/SQLite are file-based and not covered here — use scripts/seed-duckdb.sh/seed-sqlite.sh directly." >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

running() {
  docker ps --format "{{.Names}}" | grep -qx "$1"
}

seeded_any=0

if running queryon-pg || running queryon-mysql; then
  echo "=== Postgres/MySQL ==="
  "$SCRIPT_DIR/seed.sh" "$N"
  seeded_any=1
fi

if running queryon-mariadb; then
  echo "=== MariaDB ==="
  "$SCRIPT_DIR/seed-mariadb.sh" "$N"
  seeded_any=1
fi

if running queryon-tidb; then
  echo "=== TiDB ==="
  "$SCRIPT_DIR/seed-tidb.sh" "$N"
  seeded_any=1
fi

if running queryon-cockroachdb; then
  echo "=== CockroachDB ==="
  "$SCRIPT_DIR/seed-cockroachdb.sh" "$N"
  seeded_any=1
fi

if running queryon-greengage; then
  echo "=== GreengageDB ==="
  "$SCRIPT_DIR/seed-greengage.sh" "$N"
  seeded_any=1
fi

if running queryon-mssql; then
  echo "=== SQL Server ==="
  "$SCRIPT_DIR/seed-mssql.sh" "$N"
  seeded_any=1
fi

if running queryon-clickhouse; then
  echo "=== ClickHouse ==="
  CLICKHOUSE_SCHEMA="$SCRIPT_DIR/../docker/initdb-clickhouse/01_schema.sql"
  TABLE_COUNT=$(docker exec queryon-clickhouse clickhouse-client --user devuser --password devpass --database devdb --query "SELECT count() FROM system.tables WHERE database = 'devdb'" 2>/dev/null || echo "0")
  if [[ "$TABLE_COUNT" -eq 0 ]]; then
    echo "  loading schema first..."
    docker exec -i queryon-clickhouse clickhouse-client --user devuser --password devpass --database devdb --multiquery < "$CLICKHOUSE_SCHEMA"
  fi
  "$SCRIPT_DIR/seed-clickhouse.sh" "$N"
  seeded_any=1
fi

if running queryon-starrocks; then
  echo "=== StarRocks ==="
  "$SCRIPT_DIR/seed-starrocks.sh" "$N"
  seeded_any=1
fi

if [[ "$seeded_any" -eq 0 ]]; then
  echo "No dev database containers are currently running. Start one with 'docker compose up -d <service>' first." >&2
  exit 1
fi

echo
echo "Done."
