-- Bulk dev data for ClickHouse (queryon-clickhouse). Run through
-- scripts/seed-clickhouse.sh, which passes {n:UInt64} in via
-- `clickhouse-client --param_n=<N>` — ClickHouse has no session-variable
-- syntax like MySQL's `SET @n = ...` (confirmed live: `SET @n = 5`
-- fails, "Expected one of: ROLE DEFAULT, ..."), but its CLI query
-- parameter substitution ({n:UInt64}) works.
--
-- Each insert reads its own `max(id)` as a `WITH ... AS (subquery)`
-- bound once before the SELECT runs, not inline per-row — an inline
-- `(select max(id) from t) + number` re-evaluates the subquery
-- concurrently with the insert itself and produces duplicate/skipped
-- ids (confirmed live: a naive version produced a duplicate id and a
-- gap). Binding it once via WITH avoids that race.
--
-- Appends on top of whatever docker/initdb-clickhouse/01_schema.sql
-- already seeded; safe to run more than once.

insert into categories (id, name)
with (select max(id) from categories) as base_id
select
    base_id + number + 1 as id,
    concat('Category ', toString(base_id + number + 1)) as name
from numbers(greatest(intDiv({n:UInt64}, 20), 1));

insert into users (id, email, full_name, is_active, metadata)
with (select max(id) from users) as base_id
select
    base_id + number + 1 as id,
    concat('user', toString(base_id + number + 1), '@example.com') as email,
    concat('User ', toString(base_id + number + 1)) as full_name,
    if(rand() % 10 = 0, 0, 1) as is_active,
    if(rand() % 2 = 0, '{"plan": "pro"}', '{"plan": "free"}') as metadata
from numbers({n:UInt64});

insert into products (id, category_id, sku, name, price_cents, in_stock)
with
    (select max(id) from products) as base_id,
    (select min(id) from categories) as min_cat,
    (select count() from categories) as cat_count
select
    base_id + number + 1 as id,
    min_cat + (rand() % cat_count) as category_id,
    concat('SKU-', toString(base_id + number + 1)) as sku,
    concat('Product ', toString(base_id + number + 1)) as name,
    500 + (rand() % 20000) as price_cents,
    if(rand() % 8 = 0, 0, 1) as in_stock
from numbers({n:UInt64});

insert into orders (id, user_id, status, total_cents, notes)
with
    (select max(id) from orders) as base_id,
    (select min(id) from users) as min_user,
    (select count() from users) as user_count
select
    base_id + number + 1 as id,
    min_user + (rand() % user_count) as user_id,
    arrayElement(['pending', 'paid', 'shipped', 'cancelled'], (rand() % 4) + 1) as status,
    1000 + (rand() % 30000) as total_cents,
    if(rand() % 3 = 0, 'Gift wrap requested', null) as notes
from numbers({n:UInt64});

insert into order_items (id, order_id, product_id, quantity, unit_price_cents)
with
    (select max(id) from order_items) as base_id,
    (select min(id) from orders) as min_order,
    (select count() from orders) as order_count,
    (select min(id) from products) as min_product,
    (select count() from products) as product_count
select
    base_id + number + 1 as id,
    min_order + (rand() % order_count) as order_id,
    min_product + (rand() % product_count) as product_id,
    1 + (rand() % 5) as quantity,
    500 + (rand() % 20000) as unit_price_cents
from numbers({n:UInt64} * 2);
