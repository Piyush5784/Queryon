#!/usr/bin/env bash
# Bulk random data generator for the dev GreengageDB seed schema
# (docker/initdb-greengage/01_schema.sql). Bash-driven batching, not a
# SQL DO $$ ... $$ block — GreengageDB (GPDB6) is Postgres-9.4-era
# PL/pgSQL and rejects `commit;` inside a DO block ("cannot begin/end
# transactions in PL/pgSQL"), which seed-postgres.sql's per-batch commit
# depends on. Two more divergences found by testing directly against a
# live container, both matching the same class of bug already documented
# for MySQL/CockroachDB/TiDB's seed scripts:
#   - `(select id from t offset floor(random()*n) limit 1)` is evaluated
#     once per *statement* here, not once per row like real Postgres —
#     confirmed live (every row in a batch got the same picked id).
#     Fixed with the same materialize-then-join pattern used elsewhere:
#     generate a batch of random picks via generate_series, join against
#     a row_number()-numbered lookup table.
#   - No jsonb (only json) — uses json_build_object, not jsonb_build_object.
#   - No ON CONFLICT (Postgres 9.5+) — moot here anyway since
#     01_schema.sql drops the UNIQUE constraint on categories.name (GPDB
#     requires UNIQUE/PK columns to be a superset of the distribution
#     key), so a duplicate category name is just a plain non-unique row.
#
# Usage: scripts/seed-greengage.sh <n>
#   n = base row count, same semantics as seed-postgres.sql.
#
# Safe to run more than once — every insert only appends.

set -euo pipefail

N="${1:-}"
if [[ -z "$N" || ! "$N" =~ ^[0-9]+$ || "$N" -lt 1 ]]; then
  echo "Usage: $0 <n>   (n = a positive integer row count, e.g. 10000)" >&2
  exit 1
fi

BATCH_SIZE=500
CONTAINER="queryon-greengage"
DB="devdb"
PSQL="/usr/local/greenplum-db/bin/psql"

run_sql() {
  local output status
  output=$(docker exec -i "$CONTAINER" bash -lc "su - devuser -c \"$PSQL -d $DB -c \\\"$1\\\"\"" 2>&1) && status=0 || status=$?
  echo "$output" | grep -v "^$" || true
  return "$status"
}

echo "Seeding GreengageDB ($CONTAINER) with n=$N ..."

CAT_TOTAL=$(( N / 20 > 5 ? N / 20 : 5 ))
run_sql "
insert into categories (name)
select 'Category ' || g
from generate_series(1, $CAT_TOTAL) as g;
" > /dev/null
echo "  categories: $CAT_TOTAL"

done=0
while (( done < N )); do
  remaining=$(( N - done < BATCH_SIZE ? N - done : BATCH_SIZE ))
  run_sql "
  insert into users (email, full_name, is_active, metadata)
  select
      'user_' || ($done + g) || '_' || substr(md5(random()::text), 1, 8) || '@example.com',
      (array['Alex', 'Jordan', 'Taylor', 'Morgan', 'Casey', 'Riley', 'Jamie', 'Drew'])[1 + floor(random() * 8)::int]
          || ' ' ||
          (array['Smith', 'Johnson', 'Lee', 'Patel', 'Garcia', 'Kim', 'Nguyen', 'Brown'])[1 + floor(random() * 8)::int],
      random() > 0.15,
      json_build_object(
          'plan', (array['free', 'pro', 'enterprise'])[1 + floor(random() * 3)::int],
          'signupSource', (array['organic', 'referral', 'ads', 'social'])[1 + floor(random() * 4)::int]
      )
  from generate_series(1, $remaining) as g;
  " > /dev/null
  done=$(( done + remaining ))
  echo "  users: $done / $N"
done

done=0
while (( done < N )); do
  remaining=$(( N - done < BATCH_SIZE ? N - done : BATCH_SIZE ))
  run_sql "
  with cats as (select row_number() over (order by id) as rn, id from categories),
       cat_count as (select count(*)::float8 as n from categories),
       picks as (
           select g, (1 + floor(random() * (select n from cat_count)))::int as pick
           from generate_series(1, $remaining) as g
       )
  insert into products (category_id, sku, name, price_cents, in_stock)
  select
      c.id,
      'SKU-' || ($done + p.g) || '-' || substr(md5(random()::text), 1, 6),
      (array['Widget', 'Gadget', 'Gizmo', 'Doohickey', 'Contraption', 'Thingamajig'])[1 + floor(random() * 6)::int]
          || ' ' || (array['Pro', 'Max', 'Mini', 'Plus', 'Lite', 'Ultra'])[1 + floor(random() * 6)::int],
      (random() * 49900 + 100)::int,
      random() > 0.1
  from picks p
  join cats c on c.rn = p.pick;
  " > /dev/null
  done=$(( done + remaining ))
  echo "  products: $done / $N"
done

done=0
while (( done < N )); do
  remaining=$(( N - done < BATCH_SIZE ? N - done : BATCH_SIZE ))
  run_sql "
  with usrs as (select row_number() over (order by id) as rn, id from users),
       user_count as (select count(*)::float8 as n from users),
       picks as (
           select g, (1 + floor(random() * (select n from user_count)))::int as pick
           from generate_series(1, $remaining) as g
       )
  insert into orders (user_id, status, total_cents, notes)
  select
      u.id,
      (array['pending', 'paid', 'shipped', 'cancelled'])[1 + floor(random() * 4)::int],
      (random() * 99900 + 500)::int,
      case when random() > 0.7 then 'Order note ' || ($done + p.g) else null end
  from picks p
  join usrs u on u.rn = p.pick;
  " > /dev/null
  done=$(( done + remaining ))
  echo "  orders: $done / $N"
done

ITEMS_TOTAL=$(( N * 2 ))
done=0
while (( done < ITEMS_TOTAL )); do
  remaining=$(( ITEMS_TOTAL - done < BATCH_SIZE ? ITEMS_TOTAL - done : BATCH_SIZE ))
  run_sql "
  with ords as (select row_number() over (order by id) as rn, id from orders),
       order_count as (select count(*)::float8 as n from orders),
       prods as (select row_number() over (order by id) as rn, id from products),
       product_count as (select count(*)::float8 as n from products),
       picks as (
           select
               g,
               (1 + floor(random() * (select n from order_count)))::int as order_pick,
               (1 + floor(random() * (select n from product_count)))::int as product_pick
           from generate_series(1, $remaining) as g
       )
  insert into order_items (order_id, product_id, quantity, unit_price_cents)
  select
      o.id,
      pr.id,
      (1 + floor(random() * 5))::int,
      (random() * 49900 + 100)::int
  from picks p
  join ords o on o.rn = p.order_pick
  join prods pr on pr.rn = p.product_pick;
  " > /dev/null
  done=$(( done + remaining ))
  echo "  order_items: $done / $ITEMS_TOTAL"
done

echo
run_sql "
select
    (select count(*) from users) as users,
    (select count(*) from categories) as categories,
    (select count(*) from products) as products,
    (select count(*) from orders) as orders,
    (select count(*) from order_items) as order_items;
"
echo "Done."
