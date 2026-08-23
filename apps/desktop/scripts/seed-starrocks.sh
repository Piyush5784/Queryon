#!/usr/bin/env bash
# Loads docker/initdb-starrocks/01_schema.sql, then bulk-seeds n more
# users/products/orders/order_items on top of it.
#
# The schema file is NOT run as one plain `mysql < file.sql` batch —
# StarRocks schema-change DDL (CREATE INDEX included) is asynchronous,
# and the next DDL statement on the same table fails outright if the
# previous one's background job hasn't finished ("A schema change
# operation is in progress"), confirmed live when order_items' two
# CREATE INDEX statements ran back-to-back in one file. This script
# splits the schema file into two phases instead: table/index DDL first
# (each `create table`/`create index` run individually, waited out via
# wait_for_schema_change), then the seed INSERTs as one final batch
# (INSERT never triggers a schema-change job, confirmed live).
#
# Usage: scripts/seed-starrocks.sh <n>
#   n = base row count for the *additional* bulk data, same semantics as
#       seed-postgres.sql (n users, n products, n orders, ~2n order_items).
#       The schema file's own hand-written rows are untouched either way.

set -euo pipefail

N="${1:-}"
if [[ -z "$N" || ! "$N" =~ ^[0-9]+$ || "$N" -lt 1 ]]; then
  echo "Usage: $0 <n>   (n = a positive integer row count, e.g. 10000)" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONTAINER="queryon-starrocks"
BATCH_SIZE=500

# SQL is always piped over stdin, never passed as a -e/bash -lc string —
# nesting docker exec's own quoting through a second `bash -lc "..."`
# layer for arbitrary SQL (embedded quotes, backslashes) proved fragile
# in practice, so every statement here goes in via a heredoc instead.
run_sql() {
  docker exec -i "$CONTAINER" mysql -h 127.0.0.1 -P 9030 -u root -pdevpass
}

run_sql_db() {
  docker exec -i "$CONTAINER" mysql -h 127.0.0.1 -P 9030 -u root -pdevpass devdb
}

wait_for_schema_change() {
  local table="$1"
  # A brief sleep before the first check, not just between retries — a
  # newly-issued schema change (e.g. CREATE INDEX) doesn't show up in
  # `SHOW ALTER TABLE COLUMN` immediately, confirmed live: checking with
  # no delay can read back the *previous* statement's already-FINISHED
  # job (or no job at all) before the new one has registered, returning
  # "done" prematurely and letting the next statement race the real job.
  sleep 1
  for _ in $(seq 1 30); do
    local state
    state=$(docker exec -i "$CONTAINER" mysql -h 127.0.0.1 -P 9030 -u root -pdevpass devdb --skip-column-names -e "show alter table column from devdb where TableName='$table' order by CreateTime desc limit 1" 2>/dev/null | awk -F'\t' '{print $10}')
    if [[ -z "$state" || "$state" == "FINISHED" || "$state" == "CANCELLED" ]]; then
      return 0
    fi
    sleep 1
  done
  echo "Timed out waiting for schema change on $table" >&2
  return 1
}

# StarRocks' `root` has no password on a fresh container — sqlx's MySQL
# client rejects that outright ("Access denied for user 'root' (using
# password: YES)", confirmed live: sqlx always sends a non-empty auth
# response even for an empty password), so this app's own driver needs a
# real password set. Idempotent: only sets it if root is still
# passwordless (this script's own `-pdevpass` calls above already
# succeed on a re-run against a container this script already seeded).
if docker exec -i "$CONTAINER" mysql -h 127.0.0.1 -P 9030 -u root -e "select 1" > /dev/null 2>&1; then
  echo "Setting a password for StarRocks' root user (required by this app's MySQL client) ..."
  echo "SET PASSWORD FOR 'root' = PASSWORD('devpass');" | docker exec -i "$CONTAINER" mysql -h 127.0.0.1 -P 9030 -u root
fi

echo "Resetting devdb on StarRocks ($CONTAINER) ..."
echo "drop database if exists devdb;" | run_sql

echo "Loading table/index DDL ..."
echo "create database devdb;" | run_sql

run_sql_db <<'EOF' > /dev/null
create table users (
    id            bigint not null auto_increment,
    email         varchar(255) not null,
    full_name     varchar(255) not null,
    is_active     boolean not null default '1',
    metadata      json not null,
    created_at    datetime not null default current_timestamp
)
primary key (id)
distributed by hash (id);
EOF
wait_for_schema_change users

run_sql_db <<'EOF' > /dev/null
create table categories (
    id    bigint not null auto_increment,
    name  varchar(255) not null
)
primary key (id)
distributed by hash (id);
EOF
wait_for_schema_change categories

