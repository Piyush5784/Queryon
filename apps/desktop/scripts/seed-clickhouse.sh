#!/usr/bin/env bash
# Fills the dev ClickHouse (queryon-clickhouse) container with N rows of
# random data per table, for testing pagination and the DataGrid against
# a large dataset. Requires the container to already be running (see
# docker-compose.yml's clickhouse service, or a standalone `docker run`).
#
# Usage: scripts/seed-clickhouse.sh <n>
#   n = base row count. Produces roughly n users, n products, n orders,
#       2n order_items, and n/20 categories.
#
# Safe to run more than once — each insert reads its own max(id) once
# via a WITH clause and offsets from there, so repeated runs only append
# new rows on top of whatever is already there.
#
# Uses `clickhouse-client --multiquery` (not the HTTP interface) because
# ClickHouse's HTTP interface rejects more than one `;`-separated
# statement per request — confirmed live ("Multi-statements are not
# allowed").

set -euo pipefail

N="${1:-}"
if [[ -z "$N" || ! "$N" =~ ^[0-9]+$ || "$N" -lt 1 ]]; then
  echo "Usage: $0 <n>   (n = a positive integer row count, e.g. 300)" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONTAINER="queryon-clickhouse"

echo "Seeding ClickHouse ($CONTAINER) with n=$N ..."
cat "$SCRIPT_DIR/seed-clickhouse.sql" | \
  docker exec -i "$CONTAINER" clickhouse-client --user devuser --password devpass --database devdb --param_n="$N" --multiquery

echo
echo "Done."
