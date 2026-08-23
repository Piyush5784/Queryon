-- Bulk random data generator for the dev SQL Server seed schema
-- (docker/initdb-mssql/01_schema.sql). Usage: sqlcmd ... -v n=10000 -i
-- seed-mssql.sql. `n` controls scale: n users, n products, n orders,
-- ~2n order_items. Existing hand-written seed rows are left alone — this
-- only appends.
--
-- T-SQL has no array-literal type, so the array-indexed name-pool idiom
-- seed-postgres.sql/seed-cockroachdb.sh use becomes a CASE expression
-- over ABS(CHECKSUM(NEWID())) % n instead — confirmed live this
-- distributes the same way. WHILE + BEGIN TRANSACTION/COMMIT per batch
-- works exactly like Postgres's DO $$ ... $$ loop (confirmed live,
-- unlike SQLite where DDL/PRAGMA restrictions rule this style out
-- entirely — see seed-sqlite.sh's doc comment).
--
-- Same materialize-then-join fix as every other engine's seed script:
-- a correlated `(SELECT TOP 1 id FROM t ORDER BY NEWID())` scalar
-- subquery is evaluated once per *statement* here too, not once per row
-- (confirmed live — every row in a batch got the same picked id), so FK
-- picks are generated via ROW_NUMBER()-numbered temp tables joined
-- against a batch of random picks instead.
--
-- The row-number generator itself went through two versions: the first
-- used `sys.all_objects a CROSS JOIN sys.all_objects b` (a common T-SQL
-- idiom for "just give me a big number sequence"), but with ~2600 rows
-- in `sys.all_objects` on this image that cross join materializes ~7
-- million rows before `TOP` trims it, and `ROW_NUMBER() OVER (ORDER BY
-- (SELECT NULL))` over that forces a real sort/spool — confirmed live,
-- the order_items batch (up to 2400 target rows) never finished after
-- several minutes of genuine CPU-bound work (`sys.dm_exec_requests`
-- showed it actually running, not blocked). A recursive CTE counting up
-- to @remaining is the fix: no cross join, no giant intermediate set,
-- and `OPTION (MAXRECURSION 0)` (unbounded) on the final INSERT lifts
-- the default 100-row recursion cap.
--
-- A second, separate pathology was found the same way, isolated by
-- testing with and without each part until only one factor remained:
-- a `picks` CTE containing `NEWID()`, joined against a `ROW_NUMBER()`-
-- numbered lookup table (even one materialized into a real `#temp` table
-- with a real clustered index on `rn`), hangs indefinitely — confirmed
-- reproducible from a completely fresh container, ruling out resource
-- contention. The exact same join with `NEWID()` replaced by a fixed
-- deterministic expression finishes in 0.25s, isolating `NEWID()`
-- inside a CTE that gets joined as the cause (SQL Server's optimizer
-- appears to mishandle this specific combination). The fix: materialize
-- `picks` into its own `#picks` temp table with a plain `SELECT ...
-- INTO` *before* joining it against anything, forcing `NEWID()` to be
-- evaluated exactly once per row up front rather than folded into the
-- join's plan — confirmed this brings the same join back to 0.23s.

:setvar batch_size "500"

use devdb;
go

declare @cat_total int = case when $(n) / 20 > 5 then $(n) / 20 else 5 end;
;with nums(g) as (
    select 1
    union all
    select g + 1 from nums where g < @cat_total
)
insert into categories (name)
select 'Category ' + cast(g as varchar(10)) from nums
option (maxrecursion 0);
go

declare @batch_size int = $(batch_size);
declare @total int = $(n);
declare @done int = 0;
declare @remaining int;

while @done < @total
begin
    set @remaining = case when @total - @done < @batch_size then @total - @done else @batch_size end;

    begin transaction;
    ;with nums(g) as (
        select 1
        union all
        select g + 1 from nums where g < @remaining
    )
    insert into users (email, full_name, is_active, metadata)
    select
        'user_' + cast(@done + g as varchar(10)) + '_' + left(cast(newid() as varchar(36)), 8) + '@example.com',
        (case abs(checksum(newid())) % 8
            when 0 then 'Alex' when 1 then 'Jordan' when 2 then 'Taylor' when 3 then 'Morgan'
            when 4 then 'Casey' when 5 then 'Riley' when 6 then 'Jamie' else 'Drew' end)
        + ' ' +
        (case abs(checksum(newid())) % 8
            when 0 then 'Smith' when 1 then 'Johnson' when 2 then 'Lee' when 3 then 'Patel'
            when 4 then 'Garcia' when 5 then 'Kim' when 6 then 'Nguyen' else 'Brown' end),
        case when abs(checksum(newid())) % 100 < 85 then 1 else 0 end,
        '{"plan": "' +
            (case abs(checksum(newid())) % 3 when 0 then 'free' when 1 then 'pro' else 'enterprise' end) +
        '", "signupSource": "' +
            (case abs(checksum(newid())) % 4 when 0 then 'organic' when 1 then 'referral' when 2 then 'ads' else 'social' end) +
        '"}'
    from nums
    option (maxrecursion 0);
    commit transaction;

    set @done = @done + @remaining;
    print '  users: ' + cast(@done as varchar(10)) + ' / ' + cast(@total as varchar(10));
