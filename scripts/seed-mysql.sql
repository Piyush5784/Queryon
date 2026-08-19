-- Bulk random data generator for the dev MySQL seed schema.
-- Usage: see scripts/seed.sh (sets @n then pipes this file in).
-- `@n` controls scale: n users, n products, n orders, ~2n order_items.
-- Existing hand-written seed rows (docker/initdb-mysql/01_schema.sql) are
-- left alone — this only appends.
--
-- MySQL doesn't allow `WITH RECURSIVE ... INSERT INTO ... SELECT` at the
-- top level (unlike Postgres) — the CTE has to be wrapped as a derived
-- table inside the SELECT that feeds the INSERT.

set @n = coalesce(@n, 1000);
set session cte_max_recursion_depth = 2000000;

insert into categories (name)
select concat('Category ', i) from (
    with recursive seq(i) as (
        select 1
        union all
        select i + 1 from seq where i < greatest(@n div 20, 5)
    )
    select i from seq
) as s
on duplicate key update name = name;

insert into users (email, full_name, is_active, metadata)
select
    concat('user_', i, '_', substring(md5(rand()), 1, 8), '@example.com'),
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
    with recursive seq(i) as (
        select 1
        union all
        select i + 1 from seq where i < @n
    )
    select i from seq
) as s;

insert into products (category_id, sku, name, price_cents, in_stock)
select
    (select id from categories order by rand() limit 1),
    concat('SKU-', i, '-', substring(md5(rand()), 1, 6)),
    concat(
        elt(1 + floor(rand() * 6), 'Widget', 'Gadget', 'Gizmo', 'Doohickey', 'Contraption', 'Thingamajig'),
        ' ',
        elt(1 + floor(rand() * 6), 'Pro', 'Max', 'Mini', 'Plus', 'Lite', 'Ultra')
    ),
    floor(rand() * 49900 + 100),
    rand() > 0.1
from (
    with recursive seq(i) as (
        select 1
        union all
        select i + 1 from seq where i < @n
    )
    select i from seq
) as s;

insert into orders (user_id, status, total_cents, notes)
select
    (select id from users order by rand() limit 1),
    elt(1 + floor(rand() * 4), 'pending', 'paid', 'shipped', 'cancelled'),
    floor(rand() * 99900 + 500),
    case when rand() > 0.7 then concat('Order note ', i) else null end
from (
    with recursive seq(i) as (
        select 1
        union all
        select i + 1 from seq where i < @n
    )
    select i from seq
) as s;

insert into order_items (order_id, product_id, quantity, unit_price_cents)
select
    (select id from orders order by rand() limit 1),
    (select id from products order by rand() limit 1),
    floor(1 + rand() * 5),
    floor(rand() * 49900 + 100)
from (
    with recursive seq(i) as (
        select 1
        union all
        select i + 1 from seq where i < @n * 2
    )
    select i from seq
) as s;

select
    (select count(*) from users) as users,
    (select count(*) from categories) as categories,
    (select count(*) from products) as products,
    (select count(*) from orders) as orders,
    (select count(*) from order_items) as order_items;
