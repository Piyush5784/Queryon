use sqlx::mysql::MySqlPool;
use sqlx::{AssertSqlSafe, Row};

use crate::domain::schema::{
    ColumnInfo, ConstraintInfo, ConstraintKind, ForeignKeyAction, IndexInfo, TableRef,
};
use crate::error::AppError;

fn parse_foreign_key_action(rule: &str) -> Option<ForeignKeyAction> {
    match rule.to_uppercase().as_str() {
        "CASCADE" => Some(ForeignKeyAction::Cascade),
        "SET NULL" => Some(ForeignKeyAction::SetNull),
        "SET DEFAULT" => Some(ForeignKeyAction::SetDefault),
        "RESTRICT" => Some(ForeignKeyAction::Restrict),
        "NO ACTION" => Some(ForeignKeyAction::NoAction),
        _ => None,
    }
}

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
            let kind = if table_type == "VIEW" {
                "view"
            } else {
                "table"
            };
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

/// MariaDB's `information_schema.columns.column_default` returns a
/// quoted-string default still wrapped in literal single quotes with
/// internal quotes doubled (e.g. `'it''s a test'`), where real MySQL
/// returns the bare unescaped string (`it's a test`) for the identical
/// `DEFAULT` clause — confirmed by testing both engines directly against
/// the same `create table ... default 'it''s a test'`. Matches Beekeeper
/// Studio's `MariaDBClient.resolveDefault`
/// (`temp/apps/studio/src/lib/db/clients/mariadb.ts`). Only ever called
/// when `is_mariadb` is true — MySQL's own output must never be touched.
fn unquote_mariadb_default(value: Option<String>) -> Option<String> {
    let value = value?;
    if value.eq_ignore_ascii_case("null") {
        return None;
    }
    if value.len() >= 2 && value.starts_with('\'') && value.ends_with('\'') {
        let inner = &value[1..value.len() - 1];
        return Some(inner.replace("''", "'"));
    }
    Some(value)
}

pub async fn get_table_columns(
    pool: &MySqlPool,
    database: &str,
    table: &str,
    is_mariadb: bool,
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
            let default: Option<String> = row.get("column_default");
            ColumnInfo {
                name: row.get("column_name"),
                data_type: row.get("data_type"),
                is_nullable: row.get("is_nullable"),
                default: if is_mariadb { unquote_mariadb_default(default) } else { default },
                is_primary_key: is_primary_key != 0,
                ordinal_position: ordinal_position as i32,
            }
        })
        .collect())
}

pub async fn list_indexes(
    pool: &MySqlPool,
    database: &str,
    table: &str,
) -> Result<Vec<IndexInfo>, AppError> {
    let rows = sqlx::query(
        r#"
        select
            index_name as index_name,
            (non_unique = 0) as is_unique,
            (index_name = 'PRIMARY') as is_primary,
            group_concat(column_name order by seq_in_index) as columns
        from information_schema.statistics
        where table_schema = ? and table_name = ?
        group by index_name, non_unique
        order by index_name
        "#,
    )
    .bind(database)
    .bind(table)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::new(format!("Failed to list indexes: {e}")))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let columns: String = row.get("columns");
            IndexInfo {
                name: row.get("index_name"),
                columns: columns.split(',').map(|s| s.to_string()).collect(),
                is_unique: row.get("is_unique"),
                is_primary: row.get("is_primary"),
            }
        })
        .collect())
}

pub async fn list_constraints(
    pool: &MySqlPool,
    database: &str,
    table: &str,
) -> Result<Vec<ConstraintInfo>, AppError> {
    let rows = sqlx::query(
        r#"
        select
            tc.constraint_name as constraint_name,
            tc.constraint_type as constraint_type,
            group_concat(distinct kcu.column_name order by kcu.ordinal_position) as columns,
            max(kcu.referenced_table_name) as referenced_table,
            group_concat(distinct kcu.referenced_column_name order by kcu.ordinal_position) as referenced_columns,
            max(rc.update_rule) as update_rule,
            max(rc.delete_rule) as delete_rule
        from information_schema.table_constraints tc
        join information_schema.key_column_usage kcu
            on kcu.constraint_name = tc.constraint_name
            and kcu.table_schema = tc.table_schema
            and kcu.table_name = tc.table_name
        left join information_schema.referential_constraints rc
            on rc.constraint_name = tc.constraint_name
            and rc.constraint_schema = tc.table_schema
            and rc.table_name = tc.table_name
        where tc.table_schema = ? and tc.table_name = ?
            and tc.constraint_type in ('PRIMARY KEY', 'FOREIGN KEY', 'UNIQUE')
        group by tc.constraint_name, tc.constraint_type
        order by tc.constraint_name
        "#,
    )
    .bind(database)
    .bind(table)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::new(format!("Failed to list constraints: {e}")))?;

    let mut constraints: Vec<ConstraintInfo> = rows
        .into_iter()
        .map(|row| {
            let constraint_type: String = row.get("constraint_type");
            let kind = match constraint_type.as_str() {
                "PRIMARY KEY" => ConstraintKind::PrimaryKey,
                "FOREIGN KEY" => ConstraintKind::ForeignKey,
                _ => ConstraintKind::Unique,
            };
            let columns: String = row.get("columns");
            let referenced_columns: Option<String> = row.try_get("referenced_columns").ok();
            let is_fk = kind == ConstraintKind::ForeignKey;
            let update_rule: Option<String> = row.try_get("update_rule").ok().flatten();
            let delete_rule: Option<String> = row.try_get("delete_rule").ok().flatten();
            ConstraintInfo {
                name: row.get("constraint_name"),
                kind,
                columns: columns.split(',').map(|s| s.to_string()).collect(),
                referenced_table: row.try_get("referenced_table").ok(),
                referenced_columns: referenced_columns
                    .map(|c| c.split(',').map(|s| s.to_string()).collect())
                    .unwrap_or_default(),
                on_update: is_fk
                    .then(|| update_rule.as_deref().and_then(parse_foreign_key_action))
                    .flatten(),
                on_delete: is_fk
                    .then(|| delete_rule.as_deref().and_then(parse_foreign_key_action))
                    .flatten(),
                check_expression: None,
            }
        })
        .collect();

    let check_rows = sqlx::query(
        r#"
        select cc.constraint_name as constraint_name, cc.check_clause as check_clause
        from information_schema.check_constraints cc
        join information_schema.table_constraints tc
            on tc.constraint_name = cc.constraint_name
            and tc.constraint_schema = cc.constraint_schema
        where tc.table_schema = ? and tc.table_name = ? and tc.constraint_type = 'CHECK'
        "#,
    )
    .bind(database)
    .bind(table)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::new(format!("Failed to list check constraints: {e}")))?;

    for row in check_rows {
        constraints.push(ConstraintInfo {
            name: row.get("constraint_name"),
            kind: ConstraintKind::Check,
            columns: Vec::new(),
            referenced_table: None,
            referenced_columns: Vec::new(),
            on_update: None,
            on_delete: None,
            check_expression: row.get("check_clause"),
        });
    }

    Ok(constraints)
}

pub async fn get_table_ddl(pool: &MySqlPool, table: &str) -> Result<String, AppError> {
    let sql = format!("show create table `{}`", table.replace('`', "``"));
    let row = sqlx::query(AssertSqlSafe(sql))
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to get table DDL: {e}")))?;

    let ddl: String = row
        .try_get("Create Table")
        .map_err(|e| AppError::new(format!("Unexpected response reading table DDL: {e}")))?;
    Ok(ddl)
}
