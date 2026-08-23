-- DuckDB copy of docker/initdb/01_schema.sql. Not run against a Docker
-- container the way client-server engines are — DuckDB is embedded, not
-- client-server (like SQLite) — see scripts/seed-duckdb.sh, which builds
-- a fresh .duckdb file from this script via a `duckdb` CLI container.
-- Differences from the Postgres original, all confirmed live against
-- DuckDB (via the `davidgasquez/duckdb` CLI image):
--   - No `AUTO_INCREMENT`/`SERIAL`/`GENERATED ALWAYS AS IDENTITY` at all
--     (the latter is flatly rejected: "Constraint not implemented!") —
--     each table gets its own `CREATE SEQUENCE ..._id_seq` plus `id
--     BIGINT PRIMARY KEY DEFAULT nextval('..._id_seq')`.
--   - `JSON` and `BOOLEAN` are both real native types (unlike SQLite),
--     so `metadata jsonb`/`is_active boolean` port over almost as-is —
--     just `jsonb` -> `json` (no distinct binary-JSON storage type here).
--   - `TIMESTAMP DEFAULT now()` instead of `timestamptz ... default
--     now()` — DuckDB's `TIMESTAMP` has no separate timezone-aware
--     variant in play here.
--   - Foreign keys and CHECK constraints are both real and enforced
--     (confirmed live) — this schema keeps them, unlike StarRocks/
--     ClickHouse's schema files, which drop them entirely.

create sequence users_id_seq;
create table users (
    id            bigint primary key default nextval('users_id_seq'),
    email         varchar not null unique,
    full_name     varchar not null,
    is_active     boolean not null default true,
    metadata      json not null default '{}',
    created_at    timestamp not null default now()
);

create sequence categories_id_seq;
create table categories (
    id    bigint primary key default nextval('categories_id_seq'),
    name  varchar not null unique
);

create sequence products_id_seq;
create table products (
    id            bigint primary key default nextval('products_id_seq'),
    category_id   bigint references categories(id),
    sku           varchar not null unique,
    name          varchar not null,
    price_cents   integer not null check (price_cents >= 0),
    in_stock      boolean not null default true,
    created_at    timestamp not null default now()
);

create index idx_products_category_id on products(category_id);

create sequence orders_id_seq;
create table orders (
    id            bigint primary key default nextval('orders_id_seq'),
    user_id       bigint not null references users(id),
    status        varchar not null default 'pending'
                    check (status in ('pending', 'paid', 'shipped', 'cancelled')),
    total_cents   integer not null default 0,
    notes         varchar,
    created_at    timestamp not null default now()
);

create index idx_orders_user_id on orders(user_id);

create sequence order_items_id_seq;
create table order_items (
    id                bigint primary key default nextval('order_items_id_seq'),
    order_id          bigint not null references orders(id),
    product_id        bigint not null references products(id),
    quantity          integer not null check (quantity > 0),
    unit_price_cents  integer not null
);

create index idx_order_items_order_id on order_items(order_id);
create index idx_order_items_product_id on order_items(product_id);

insert into users (email, full_name, is_active, metadata) values
    ('alice@example.com', 'Alice Nguyen', true,  '{"plan": "pro"}'),
    ('bob@example.com',   'Bob Martinez', true,  '{"plan": "free"}'),
    ('carol@example.com', 'Carol Singh',  false, '{"plan": "free"}'),
    ('dave@example.com',  'Dave Okafor',  true,  '{"plan": "pro"}');

insert into categories (name) values
    ('Electronics'), ('Books'), ('Home & Kitchen');

insert into products (category_id, sku, name, price_cents, in_stock) values
    (1, 'ELEC-001', 'Wireless Mouse', 2499, true),
    (1, 'ELEC-002', 'Mechanical Keyboard', 8999, true),
    (2, 'BOOK-001', 'The Pragmatic Programmer', 3999, true),
    (3, 'HOME-001', 'French Press', 1899, false);

insert into orders (user_id, status, total_cents, notes) values
    (1, 'paid', 11498, null),
    (1, 'shipped', 3999, 'Gift wrap requested'),
    (2, 'pending', 2499, null),
    (4, 'cancelled', 1899, 'Customer changed mind');

insert into order_items (order_id, product_id, quantity, unit_price_cents) values
    (1, 1, 1, 2499),
    (1, 2, 1, 8999),
    (2, 3, 1, 3999),
    (3, 1, 1, 2499),
    (4, 4, 1, 1899);