end
go

declare @batch_size int = $(batch_size);
declare @total int = $(n);
declare @done int = 0;
declare @remaining int;
declare @cat_count int = (select count(*) from categories);

while @done < @total
begin
    set @remaining = case when @total - @done < @batch_size then @total - @done else @batch_size end;

    begin transaction;
    ;with nums(g) as (
        select 1
        union all
        select g + 1 from nums where g < @remaining
    )
    select g, abs(checksum(newid())) % @cat_count + 1 as pick
    into #picks
    from nums
    option (maxrecursion 0);

    with cats as (select row_number() over (order by id) as rn, id from categories)
    insert into products (category_id, sku, name, price_cents, in_stock)
    select
        c.id,
        'SKU-' + cast(@done + p.g as varchar(10)) + '-' + left(cast(newid() as varchar(36)), 6),
        (case abs(checksum(newid())) % 6
            when 0 then 'Widget' when 1 then 'Gadget' when 2 then 'Gizmo'
            when 3 then 'Doohickey' when 4 then 'Contraption' else 'Thingamajig' end)
        + ' ' +
        (case abs(checksum(newid())) % 6
            when 0 then 'Pro' when 1 then 'Max' when 2 then 'Mini'
            when 3 then 'Plus' when 4 then 'Lite' else 'Ultra' end),
        abs(checksum(newid())) % 49900 + 100,
        case when abs(checksum(newid())) % 100 < 90 then 1 else 0 end
    from #picks p
    join cats c on c.rn = p.pick;

    drop table #picks;
    commit transaction;

    set @done = @done + @remaining;
    print '  products: ' + cast(@done as varchar(10)) + ' / ' + cast(@total as varchar(10));
end
go

declare @batch_size int = $(batch_size);
declare @total int = $(n);
declare @done int = 0;
declare @remaining int;
declare @user_count int = (select count(*) from users);

while @done < @total
begin
    set @remaining = case when @total - @done < @batch_size then @total - @done else @batch_size end;

    begin transaction;
    ;with nums(g) as (
        select 1
        union all
        select g + 1 from nums where g < @remaining
    )
    select g, abs(checksum(newid())) % @user_count + 1 as pick
    into #picks
    from nums
    option (maxrecursion 0);

    with usrs as (select row_number() over (order by id) as rn, id from users)
    insert into orders (user_id, status, total_cents, notes)
    select
        u.id,
        (case abs(checksum(newid())) % 4
            when 0 then 'pending' when 1 then 'paid' when 2 then 'shipped' else 'cancelled' end),
        abs(checksum(newid())) % 99900 + 500,
        case when abs(checksum(newid())) % 100 < 30 then 'Order note ' + cast(@done + p.g as varchar(10)) else null end
    from #picks p
    join usrs u on u.rn = p.pick;

    drop table #picks;
    commit transaction;

    set @done = @done + @remaining;
    print '  orders: ' + cast(@done as varchar(10)) + ' / ' + cast(@total as varchar(10));
end
go

declare @batch_size int = $(batch_size);
declare @total int = $(n) * 2;
declare @done int = 0;
declare @remaining int;
declare @order_count int = (select count(*) from orders);
declare @product_count int = (select count(*) from products);

while @done < @total
begin
    set @remaining = case when @total - @done < @batch_size then @total - @done else @batch_size end;

    begin transaction;
    ;with nums(g) as (
        select 1
        union all
        select g + 1 from nums where g < @remaining
    )
    select
        g,
        abs(checksum(newid())) % @order_count + 1 as order_pick,
        abs(checksum(newid())) % @product_count + 1 as product_pick
    into #picks
    from nums
    option (maxrecursion 0);

    with ords as (select row_number() over (order by id) as rn, id from orders),
    prods as (select row_number() over (order by id) as rn, id from products)
    insert into order_items (order_id, product_id, quantity, unit_price_cents)
    select
        o.id,
        pr.id,
        abs(checksum(newid())) % 5 + 1,
        abs(checksum(newid())) % 49900 + 100
    from #picks p
    join ords o on o.rn = p.order_pick
    join prods pr on pr.rn = p.product_pick;

    drop table #picks;
    commit transaction;

    set @done = @done + @remaining;
    print '  order_items: ' + cast(@done as varchar(10)) + ' / ' + cast(@total as varchar(10));
end
go

select
    (select count(*) from users) as users,
    (select count(*) from categories) as categories,
    (select count(*) from products) as products,
    (select count(*) from orders) as orders,
    (select count(*) from order_items) as order_items;
go
