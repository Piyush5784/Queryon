-- ClickHouse dialect of the shared dev schema (docker/initdb/01_schema.sql).
-- ClickHouse is a columnar OLAP store, not a transactional relational
-- engine — every table here uses MergeTree, the general-purpose engine,
-- and diverges from the Postgres original in ways confirmed live against
-- clickhouse/clickhouse-server: no CREATE TABLE without an explicit
-- ENGINE clause; no AUTO_INCREMENT concept (ids are assigned explicitly);
-- no FOREIGN KEY enforcement at all (the syntax parses but is silently
-- dropped, never stored or checked); CHECK constraints ARE real and
-- enforced, unlike foreign keys; ORDER BY stands in for what a primary
-- key/index normally does. Load this by hand through
-- `clickhouse-client --multiquery` (not the HTTP interface — it rejects
-- more than one `;`-separated statement per request, confirmed live):
--   docker exec -i queryon-clickhouse clickhouse-client --user devuser \
--     --password devpass --database devdb --multiquery \
--     < docker/initdb-clickhouse/01_schema.sql
-- The official image's own docker-entrypoint-initdb.d convention was
-- tried first but confirmed live to silently do nothing with this file
-- (no tables appear, no error in the container logs either) — see
-- docker-compose.yml's clickhouse service comment. Then run
-- scripts/seed-clickhouse.sh for bulk rows.

create table users (
    id          UInt64,
    email       String,
    full_name   String,
    is_active   UInt8 default 1,
    metadata    String default '{}',
    created_at  DateTime default now()
) engine = MergeTree() order by id;

create table categories (
    id    UInt64,
    name  String
) engine = MergeTree() order by id;

create table products (
    id            UInt64,
    category_id   Nullable(UInt64),
    sku           String,
    name          String,
    price_cents   Int32,
    in_stock      UInt8 default 1,
    created_at    DateTime default now(),
    constraint chk_price_nonneg check price_cents >= 0
) engine = MergeTree() order by id;

create table orders (
    id            UInt64,
    user_id       UInt64,
    status        String default 'pending',
    total_cents   Int32 default 0,
    notes         Nullable(String),
    created_at    DateTime default now()
) engine = MergeTree() order by id;

create table order_items (
    id                UInt64,
    order_id          UInt64,
    product_id        UInt64,
    quantity          Int32,
    unit_price_cents  Int32,
    constraint chk_quantity_positive check quantity > 0
) engine = MergeTree() order by id;

-- Sample data --------------------------------------------------------

insert into users (id, email, full_name, is_active, metadata) values
    (1, 'alice@example.com', 'Alice Nguyen', 1, '{"plan": "pro"}'),
    (2, 'bob@example.com',   'Bob Martinez', 1, '{"plan": "free"}'),
    (3, 'carol@example.com', 'Carol Singh',  0, '{"plan": "free"}'),
    (4, 'dave@example.com',  'Dave Okafor',  1, '{"plan": "pro"}');

insert into categories (id, name) values
    (1, 'Electronics'), (2, 'Books'), (3, 'Home & Kitchen');

insert into products (id, category_id, sku, name, price_cents, in_stock) values
    (1, 1, 'ELEC-001', 'Wireless Mouse', 2499, 1),
    (2, 1, 'ELEC-002', 'Mechanical Keyboard', 8999, 1),
    (3, 2, 'BOOK-001', 'The Pragmatic Programmer', 3999, 1),
    (4, 3, 'HOME-001', 'French Press', 1899, 0);

insert into orders (id, user_id, status, total_cents, notes) values
    (1, 1, 'paid', 11498, null),
    (2, 1, 'shipped', 3999, 'Gift wrap requested'),
    (3, 2, 'pending', 2499, null),
    (4, 4, 'cancelled', 1899, 'Customer changed mind');

insert into order_items (id, order_id, product_id, quantity, unit_price_cents) values
    (1, 1, 1, 1, 2499),
    (2, 1, 2, 1, 8999),
    (3, 2, 3, 1, 3999),
    (4, 3, 1, 1, 2499),
    (5, 4, 4, 1, 1899);
