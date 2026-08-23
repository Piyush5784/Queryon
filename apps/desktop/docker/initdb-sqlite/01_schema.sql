-- SQLite copy of docker/initdb/01_schema.sql. Not run against a Docker
-- container (SQLite is embedded, not client-server) — see
-- scripts/seed-sqlite.sh, which builds a fresh .db file from this script
-- via the `sqlite3` CLI. Differences from the Postgres original, all
-- confirmed by testing directly against a live SQLite file:
--   - `integer primary key autoincrement` instead of `bigint generated
--     always as identity` — SQLite's own auto-increment idiom, and the
--     only column type SQLite treats as an alias for the table's rowid.
--   - `text` instead of `jsonb` — no native JSON type at all; JSON is
--     just stored as text (SQLite's `json_*()` functions operate on text
--     columns directly, no separate binary JSON storage or type).
--   - `text not null default (datetime('now'))` instead of `timestamptz
--     not null default now()` — no timezone-aware timestamp type;
--     `datetime('now')` returns UTC text in `YYYY-MM-DD HH:MM:SS` form.
--   - `integer not null default 1` instead of `boolean ... default
--     true` — no native boolean type; 0/1 is the standard SQLite idiom,
--     matching how Beekeeper's own client binds booleans
--     (`_.isBoolean(p) ? Number(p) : p` in `sqlite.ts`).

create table users (
    id            integer primary key autoincrement,
    email         text not null unique,
    full_name     text not null,
    is_active     integer not null default 1,
    metadata      text not null default '{}',
    created_at    text not null default (datetime('now'))
);

create table categories (
    id    integer primary key autoincrement,
    name  text not null unique
);

create table products (
    id            integer primary key autoincrement,
    category_id   integer references categories(id) on delete set null,
    sku           text not null unique,
    name          text not null,
    price_cents   integer not null check (price_cents >= 0),
    in_stock      integer not null default 1,
    created_at    text not null default (datetime('now'))
);

create index idx_products_category_id on products(category_id);

create table orders (
    id            integer primary key autoincrement,
    user_id       integer not null references users(id) on delete cascade,
    status        text not null default 'pending'
                    check (status in ('pending', 'paid', 'shipped', 'cancelled')),
    total_cents   integer not null default 0,
    notes         text,
    created_at    text not null default (datetime('now'))
);

create index idx_orders_user_id on orders(user_id);

create table order_items (
    id            integer primary key autoincrement,
    order_id      integer not null references orders(id) on delete cascade,
    product_id    integer not null references products(id) on delete restrict,
    quantity      integer not null check (quantity > 0),
    unit_price_cents integer not null
);

create index idx_order_items_order_id on order_items(order_id);
create index idx_order_items_product_id on order_items(product_id);

insert into users (email, full_name, is_active, metadata) values
    ('alice@example.com', 'Alice Nguyen', 1, '{"plan": "pro"}'),
    ('bob@example.com',   'Bob Martinez', 1, '{"plan": "free"}'),
    ('carol@example.com', 'Carol Singh',  0, '{"plan": "free"}'),
    ('dave@example.com',  'Dave Okafor',  1, '{"plan": "pro"}');

insert into categories (name) values
    ('Electronics'), ('Books'), ('Home & Kitchen');

insert into products (category_id, sku, name, price_cents, in_stock) values
    (1, 'ELEC-001', 'Wireless Mouse', 2499, 1),
    (1, 'ELEC-002', 'Mechanical Keyboard', 8999, 1),
    (2, 'BOOK-001', 'The Pragmatic Programmer', 3999, 1),
    (3, 'HOME-001', 'French Press', 1899, 0);

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
