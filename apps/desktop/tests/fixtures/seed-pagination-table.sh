#!/usr/bin/env bash
# Seeds a 5,000-row table in the real queryon-pg container for pagination
# testing, and exports it as JSON for the Playwright test's mocked Tauri
# bridge to serve (see postgres.spec.mjs).
#
# Usage: ./seed-pagination-table.sh
# Requires: queryon-pg container running (docker compose up -d postgres, from apps/desktop)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT_FILE="$SCRIPT_DIR/pagination-rows.json"

docker exec queryon-pg psql -U devuser -d devdb -c "
DROP TABLE IF EXISTS pagination_test;
CREATE TABLE pagination_test (
  id serial PRIMARY KEY,
  email text NOT NULL,
  full_name text NOT NULL,
  plan text NOT NULL,
  created_at date NOT NULL
);
INSERT INTO pagination_test (email, full_name, plan, created_at)
SELECT
  'user' || i || '@example.com',
  'User ' || i,
  CASE WHEN i % 2 = 0 THEN 'pro' ELSE 'free' END,
  (DATE '2026-01-01' + (i % 365))
FROM generate_series(1, 5000) AS i;
"

docker exec queryon-pg psql -U devuser -d devdb -t -A -c "
SELECT json_agg(json_build_array(id, email, full_name, plan, created_at::text) ORDER BY id)
FROM pagination_test;
" > "$OUT_FILE"

echo "Seeded 5,000 rows and wrote $(wc -c < "$OUT_FILE") bytes to $OUT_FILE"
