#!/usr/bin/env bash
set -euo pipefail

DB_PATH="${1:-}"
N="${2:-}"
if [[ -z "$DB_PATH" || -z "$N" || ! "$N" =~ ^[0-9]+$ || "$N" -lt 1 ]]; then
  echo "Usage: $0 <path> <n>   (n = a positive integer row count, e.g. 10000)" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DB_DIR="$(cd "$(dirname "$DB_PATH")" && pwd)"
DB_FILE="$(basename "$DB_PATH")"
IMAGE="davidgasquez/duckdb"

run_sql() {
  docker run --rm -v "$DB_DIR:/data" "$IMAGE" duckdb "/data/$DB_FILE" -c "$1"
}

run_sql_file() {
  docker run --rm -v "$DB_DIR:/data" -v "$SCRIPT_DIR/../docker/initdb-duckdb:/schema:ro" "$IMAGE" duckdb "/data/$DB_FILE" -init /schema/01_schema.sql -c "select 1"
}

echo "Seeding DuckDB ($DB_PATH) with n=$N ..."
rm -f "$DB_PATH"
run_sql_file > /dev/null

CAT_TOTAL=$(( N / 20 > 5 ? N / 20 : 5 ))
run_sql "
insert into categories (name)
select 'Category ' || (g + (select count(*) from categories))
from range($CAT_TOTAL) as t(g);
"
echo "  categories: $CAT_TOTAL"

run_sql "
insert into users (email, full_name, is_active, metadata)
select
    'user_' || (g + (select count(*) from users)) || '_' || lower(hex(random()::int)) || '@example.com',
    list_extract(['Alex','Jordan','Taylor','Morgan','Casey','Riley','Jamie','Drew'], (abs(random())::bigint % 8)::int + 1)
        || ' ' ||
    list_extract(['Smith','Johnson','Lee','Patel','Garcia','Kim','Nguyen','Brown'], (abs(random())::bigint % 8)::int + 1),
    case when abs(random())::bigint % 100 < 85 then true else false end,
    json_object(
        'plan', list_extract(['free','pro','enterprise'], (abs(random())::bigint % 3)::int + 1),
        'signupSource', list_extract(['organic','referral','ads','social'], (abs(random())::bigint % 4)::int + 1)
    )
from range($N) as t(g);
"
echo "  users: $N"

run_sql "
with cats as (select row_number() over (order by id) as rn, id from categories),
cat_count as (select count(*) as n from categories)
insert into products (category_id, sku, name, price_cents, in_stock)
select
    c.id,
    'SKU-' || (t.g + (select count(*) from products)) || '-' || lower(hex((random()*1000000)::int)),
    list_extract(['Widget','Gadget','Gizmo','Doohickey','Contraption','Thingamajig'], (abs(random())::bigint % 6)::int + 1)
        || ' ' || list_extract(['Pro','Max','Mini','Plus','Lite','Ultra'], (abs(random())::bigint % 6)::int + 1),
    (abs(random())::bigint % 49900 + 100)::int,
    case when abs(random())::bigint % 100 < 90 then true else false end
from range($N) as t(g)
join cats c on c.rn = (abs(random())::bigint % (select n from cat_count)) + 1;
"
echo "  products: $N"

run_sql "
with usrs as (select row_number() over (order by id) as rn, id from users),
user_count as (select count(*) as n from users)
insert into orders (user_id, status, total_cents, notes)
select
    u.id,
    list_extract(['pending','paid','shipped','cancelled'], (abs(random())::bigint % 4)::int + 1),
    (abs(random())::bigint % 99900 + 500)::int,
    case when abs(random())::bigint % 100 < 30 then 'Order note ' || t.g else null end
from range($N) as t(g)
join usrs u on u.rn = (abs(random())::bigint % (select n from user_count)) + 1;
"
echo "  orders: $N"

ITEMS_TOTAL=$(( N * 2 ))
run_sql "
with ords as (select row_number() over (order by id) as rn, id from orders),
order_count as (select count(*) as n from orders),
prods as (select row_number() over (order by id) as rn, id from products),
product_count as (select count(*) as n from products)
insert into order_items (order_id, product_id, quantity, unit_price_cents)
select
    o.id,
    pr.id,
    (abs(random())::bigint % 5 + 1)::int,
    (abs(random())::bigint % 49900 + 100)::int
from range($ITEMS_TOTAL) as t(g)
join ords o on o.rn = (abs(random())::bigint % (select n from order_count)) + 1
join prods pr on pr.rn = (abs(random())::bigint % (select n from product_count)) + 1;
"
echo "  order_items: $ITEMS_TOTAL"

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
