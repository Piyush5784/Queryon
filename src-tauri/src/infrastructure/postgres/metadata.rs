use deadpool_postgres::Client;

use crate::domain::schema::{ColumnInfo, TableRef};
use crate::error::{describe_pg_error, AppError};

pub async fn list_tables(client: &Client) -> Result<Vec<TableRef>, AppError> {
    let rows = client
        .query(
            r#"
            select
                n.nspname as schema,
                c.relname as name,
                case c.relkind
                    when 'r' then 'table'
                    when 'v' then 'view'
                    when 'm' then 'materialized_view'
                    else c.relkind::text
                end as kind,
                greatest(c.reltuples, 0)::bigint as estimated_rows
            from pg_catalog.pg_class c
            join pg_catalog.pg_namespace n on n.oid = c.relnamespace
            where c.relkind in ('r', 'v', 'm')
                and n.nspname not in ('pg_catalog', 'information_schema', 'pg_toast')
                and n.nspname not like 'pg_temp_%'
            order by n.nspname, c.relname
            "#,
            &[],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to list tables: {}", describe_pg_error(&e))))?;

    Ok(rows
        .into_iter()
        .map(|row| TableRef {
            schema: row.get("schema"),
            name: row.get("name"),
            kind: row.get("kind"),
            estimated_rows: row.get("estimated_rows"),
        })
        .collect())
}

pub async fn get_table_columns(
    client: &Client,
    schema: &str,
    table: &str,
) -> Result<Vec<ColumnInfo>, AppError> {
    let rows = client
        .query(
            r#"
            select
                col.column_name,
                col.data_type,
                (col.is_nullable = 'YES') as is_nullable,
                col.column_default,
                col.ordinal_position::int,
                exists (
                    select 1
                    from information_schema.table_constraints tc
                    join information_schema.key_column_usage kcu
                        on kcu.constraint_name = tc.constraint_name
                        and kcu.table_schema = tc.table_schema
                        and kcu.table_name = tc.table_name
                    where tc.constraint_type = 'PRIMARY KEY'
                        and tc.table_schema = col.table_schema
                        and tc.table_name = col.table_name
                        and kcu.column_name = col.column_name
                ) as is_primary_key
            from information_schema.columns col
            where col.table_schema = $1 and col.table_name = $2
            order by col.ordinal_position
            "#,
            &[&schema, &table],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to load columns: {}", describe_pg_error(&e))))?;

    Ok(rows
        .into_iter()
        .map(|row| ColumnInfo {
            name: row.get("column_name"),
            data_type: row.get("data_type"),
            is_nullable: row.get("is_nullable"),
            default: row.get("column_default"),
            is_primary_key: row.get("is_primary_key"),
            ordinal_position: row.get("ordinal_position"),
        })
        .collect())
}