run_sql_db <<'EOF' > /dev/null
create table products (
    id            bigint not null auto_increment,
    category_id   bigint,
    sku           varchar(100) not null,
    name          varchar(255) not null,
    price_cents   int not null,
    in_stock      boolean not null default '1',
    created_at    datetime not null default current_timestamp
)
primary key (id)
distributed by hash (id);
EOF
wait_for_schema_change products

echo "create index idx_products_category_id on products(category_id);" | run_sql_db > /dev/null
wait_for_schema_change products

run_sql_db <<'EOF' > /dev/null
create table orders (
    id            bigint not null auto_increment,
    user_id       bigint not null,
    status        varchar(20) not null default 'pending',
    total_cents   int not null default '0',
    notes         varchar(1000),
    created_at    datetime not null default current_timestamp
)
primary key (id)
distributed by hash (id);
EOF
wait_for_schema_change orders

echo "create index idx_orders_user_id on orders(user_id);" | run_sql_db > /dev/null
wait_for_schema_change orders

run_sql_db <<'EOF' > /dev/null
create table order_items (
    id            bigint not null auto_increment,
    order_id      bigint not null,
    product_id    bigint not null,
    quantity      int not null,
    unit_price_cents int not null
)
primary key (id)
distributed by hash (id);
EOF
wait_for_schema_change order_items

echo "create index idx_order_items_order_id on order_items(order_id);" | run_sql_db > /dev/null
wait_for_schema_change order_items

echo "create index idx_order_items_product_id on order_items(product_id);" | run_sql_db > /dev/null
wait_for_schema_change order_items

echo "Loading seed rows ..."
# No explicit id values — see docker/initdb-starrocks/01_schema.sql's doc
# comment for why (auto_increment restarts at 1 on this table's very
# first insert, and a colliding id on a PRIMARY KEY table silently
# upserts rather than erroring, so a later explicit id=1..4 batch would
# overwrite these rows outright).
run_sql_db <<'EOF' > /dev/null
insert into users (email, full_name, is_active, metadata) values
    ('alice@example.com', 'Alice Nguyen', true,  '{"plan": "pro"}'),
    ('bob@example.com',   'Bob Martinez', true,  '{"plan": "free"}'),
    ('carol@example.com', 'Carol Singh',  false, '{"plan": "free"}'),
    ('dave@example.com',  'Dave Okafor',  true,  '{"plan": "pro"}');
insert into categories (name) values
    ('Electronics'), ('Books'), ('Home & Kitchen');
insert into products (category_id, sku, name, price_cents, in_stock)
select id, 'ELEC-001', 'Wireless Mouse', 2499, true from categories where name = 'Electronics'
union all
select id, 'ELEC-002', 'Mechanical Keyboard', 8999, true from categories where name = 'Electronics'
union all
select id, 'BOOK-001', 'The Pragmatic Programmer', 3999, true from categories where name = 'Books'
union all
select id, 'HOME-001', 'French Press', 1899, false from categories where name = 'Home & Kitchen';
insert into orders (user_id, status, total_cents, notes)
select id, 'paid', 11498, null from users where email = 'alice@example.com'
union all
select id, 'shipped', 3999, 'Gift wrap requested' from users where email = 'alice@example.com'
union all
select id, 'pending', 2499, null from users where email = 'bob@example.com'
union all
select id, 'cancelled', 1899, 'Customer changed mind' from users where email = 'dave@example.com';
insert into order_items (order_id, product_id, quantity, unit_price_cents)
select o.id, p.id, 1, 2499 from orders o, products p where o.total_cents = 11498 and p.sku = 'ELEC-001'
union all
select o.id, p.id, 1, 8999 from orders o, products p where o.total_cents = 11498 and p.sku = 'ELEC-002'
union all
select o.id, p.id, 1, 3999 from orders o, products p where o.total_cents = 3999 and p.sku = 'BOOK-001'
union all
select o.id, p.id, 1, 2499 from orders o, products p where o.total_cents = 2499 and p.sku = 'ELEC-001'
union all
select o.id, p.id, 1, 1899 from orders o, products p where o.total_cents = 1899 and p.sku = 'HOME-001';
EOF

echo "Seeding StarRocks (queryon-starrocks) with n=$N ..."

CAT_TOTAL=$(( N / 20 > 5 ? N / 20 : 5 ))
run_sql_db <<EOF > /dev/null
insert into categories (name)
select concat('Category ', CAST(seq AS VARCHAR))
from table(generate_series(1, $CAT_TOTAL)) as t(seq);
EOF
echo "  categories: $CAT_TOTAL"

