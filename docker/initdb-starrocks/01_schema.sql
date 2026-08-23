-- StarRocks copy of docker/initdb/01_schema.sql. Differences, all
-- confirmed by testing directly against a live
-- starrocks/allin1-ubuntu:latest container:
--   - Every table needs an explicit `PRIMARY KEY(...) DISTRIBUTED BY
--     HASH(...)` clause plain MySQL doesn't have.
--   - No foreign key support at all, inline or via ALTER TABLE
--     ("Unexpected input '('" / "No viable statement"). FK columns are
--     kept as plain columns with no referential enforcement.
--   - No CHECK constraint support ("Unexpected input 'check'").
--   - `AUTO_INCREMENT` columns must be `BIGINT`, never `INT` ("The
--     AUTO_INCREMENT column must be BIGINT").
--   - Every literal DEFAULT value must be a quoted string, not a bare
--     literal — confirmed for both booleans ("Unsupported expr
--     BoolLiteral(...) for default value") and plain integers
--     ("Unsupported expr IntLiteral(...) for default value"). `default
--     '1'`/`default '0'` work; `default true`/`default 0` don't.
--   - `json` is a real native type (unlike SQL Server/GreengageDB) — no
--     substitution needed there.
--   - Schema-change DDL (CREATE INDEX included) is asynchronous — the
--     next DDL statement on the same table fails if the previous one's
--     background job hasn't finished yet ("A schema change operation is
--     in progress"), confirmed live when the two `CREATE INDEX` statements
--     on order_items ran back-to-back in this same file. This file is not
--     run directly by a plain mysql client for that reason — see
--     scripts/seed-starrocks.sh, which loads it statement-by-statement
--     with a wait for each schema change to finish in between.
--   - No explicit `id` values on insert, unlike every other engine's
--     01_schema.sql — a PRIMARY KEY table's auto_increment always starts
--     counting from 1 on the table's first-ever insert regardless of
--     what ids a *later* insert asks for explicitly, and PRIMARY KEY
--     tables upsert (silently replace) on a colliding id rather than
--     erroring. Confirmed live: hand-seeding rows with explicit id=1..4
--     here, then bulk-seeding more with scripts/seed-starrocks.sh (which
--     also starts its own auto-increment ids at 1, since it's the same
--     table), silently overwrote every hand-seeded row. Letting
--     auto_increment assign every id and referencing rows by their
--     known column values (e.g. category name) instead of a hardcoded
--     id sidesteps this entirely.

create database devdb;
use devdb;

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

create table categories (
    id    bigint not null auto_increment,
    name  varchar(255) not null
)
primary key (id)
distributed by hash (id);

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

create index idx_products_category_id on products(category_id);

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

create index idx_orders_user_id on orders(user_id);

create table order_items (
    id            bigint not null auto_increment,
    order_id      bigint not null,
    product_id    bigint not null,
    quantity      int not null,
    unit_price_cents int not null
)
primary key (id)
distributed by hash (id);

create index idx_order_items_order_id on order_items(order_id);
create index idx_order_items_product_id on order_items(product_id);

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
select o.id, p.id, 1, 2499
from orders o, products p
where o.total_cents = 11498 and p.sku = 'ELEC-001'
union all
select o.id, p.id, 1, 8999
from orders o, products p
where o.total_cents = 11498 and p.sku = 'ELEC-002'
union all
select o.id, p.id, 1, 3999
from orders o, products p
where o.total_cents = 3999 and p.sku = 'BOOK-001'
union all
select o.id, p.id, 1, 2499
from orders o, products p
where o.total_cents = 2499 and p.sku = 'ELEC-001'
union all
select o.id, p.id, 1, 1899
from orders o, products p
where o.total_cents = 1899 and p.sku = 'HOME-001';
