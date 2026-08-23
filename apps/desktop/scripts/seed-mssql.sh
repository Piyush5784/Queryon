#!/usr/bin/env bash
set -euo pipefail
N="${1:-}"
if [[ -z "$N" || ! "$N" =~ ^[0-9]+$ || "$N" -lt 1 ]]; then
  echo "Usage: $0 <n>   (n = a positive integer row count, e.g. 10000)" >&2
  exit 1
fi
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "Seeding SQL Server (queryon-mssql) with n=$N ..."
docker cp "$SCRIPT_DIR/seed-mssql.sql" queryon-mssql:/tmp/seed-mssql.sql
docker exec queryon-mssql /opt/mssql-tools18/bin/sqlcmd -S localhost -U sa -P 'DevPass123!' -C -v n="$N" -i /tmp/seed-mssql.sql
echo; echo "Done."
