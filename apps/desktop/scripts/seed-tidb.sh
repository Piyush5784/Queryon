#!/usr/bin/env bash
# Bulk random data generator for the dev TiDB seed schema
# (docker/initdb-mysql/01_schema.sql, applied to TiDB by the
# tidb-init compose service). Driven from bash instead of
# scripts/seed-mysql.sql's stored-procedure loops — TiDB does not
# support stored procedures at all (`ERROR 8108: Unsupported type
# *ast.DropProcedureStmt`), confirmed by running seed-mysql.sql directly
# against a live TiDB container. Recursive CTEs, temp tables, window
# functions, and JSON functions all work identically to MySQL, so only
# the batching mechanism needed to move into the driving script — same
# fix already applied for CockroachDB (see seed-cockroachdb.sh), and the
# same materialize-then-join pattern for FK picks: TiDB's
# `(select id from t order by rand() limit 1)` correlated subquery
# freezes to one value for every row, just like MySQL/MariaDB/CockroachDB
# — verified directly, not assumed.
#
# Usage: scripts/seed-tidb.sh <n>
#   n = base row count, same semantics as seed-mysql.sql.
#
# Safe to run more than once — every insert only appends.

set -euo pipefail

N="${1:-}"
if [[ -z "$N" || ! "$N" =~ ^[0-9]+$ || "$N" -lt 1 ]]; then
  echo "Usage: $0 <n>   (n = a positive integer row count, e.g. 10000)" >&2
  exit 1
fi

BATCH_SIZE=500
HOST="tidb"
PORT=4000
DB="devdb"

run_sql() {
  local output status
  output=$(docker run --rm --network neondb-client_default mariadb:11 \
    mariadb --skip-ssl -h "$HOST" -P "$PORT" -u devuser -pdevpass "$DB" \
    -e "set session cte_max_recursion_depth = 2000000; $1" 2>&1) && status=0 || status=$?
  # Capture output first so a silent successful query (nothing left
  # after filtering the client's password warning) never trips `set -e`
  # via grep's own exit code — only the actual mariadb command's status
  # decides whether this call failed.
  echo "$output" | grep -v "Using a password" || true
  return "$status"
}

echo "Seeding TiDB (queryon-tidb) with n=$N ..."

CAT_TOTAL=$(( N / 20 > 5 ? N / 20 : 5 ))
run_sql "
insert into categories (name)
with recursive nums(i) as (select 1 union all select i + 1 from nums where i < $CAT_TOTAL)
select concat('Category ', i) from nums
on duplicate key update name = name;
" > /dev/null

echo "  categories: $CAT_TOTAL"

done=0
while (( done < N )); do
  remaining=$(( N - done < BATCH_SIZE ? N - done : BATCH_SIZE ))
  run_sql "
  insert into users (email, full_name, is_active, metadata)
  select
      concat('user_', $done + n.i, '_', substring(md5(rand()), 1, 8), '@example.com'),
      concat(
          elt(1 + floor(rand() * 8), 'Alex', 'Jordan', 'Taylor', 'Morgan', 'Casey', 'Riley', 'Jamie', 'Drew'),
          ' ',
          elt(1 + floor(rand() * 8), 'Smith', 'Johnson', 'Lee', 'Patel', 'Garcia', 'Kim', 'Nguyen', 'Brown')
      ),
      rand() > 0.15,
      json_object(
          'plan', elt(1 + floor(rand() * 3), 'free', 'pro', 'enterprise'),
          'signupSource', elt(1 + floor(rand() * 4), 'organic', 'referral', 'ads', 'social')
      )
  from (
      with recursive nums(i) as (select 1 union all select i + 1 from nums where i < $remaining)
      select i from nums
  ) n;
  " > /dev/null
  done=$(( done + remaining ))
  echo "  users: $done / $N"
done

done=0
while (( done < N )); do
  remaining=$(( N - done < BATCH_SIZE ? N - done : BATCH_SIZE ))
  run_sql "
  insert into products (category_id, sku, name, price_cents, in_stock)
  with cats as (select row_number() over (order by id) as rn, id from categories),
       picks as (
           with recursive nums(i) as (select 1 union all select i + 1 from nums where i < $remaining)
           select i as g, 1 + floor(rand() * (select count(*) from categories)) as pick from nums
       )
  select
      c.id,
      concat('SKU-', $done + p.g, '-', substring(md5(rand()), 1, 6)),
      concat(
          elt(1 + floor(rand() * 6), 'Widget', 'Gadget', 'Gizmo', 'Doohickey', 'Contraption', 'Thingamajig'),
          ' ',
          elt(1 + floor(rand() * 6), 'Pro', 'Max', 'Mini', 'Plus', 'Lite', 'Ultra')
      ),
      floor(rand() * 49900 + 100),
      rand() > 0.1
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
  insert into orders (user_id, status, total_cents, notes)
  with usrs as (select row_number() over (order by id) as rn, id from users),
       picks as (
           with recursive nums(i) as (select 1 union all select i + 1 from nums where i < $remaining)
           select i as g, 1 + floor(rand() * (select count(*) from users)) as pick from nums
       )
  select
      u.id,
      elt(1 + floor(rand() * 4), 'pending', 'paid', 'shipped', 'cancelled'),
      floor(rand() * 99900 + 500),
      case when rand() > 0.7 then concat('Order note ', $done + p.g) else null end
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
  insert into order_items (order_id, product_id, quantity, unit_price_cents)
  with ords as (select row_number() over (order by id) as rn, id from orders),
       prods as (select row_number() over (order by id) as rn, id from products),
       picks as (
           with recursive nums(i) as (select 1 union all select i + 1 from nums where i < $remaining)
           select
               i as g,
               1 + floor(rand() * (select count(*) from orders)) as order_pick,
               1 + floor(rand() * (select count(*) from products)) as product_pick
           from nums
       )
  select
      o.id,
      pr.id,
      1 + floor(rand() * 5),
      floor(rand() * 49900 + 100)
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
