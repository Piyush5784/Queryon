
set @n = coalesce(@n, 1000);
set @batch_size = 500;
set session max_recursive_iterations = 2000000;

drop temporary table if exists _seed_numbers;
create temporary table _seed_numbers (i int primary key);

set @max_n = greatest(@n * 2, @n);
insert into _seed_numbers (i)
with recursive seq(i) as (
    select 1
    union all
    select i + 1 from seq where i < @max_n
)
select i from seq;

insert into categories (name)
select concat('Category ', i) from _seed_numbers where i <= greatest(@n div 20, 5)
on duplicate key update name = name;

drop procedure if exists _seed_users;
delimiter $$
create procedure _seed_users(in total int, in batch_size int)
begin
    declare done int default 0;
    declare remaining int;
    seed_loop: loop
        if done >= total then
            leave seed_loop;
        end if;
        set remaining = least(batch_size, total - done);
        start transaction;
        insert into users (email, full_name, is_active, metadata)
        select
            concat('user_', done + i, '_', substring(md5(rand()), 1, 8), '@example.com'),
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
        from _seed_numbers
        where i <= remaining;
        commit;
        set done = done + remaining;
    end loop;
end$$
delimiter ;

call _seed_users(@n, @batch_size);
drop procedure _seed_users;

-- Row-numbered lookup tables for materialize-then-join FK picks.
drop temporary table if exists _cat_ids;
create temporary table _cat_ids as select row_number() over (order by id) as rn, id from categories;
alter table _cat_ids add primary key (rn);
set @cat_count = (select count(*) from _cat_ids);

drop procedure if exists _seed_products;
delimiter $$
create procedure _seed_products(in total int, in batch_size int, in cat_count int)
begin
    declare done int default 0;
    declare remaining int;
    seed_loop: loop
        if done >= total then
            leave seed_loop;
        end if;
        set remaining = least(batch_size, total - done);

        drop temporary table if exists _product_picks;
        create temporary table _product_picks as
        select i, 1 + floor(rand() * cat_count) as cat_pick
        from _seed_numbers
        where i <= remaining;

        start transaction;
        insert into products (category_id, sku, name, price_cents, in_stock)
        select
            c.id,
            concat('SKU-', done + p.i, '-', substring(md5(rand()), 1, 6)),
            concat(
                elt(1 + floor(rand() * 6), 'Widget', 'Gadget', 'Gizmo', 'Doohickey', 'Contraption', 'Thingamajig'),
                ' ',
                elt(1 + floor(rand() * 6), 'Pro', 'Max', 'Mini', 'Plus', 'Lite', 'Ultra')
            ),
            floor(rand() * 49900 + 100),
            rand() > 0.1
        from _product_picks p
        join _cat_ids c on c.rn = p.cat_pick;
        commit;
        set done = done + remaining;
    end loop;
end$$
delimiter ;

call _seed_products(@n, @batch_size, @cat_count);
drop procedure _seed_products;
drop temporary table if exists _product_picks;

drop temporary table if exists _user_ids;
create temporary table _user_ids as select row_number() over (order by id) as rn, id from users;
alter table _user_ids add primary key (rn);
set @user_count = (select count(*) from _user_ids);

drop procedure if exists _seed_orders;
delimiter $$
create procedure _seed_orders(in total int, in batch_size int, in user_count int)
begin
    declare done int default 0;
    declare remaining int;
    seed_loop: loop
        if done >= total then
            leave seed_loop;
        end if;
        set remaining = least(batch_size, total - done);

        drop temporary table if exists _order_picks;
        create temporary table _order_picks as
        select i, 1 + floor(rand() * user_count) as user_pick
        from _seed_numbers
        where i <= remaining;

        start transaction;
        insert into orders (user_id, status, total_cents, notes)
        select
            u.id,
            elt(1 + floor(rand() * 4), 'pending', 'paid', 'shipped', 'cancelled'),
            floor(rand() * 99900 + 500),
            case when rand() > 0.7 then concat('Order note ', done + o.i) else null end
        from _order_picks o
        join _user_ids u on u.rn = o.user_pick;
        commit;
        set done = done + remaining;
    end loop;
end$$
delimiter ;

call _seed_orders(@n, @batch_size, @user_count);
drop procedure _seed_orders;
drop temporary table if exists _order_picks;

drop temporary table if exists _order_ids;
create temporary table _order_ids as select row_number() over (order by id) as rn, id from orders;
alter table _order_ids add primary key (rn);
set @order_count = (select count(*) from _order_ids);

drop temporary table if exists _product_ids;
create temporary table _product_ids as select row_number() over (order by id) as rn, id from products;
alter table _product_ids add primary key (rn);
set @product_count = (select count(*) from _product_ids);

drop procedure if exists _seed_order_items;
delimiter $$
create procedure _seed_order_items(in total int, in batch_size int, in order_count int, in product_count int)
begin
    declare done int default 0;
    declare remaining int;
    seed_loop: loop
        if done >= total then
            leave seed_loop;
        end if;
        set remaining = least(batch_size, total - done);

        drop temporary table if exists _item_picks;
        create temporary table _item_picks as
        select
            i,
            1 + floor(rand() * order_count) as order_pick,
            1 + floor(rand() * product_count) as product_pick
        from _seed_numbers
        where i <= remaining;

        start transaction;
        insert into order_items (order_id, product_id, quantity, unit_price_cents)
        select
            o.id,
            p.id,
            floor(1 + rand() * 5),
            floor(rand() * 49900 + 100)
        from _item_picks ip
        join _order_ids o on o.rn = ip.order_pick
        join _product_ids p on p.rn = ip.product_pick;
        commit;
        set done = done + remaining;
    end loop;
end$$
delimiter ;

call _seed_order_items(@n * 2, @batch_size, @order_count, @product_count);
drop procedure _seed_order_items;
drop temporary table if exists _item_picks;

select
    (select count(*) from users) as users,
    (select count(*) from categories) as categories,
    (select count(*) from products) as products,
    (select count(*) from orders) as orders,
    (select count(*) from order_items) as order_items;
