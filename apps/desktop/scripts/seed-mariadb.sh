#!/usr/bin/env bash
# Fills the dev MariaDB (queryon-mariadb) container with N rows of random
# data per table, for testing pagination and the DataGrid against a large
# dataset. Requires `docker compose up -d mariadb` to already be running.
#
# Usage: scripts/seed-mariadb.sh <n>
#   n = base row count. Produces roughly n users, n products, n orders,
#       2n order_items, and n/20 categories.
#
# Safe to run more than once — it only appends new random rows on top of
# whatever is already there (existing hand-written seed data included).

set -euo pipefail

N="${1:-}"
if [[ -z "$N" || ! "$N" =~ ^[0-9]+$ || "$N" -lt 1 ]]; then
  echo "Usage: $0 <n>   (n = a positive integer row count, e.g. 10000)" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Seeding MariaDB (queryon-mariadb) with n=$N ..."
{ echo "set @n = $N;"; cat "$SCRIPT_DIR/seed-mariadb.sql"; } | \
  docker exec -i queryon-mariadb mariadb -u devuser -pdevpass devdb 2>&1 | grep -v "Using a password"

echo
echo "Done."
