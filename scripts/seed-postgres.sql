-- Bulk random data generator for the dev Postgres seed schema.
-- Usage: psql ... -v n=10000 -f seed-postgres.sql
-- `n` controls scale: n users, n products, n orders, ~2n order_items.
-- Existing hand-written seed rows (docker/initdb/01_schema.sql) are left
-- alone — this only appends.

\set n :n
\if :{?n}
\else
    \set n 1000
\endif

insert into categories (name)
select 'Category ' || g
from generate_series(1, greatest(:n / 20, 5)) as g
on conflict (name) do nothing;

insert into users (email, full_name, is_active, metadata)
select
    'user_' || g || '_' || substr(md5(random()::text), 1, 8) || '@example.com',
    (array['Alex', 'Jordan', 'Taylor', 'Morgan', 'Casey', 'Riley', 'Jamie', 'Drew'])[1 + floor(random() * 8)]
        || ' ' ||
        (array['Smith', 'Johnson', 'Lee', 'Patel', 'Garcia', 'Kim', 'Nguyen', 'Brown'])[1 + floor(random() * 8)],
    random() > 0.15,
    jsonb_build_object(
        'plan', (array['free', 'pro', 'enterprise'])[1 + floor(random() * 3)],
        'signupSource', (array['organic', 'referral', 'ads', 'social'])[1 + floor(random() * 4)]
    )
from generate_series(1, :n) as g;

insert into products (category_id, sku, name, price_cents, in_stock)
select
    (select id from categories order by random() limit 1),
    'SKU-' || g || '-' || substr(md5(random()::text), 1, 6),
    (array['Widget', 'Gadget', 'Gizmo', 'Doohickey', 'Contraption', 'Thingamajig'])[1 + floor(random() * 6)]
        || ' ' || (array['Pro', 'Max', 'Mini', 'Plus', 'Lite', 'Ultra'])[1 + floor(random() * 6)],
    (random() * 49900 + 100)::int,
    random() > 0.1
from generate_series(1, :n) as g;

insert into orders (user_id, status, total_cents, notes)
select
    (select id from users order by random() limit 1),
    (array['pending', 'paid', 'shipped', 'cancelled'])[1 + floor(random() * 4)],
    (random() * 99900 + 500)::int,
    case when random() > 0.7 then 'Order note ' || g else null end
from generate_series(1, :n) as g;

insert into order_items (order_id, product_id, quantity, unit_price_cents)
select
    (select id from orders order by random() limit 1),
    (select id from products order by random() limit 1),
    (1 + floor(random() * 5))::int,
    (random() * 49900 + 100)::int
from generate_series(1, :n * 2) as g;

select
    (select count(*) from users) as users,
    (select count(*) from categories) as categories,
    (select count(*) from products) as products,
    (select count(*) from orders) as orders,
    (select count(*) from order_items) as order_items;