done=0
while (( done < N )); do
  remaining=$(( N - done < BATCH_SIZE ? N - done : BATCH_SIZE ))
  run_sql_db <<EOF > /dev/null
  insert into users (email, full_name, is_active, metadata)
  select
      concat('user_', CAST($done + seq AS VARCHAR), '_', substring(md5(rand()), 1, 8), '@example.com'),
      concat(
          case cast(floor(rand()*8) as int)
              when 0 then 'Alex' when 1 then 'Jordan' when 2 then 'Taylor' when 3 then 'Morgan'
              when 4 then 'Casey' when 5 then 'Riley' when 6 then 'Jamie' else 'Drew' end,
          ' ',
          case cast(floor(rand()*8) as int)
              when 0 then 'Smith' when 1 then 'Johnson' when 2 then 'Lee' when 3 then 'Patel'
              when 4 then 'Garcia' when 5 then 'Kim' when 6 then 'Nguyen' else 'Brown' end
      ),
      rand() < 0.85,
      concat('{"plan": "',
          case cast(floor(rand()*3) as int) when 0 then 'free' when 1 then 'pro' else 'enterprise' end,
          '", "signupSource": "',
          case cast(floor(rand()*4) as int) when 0 then 'organic' when 1 then 'referral' when 2 then 'ads' else 'social' end,
          '"}')
  from table(generate_series(1, $remaining)) as t(seq);
EOF
  done=$(( done + remaining ))
  echo "  users: $done / $N"
done

done=0
while (( done < N )); do
  remaining=$(( N - done < BATCH_SIZE ? N - done : BATCH_SIZE ))
  run_sql_db <<EOF > /dev/null
  insert into products (category_id, sku, name, price_cents, in_stock)
  with cats as (select row_number() over (order by id) as rn, id from categories),
       cat_count as (select count(*) as n from categories),
       picks as (
           select seq, cast(floor(rand() * (select n from cat_count)) as int) + 1 as pick
           from table(generate_series(1, $remaining)) as t(seq)
       )
  select
      c.id,
      concat('SKU-', CAST($done + p.seq AS VARCHAR), '-', substring(md5(rand()), 1, 6)),
      concat(
          case cast(floor(rand()*6) as int)
              when 0 then 'Widget' when 1 then 'Gadget' when 2 then 'Gizmo'
              when 3 then 'Doohickey' when 4 then 'Contraption' else 'Thingamajig' end,
          ' ',
          case cast(floor(rand()*6) as int)
              when 0 then 'Pro' when 1 then 'Max' when 2 then 'Mini'
              when 3 then 'Plus' when 4 then 'Lite' else 'Ultra' end
      ),
      cast(floor(rand()*49900) as int) + 100,
      rand() < 0.9
  from picks p
  join cats c on c.rn = p.pick;
EOF
  done=$(( done + remaining ))
  echo "  products: $done / $N"
done

done=0
while (( done < N )); do
  remaining=$(( N - done < BATCH_SIZE ? N - done : BATCH_SIZE ))
  run_sql_db <<EOF > /dev/null
  insert into orders (user_id, status, total_cents, notes)
  with usrs as (select row_number() over (order by id) as rn, id from users),
       user_count as (select count(*) as n from users),
       picks as (
           select seq, cast(floor(rand() * (select n from user_count)) as int) + 1 as pick
           from table(generate_series(1, $remaining)) as t(seq)
       )
  select
      u.id,
      case cast(floor(rand()*4) as int)
          when 0 then 'pending' when 1 then 'paid' when 2 then 'shipped' else 'cancelled' end,
      cast(floor(rand()*99900) as int) + 500,
      case when rand() < 0.3 then concat('Order note ', CAST($done + p.seq AS VARCHAR)) else null end
  from picks p
  join usrs u on u.rn = p.pick;
EOF
  done=$(( done + remaining ))
  echo "  orders: $done / $N"
done

ITEMS_TOTAL=$(( N * 2 ))
done=0
while (( done < ITEMS_TOTAL )); do
  remaining=$(( ITEMS_TOTAL - done < BATCH_SIZE ? ITEMS_TOTAL - done : BATCH_SIZE ))
  run_sql_db <<EOF > /dev/null
  insert into order_items (order_id, product_id, quantity, unit_price_cents)
  with ords as (select row_number() over (order by id) as rn, id from orders),
       order_count as (select count(*) as n from orders),
       prods as (select row_number() over (order by id) as rn, id from products),
       product_count as (select count(*) as n from products),
       picks as (
           select
               seq,
               cast(floor(rand() * (select n from order_count)) as int) + 1 as order_pick,
               cast(floor(rand() * (select n from product_count)) as int) + 1 as product_pick
           from table(generate_series(1, $remaining)) as t(seq)
       )
  select
      o.id,
      pr.id,
      cast(floor(rand()*5) as int) + 1,
      cast(floor(rand()*49900) as int) + 100
  from picks p
  join ords o on o.rn = p.order_pick
  join prods pr on pr.rn = p.product_pick;
EOF
  done=$(( done + remaining ))
  echo "  order_items: $done / $ITEMS_TOTAL"
done

echo
run_sql_db <<'EOF'
select
    (select count(*) from users) as users,
    (select count(*) from categories) as categories,
    (select count(*) from products) as products,
    (select count(*) from orders) as orders,
    (select count(*) from order_items) as order_items;
EOF
echo "Done."
