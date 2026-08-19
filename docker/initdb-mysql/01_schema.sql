-- MySQL-flavored mirror of docker/initdb/01_schema.sql — same table/column
-- shapes so the same kind of integration-test assertions apply to both
-- engines.

create table users (
    id            bigint auto_increment primary key,
    email         varchar(255) not null unique,
    full_name     varchar(255) not null,
    is_active     boolean not null default true,
    metadata      json not null,
    created_at    timestamp not null default current_timestamp
);

create table categories (
    id    bigint auto_increment primary key,
    name  varchar(255) not null unique
);

create table products (
    id            bigint auto_increment primary key,
    category_id   bigint,
    sku           varchar(255) not null unique,
    name          varchar(255) not null,
    price_cents   int not null check (price_cents >= 0),
    in_stock      boolean not null default true,
    created_at    timestamp not null default current_timestamp,
    foreign key (category_id) references categories(id) on delete set null
);

create index idx_products_category_id on products(category_id);

create table orders (
    id            bigint auto_increment primary key,
    user_id       bigint not null,
    status        varchar(20) not null default 'pending'
                    check (status in ('pending', 'paid', 'shipped', 'cancelled')),
    total_cents   int not null default 0,
    notes         text,
    created_at    timestamp not null default current_timestamp,
    foreign key (user_id) references users(id) on delete cascade
);

create index idx_orders_user_id on orders(user_id);

create table order_items (
    id                bigint auto_increment primary key,
    order_id          bigint not null,
    product_id        bigint not null,
    quantity          int not null check (quantity > 0),
    unit_price_cents  int not null,
    foreign key (order_id) references orders(id) on delete cascade,
    foreign key (product_id) references products(id) on delete restrict
);

create index idx_order_items_order_id on order_items(order_id);
create index idx_order_items_product_id on order_items(product_id);

-- Sample data --------------------------------------------------------

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
