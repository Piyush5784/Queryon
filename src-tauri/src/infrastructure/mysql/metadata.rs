use sqlx::mysql::MySqlPool;
use sqlx::Row;

use crate::domain::schema::{ColumnInfo, TableRef};
use crate::error::AppError;

/// MySQL has no schema layer distinct from the database itself —
/// `information_schema.tables.table_schema` is the database name the
/// connection is already scoped to. `TableRef.schema` is populated with
/// that same name so the UI's schema grouping still makes sense.
///
/// Every selected column carries an explicit lowercase alias: MySQL
/// renders result column labels using `information_schema`'s stored
/// (uppercase) casing regardless of how the column was referenced in the
/// query, unlike Postgres which lowercases unquoted identifiers.
pub async fn list_tables(pool: &MySqlPool, database: &str) -> Result<Vec<TableRef>, AppError> {
    let rows = sqlx::query(
        r#"
        select
            table_schema as table_schema,
            table_name as table_name,
            table_type as table_type,
            table_rows as table_rows
        from information_schema.tables
        where table_schema = ?
        order by table_name
        "#,
    )
    .bind(database)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::new(format!("Failed to list tables: {e}")))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let table_type: String = row.get("table_type");
            let kind = if table_type == "VIEW" { "view" } else { "table" };
            let estimated_rows: Option<i64> = row.try_get("table_rows").ok();

            TableRef {
                schema: row.get("table_schema"),
                name: row.get("table_name"),
                kind: kind.to_string(),
                estimated_rows: estimated_rows.unwrap_or(0).max(0) as f64,
            }
        })
        .collect())
}

pub async fn get_table_columns(
    pool: &MySqlPool,
    database: &str,
    table: &str,
) -> Result<Vec<ColumnInfo>, AppError> {
    let rows = sqlx::query(
        r#"
        select
            col.column_name as column_name,
            col.data_type as data_type,
            (col.is_nullable = 'YES') as is_nullable,
            col.column_default as column_default,
            col.ordinal_position as ordinal_position,
            (
                select count(*) > 0
                from information_schema.key_column_usage kcu
                join information_schema.table_constraints tc
                    on tc.constraint_name = kcu.constraint_name
                    and tc.table_schema = kcu.table_schema
                    and tc.table_name = kcu.table_name
                where tc.constraint_type = 'PRIMARY KEY'
                    and kcu.table_schema = col.table_schema
                    and kcu.table_name = col.table_name
                    and kcu.column_name = col.column_name
            ) as is_primary_key
        from information_schema.columns col
        where col.table_schema = ? and col.table_name = ?
        order by col.ordinal_position
        "#,
    )
    .bind(database)
    .bind(table)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::new(format!("Failed to load columns: {e}")))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let is_primary_key: i64 = row.get("is_primary_key");
            let ordinal_position: u32 = row.get("ordinal_position");
            ColumnInfo {
                name: row.get("column_name"),
                data_type: row.get("data_type"),
                is_nullable: row.get("is_nullable"),
                default: row.get("column_default"),
                is_primary_key: is_primary_key != 0,
                ordinal_position: ordinal_position as i32,
            }
        })
        .collect())
}
