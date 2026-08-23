#!/usr/bin/env bash
# Bulk random data generator for the dev SQLite seed schema
# (docker/initdb-sqlite/01_schema.sql). SQLite is embedded, not
# client-server, so there is no container to seed into — this builds a
# fresh .db file at the given path via the `sqlite3` CLI directly.
#
# Usage: scripts/seed-sqlite.sh <path> <n>
#   path = where to create the .db file (overwritten if it exists)
#   n = base row count, same semantics as seed-postgres.sql (n users,
#       n products, n orders, ~2n order_items)
#
# Same materialize-then-join fix as every other engine's seed script —
# confirmed live: a correlated `(select id from t order by random() limit
# 1)` scalar subquery is evaluated once per statement here too, not once
# per row, so every row in a batch gets the same picked id without this.
# SQLite's recursive CTE syntax (`with nums(g) as (... union all ...)`)
# is what generates the row-number sequence in place of Postgres's
# generate_series, which SQLite doesn't have.

set -euo pipefail

DB_PATH="${1:-}"
N="${2:-}"
if [[ -z "$DB_PATH" || -z "$N" || ! "$N" =~ ^[0-9]+$ || "$N" -lt 1 ]]; then
  echo "Usage: $0 <path> <n>   (n = a positive integer row count, e.g. 10000)" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BATCH_SIZE=500

echo "Seeding SQLite ($DB_PATH) with n=$N ..."
rm -f "$DB_PATH"
sqlite3 "$DB_PATH" < "$SCRIPT_DIR/../docker/initdb-sqlite/01_schema.sql"

run_sql() {
  sqlite3 "$DB_PATH" "$1"
}

CAT_TOTAL=$(( N / 20 > 5 ? N / 20 : 5 ))
run_sql "
insert into categories (name)
with nums(g) as (
  select 1
  union all
  select g + 1 from nums where g < $CAT_TOTAL
)
select 'Category ' || g from nums;
"
echo "  categories: $CAT_TOTAL"

done=0
while (( done < N )); do
  remaining=$(( N - done < BATCH_SIZE ? N - done : BATCH_SIZE ))
  run_sql "
  insert into users (email, full_name, is_active, metadata)
  with nums(g) as (
    select 1
    union all
    select g + 1 from nums where g < $remaining
  )
  select
      'user_' || ($done + g) || '_' || lower(hex(randomblob(4))) || '@example.com',
      (select value from json_each('[\"Alex\",\"Jordan\",\"Taylor\",\"Morgan\",\"Casey\",\"Riley\",\"Jamie\",\"Drew\"]') limit 1 offset abs(random()) % 8)
          || ' ' ||
      (select value from json_each('[\"Smith\",\"Johnson\",\"Lee\",\"Patel\",\"Garcia\",\"Kim\",\"Nguyen\",\"Brown\"]') limit 1 offset abs(random()) % 8),
      case when abs(random()) % 100 < 85 then 1 else 0 end,
      json_object(
          'plan', (select value from json_each('[\"free\",\"pro\",\"enterprise\"]') limit 1 offset abs(random()) % 3),
          'signupSource', (select value from json_each('[\"organic\",\"referral\",\"ads\",\"social\"]') limit 1 offset abs(random()) % 4)
      )
  from nums;
  "
  done=$(( done + remaining ))
  echo "  users: $done / $N"
done

done=0
while (( done < N )); do
  remaining=$(( N - done < BATCH_SIZE ? N - done : BATCH_SIZE ))
  run_sql "
  with nums(g) as (
    select 1
    union all
    select g + 1 from nums where g < $remaining
  ),
  cats as (select row_number() over (order by id) as rn, id from categories),
  cat_count as (select count(*) as n from categories),
  picks as (
      select g, abs(random()) % (select n from cat_count) + 1 as pick
      from nums
  )
  insert into products (category_id, sku, name, price_cents, in_stock)
  select
      c.id,
      'SKU-' || ($done + p.g) || '-' || lower(hex(randomblob(3))),
      (select value from json_each('[\"Widget\",\"Gadget\",\"Gizmo\",\"Doohickey\",\"Contraption\",\"Thingamajig\"]') limit 1 offset abs(random()) % 6)
          || ' ' || (select value from json_each('[\"Pro\",\"Max\",\"Mini\",\"Plus\",\"Lite\",\"Ultra\"]') limit 1 offset abs(random()) % 6),
      abs(random()) % 49900 + 100,
      case when abs(random()) % 100 < 90 then 1 else 0 end
  from picks p
  join cats c on c.rn = p.pick;
  "
  done=$(( done + remaining ))
  echo "  products: $done / $N"
done

done=0
while (( done < N )); do
  remaining=$(( N - done < BATCH_SIZE ? N - done : BATCH_SIZE ))
  run_sql "
  with nums(g) as (
    select 1
    union all
    select g + 1 from nums where g < $remaining
  ),
  usrs as (select row_number() over (order by id) as rn, id from users),
  user_count as (select count(*) as n from users),
  picks as (
      select g, abs(random()) % (select n from user_count) + 1 as pick
      from nums
  )
  insert into orders (user_id, status, total_cents, notes)
  select
      u.id,
      (select value from json_each('[\"pending\",\"paid\",\"shipped\",\"cancelled\"]') limit 1 offset abs(random()) % 4),
      abs(random()) % 99900 + 500,
      case when abs(random()) % 100 < 30 then 'Order note ' || ($done + p.g) else null end
  from picks p
  join usrs u on u.rn = p.pick;
  "
  done=$(( done + remaining ))
  echo "  orders: $done / $N"
done

ITEMS_TOTAL=$(( N * 2 ))
done=0
while (( done < ITEMS_TOTAL )); do
  remaining=$(( ITEMS_TOTAL - done < BATCH_SIZE ? ITEMS_TOTAL - done : BATCH_SIZE ))
  run_sql "
  with nums(g) as (
    select 1
    union all
    select g + 1 from nums where g < $remaining
  ),
  ords as (select row_number() over (order by id) as rn, id from orders),
  order_count as (select count(*) as n from orders),
  prods as (select row_number() over (order by id) as rn, id from products),
  product_count as (select count(*) as n from products),
  picks as (
      select
          g,
          abs(random()) % (select n from order_count) + 1 as order_pick,
          abs(random()) % (select n from product_count) + 1 as product_pick
      from nums
  )
  insert into order_items (order_id, product_id, quantity, unit_price_cents)
  select
      o.id,
      pr.id,
      abs(random()) % 5 + 1,
      abs(random()) % 49900 + 100
  from picks p
  join ords o on o.rn = p.order_pick
  join prods pr on pr.rn = p.product_pick;
  "
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
