-- Bulk random data generator for the dev Postgres seed schema.
-- Usage: psql ... -v n=10000 -f seed-postgres.sql
-- `n` controls scale: n users, n products, n orders, ~2n order_items.
-- Existing hand-written seed rows (docker/initdb/01_schema.sql) are left
-- alone — this only appends.
--
-- Inserted in chunks of `batch_size` rows, each batch committed as its
-- own transaction (via COMMIT inside the DO block, PG11+), so a
-- kill/timeout only loses the in-flight batch and lock hold-time per
-- statement stays short. Foreign keys are picked from a pre-materialized
-- id table instead of `order by random() limit 1` per row — that pattern
-- re-sorts the whole referenced table on every single row and is the
-- actual cost driver at scale, not the row count itself.
--
-- psql's `:var` substitution does not happen inside dollar-quoted ($$)
-- function bodies, so `n`/`batch_size` are passed into each DO block via
-- this config table instead of `:n` directly.

\set n :n
\if :{?n}
\else
    \set n 1000
\endif

\set batch_size 500

create temp table _seed_config as select :n::int as n, :batch_size::int as batch_size;

insert into categories (name)
select 'Category ' || g
from generate_series(1, greatest(:n / 20, 5)) as g
on conflict (name) do nothing;

do $$
declare
    batch_size int := (select s.batch_size from _seed_config s);
    total int := (select s.n from _seed_config s);
    done int := 0;
    remaining int;
begin
    while done < total loop
        remaining := least(batch_size, total - done);
        insert into users (email, full_name, is_active, metadata)
        select
            'user_' || (done + g) || '_' || substr(md5(random()::text), 1, 8) || '@example.com',
            (array['Alex', 'Jordan', 'Taylor', 'Morgan', 'Casey', 'Riley', 'Jamie', 'Drew'])[1 + floor(random() * 8)]
                || ' ' ||
                (array['Smith', 'Johnson', 'Lee', 'Patel', 'Garcia', 'Kim', 'Nguyen', 'Brown'])[1 + floor(random() * 8)],
            random() > 0.15,
            jsonb_build_object(
                'plan', (array['free', 'pro', 'enterprise'])[1 + floor(random() * 3)],
                'signupSource', (array['organic', 'referral', 'ads', 'social'])[1 + floor(random() * 4)]
            )
        from generate_series(1, remaining) as g;
        done := done + remaining;
        commit;
    end loop;
end $$;

create temp table _cat_ids as select id from categories;

do $$
declare
    batch_size int := (select s.batch_size from _seed_config s);
    total int := (select s.n from _seed_config s);
    done int := 0;
    remaining int;
    cat_count int := (select count(*) from _cat_ids);
begin
    while done < total loop
        remaining := least(batch_size, total - done);
        insert into products (category_id, sku, name, price_cents, in_stock)
        select
            (select id from _cat_ids offset floor(random() * cat_count) limit 1),
            'SKU-' || (done + g) || '-' || substr(md5(random()::text), 1, 6),
            (array['Widget', 'Gadget', 'Gizmo', 'Doohickey', 'Contraption', 'Thingamajig'])[1 + floor(random() * 6)]
                || ' ' || (array['Pro', 'Max', 'Mini', 'Plus', 'Lite', 'Ultra'])[1 + floor(random() * 6)],
            (random() * 49900 + 100)::int,
            random() > 0.1
        from generate_series(1, remaining) as g;
        done := done + remaining;
        commit;
    end loop;
end $$;

create temp table _user_ids as select id from users;
create temp table _product_ids as select id from products;

do $$
declare
    batch_size int := (select s.batch_size from _seed_config s);
    total int := (select s.n from _seed_config s);
    done int := 0;
    remaining int;
    user_count int := (select count(*) from _user_ids);
begin
    while done < total loop
        remaining := least(batch_size, total - done);
        insert into orders (user_id, status, total_cents, notes)
        select
            (select id from _user_ids offset floor(random() * user_count) limit 1),
            (array['pending', 'paid', 'shipped', 'cancelled'])[1 + floor(random() * 4)],
            (random() * 99900 + 500)::int,
            case when random() > 0.7 then 'Order note ' || (done + g) else null end
        from generate_series(1, remaining) as g;
        done := done + remaining;
        commit;
    end loop;
end $$;

create temp table _order_ids as select id from orders;

do $$
declare
    batch_size int := (select s.batch_size from _seed_config s);
    total int := (select s.n from _seed_config s) * 2;
    done int := 0;
    remaining int;
    order_count int := (select count(*) from _order_ids);
    product_count int := (select count(*) from _product_ids);
begin
    while done < total loop
        remaining := least(batch_size, total - done);
        insert into order_items (order_id, product_id, quantity, unit_price_cents)
        select
            (select id from _order_ids offset floor(random() * order_count) limit 1),
            (select id from _product_ids offset floor(random() * product_count) limit 1),
            (1 + floor(random() * 5))::int,
            (random() * 49900 + 100)::int
        from generate_series(1, remaining) as g;
        done := done + remaining;
        commit;
    end loop;
end $$;

select
    (select count(*) from users) as users,
    (select count(*) from categories) as categories,
    (select count(*) from products) as products,
    (select count(*) from orders) as orders,
    (select count(*) from order_items) as order_items;
