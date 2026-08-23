-- SQL Server copy of docker/initdb/01_schema.sql. Differences, all
-- confirmed by testing directly against a live
-- mcr.microsoft.com/mssql/server:2022-latest container:
--   - `int identity(1,1) primary key` instead of `bigint generated
--     always as identity` — SQL Server's own auto-increment idiom.
--   - `nvarchar(max)` instead of `jsonb` — no native JSON type until SQL
--     Server 2025+; JSON is conventionally stored as text and queried
--     via ISJSON()/JSON_VALUE(), confirmed this 2022 image has no `json`
--     type at all ("Cannot find data type json").
--   - `bit` instead of `boolean` — SQL Server's boolean type, 0/1.
--   - `datetime2` instead of `timestamptz` — no timezone-aware type;
--     `sysutcdatetime()` is the UTC-now equivalent of Postgres's `now()`.
--   - FK actions use `no action`, not `restrict` — SQL Server has no
--     `RESTRICT` keyword for `ON UPDATE`/`ON DELETE` at all.

create database devdb;
go
use devdb;
go

create table users (
    id            int identity(1,1) primary key,
    email         nvarchar(255) not null unique,
    full_name     nvarchar(255) not null,
    is_active     bit not null default 1,
    metadata      nvarchar(max) not null default '{}',
    created_at    datetime2 not null default sysutcdatetime()
);
go

create table categories (
    id    int identity(1,1) primary key,
    name  nvarchar(255) not null unique
);
go

create table products (
    id            int identity(1,1) primary key,
    category_id   int references categories(id) on delete set null,
    sku           nvarchar(100) not null unique,
    name          nvarchar(255) not null,
    price_cents   int not null check (price_cents >= 0),
    in_stock      bit not null default 1,
    created_at    datetime2 not null default sysutcdatetime()
);
go

create index idx_products_category_id on products(category_id);
go

create table orders (
    id            int identity(1,1) primary key,
    user_id       int not null references users(id) on delete cascade,
    status        nvarchar(20) not null default 'pending'
                    check (status in ('pending', 'paid', 'shipped', 'cancelled')),
    total_cents   int not null default 0,
    notes         nvarchar(max),
    created_at    datetime2 not null default sysutcdatetime()
);
go

create index idx_orders_user_id on orders(user_id);
go

create table order_items (
    id            int identity(1,1) primary key,
    order_id      int not null references orders(id) on delete cascade,
    product_id    int not null references products(id) on delete no action,
    quantity      int not null check (quantity > 0),
    unit_price_cents int not null
);
go

create index idx_order_items_order_id on order_items(order_id);
create index idx_order_items_product_id on order_items(product_id);
go

insert into users (email, full_name, is_active, metadata) values
    ('alice@example.com', 'Alice Nguyen', 1, '{"plan": "pro"}'),
    ('bob@example.com',   'Bob Martinez', 1, '{"plan": "free"}'),
    ('carol@example.com', 'Carol Singh',  0, '{"plan": "free"}'),
    ('dave@example.com',  'Dave Okafor',  1, '{"plan": "pro"}');
go

insert into categories (name) values
    ('Electronics'), ('Books'), ('Home & Kitchen');
go

insert into products (category_id, sku, name, price_cents, in_stock) values
    (1, 'ELEC-001', 'Wireless Mouse', 2499, 1),
    (1, 'ELEC-002', 'Mechanical Keyboard', 8999, 1),
    (2, 'BOOK-001', 'The Pragmatic Programmer', 3999, 1),
    (3, 'HOME-001', 'French Press', 1899, 0);
go

insert into orders (user_id, status, total_cents, notes) values
    (1, 'paid', 11498, null),
    (1, 'shipped', 3999, 'Gift wrap requested'),
    (2, 'pending', 2499, null),
    (4, 'cancelled', 1899, 'Customer changed mind');
go

insert into order_items (order_id, product_id, quantity, unit_price_cents) values
    (1, 1, 1, 2499),
    (1, 2, 1, 8999),
    (2, 3, 1, 3999),
    (3, 1, 1, 2499),
    (4, 4, 1, 1899);
go
