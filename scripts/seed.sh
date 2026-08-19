#!/usr/bin/env bash
# Fills the dev Postgres (queryon-pg) and dev MySQL (queryon-mysql)
# containers with N rows of random data per table, for testing pagination
# and the DataGrid against a large dataset. Requires `docker compose up -d`
# to already be running (see README.md).
#
# Usage: scripts/seed.sh <n>
#   n = base row count. Produces roughly n users, n products, n orders,
#       2n order_items, and n/20 categories, in both Postgres and MySQL.
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

echo "Seeding Postgres (queryon-pg) with n=$N ..."
docker exec -i queryon-pg psql -U devuser -d devdb -v n="$N" -f - < "$SCRIPT_DIR/seed-postgres.sql"

echo
echo "Seeding MySQL (queryon-mysql) with n=$N ..."
{ echo "set @n = $N;"; cat "$SCRIPT_DIR/seed-mysql.sql"; } | \
  docker exec -i queryon-mysql mysql -u devuser -pdevpass devdb 2>&1 | grep -v "Using a password"

echo
echo "Done."
