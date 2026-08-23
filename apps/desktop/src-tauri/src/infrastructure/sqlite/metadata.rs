use sqlx::sqlite::SqlitePool;
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

/// SQLite has no schema layer at all (`ATTACH DATABASE` aliases exist but
/// this app doesn't expose them) — every `TableRef.schema` is the fixed
/// string `"main"`, matching the implicit schema name SQLite itself uses
/// in qualified references like `main.products`. `sqlite_sequence` is an
/// internal bookkeeping table for `AUTOINCREMENT` columns, not a user
/// table — filtered out the same way Beekeeper's `listTables` implicitly
/// excludes it by only querying `type='table'` names the user created
/// (confirmed live: a fresh `sqlite_master` already contains it once any
/// table uses `AUTOINCREMENT`).
pub async fn list_tables(pool: &SqlitePool) -> Result<Vec<TableRef>, AppError> {
    let rows = sqlx::query(
        r#"
        select name, type
        from sqlite_master
        where type in ('table', 'view')
            and name != 'sqlite_sequence'
            and name not like 'sqlite_%'
        order by name
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::new(format!("Failed to list tables: {}", crate::error::describe_sqlite_error(&e))))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let kind: String = row.get("type");
            TableRef {
                schema: "main".to_string(),
                name: row.get("name"),
                kind,
                estimated_rows: 0.0,
            }
        })
        .collect())
}

/// `PRAGMA table_xinfo` reports one row per column: `cid` (ordinal),
/// `name`, `type`, `notnull`, `dflt_value`, `pk` (0 = not part of the
/// primary key, otherwise its 1-based position within a composite key —
/// confirmed live against a 2-column PRIMARY KEY table), `hidden`
/// (generated-column marker, not used here). Unlike `information_schema`
/// there's no separate primary-key lookup needed — `pk > 0` is already
/// available on the same row Beekeeper's own `dataToColumns` reads it
/// from (`temp/apps/studio/src/lib/db/clients/sqlite.ts`).
pub async fn get_table_columns(pool: &SqlitePool, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
    let sql = format!("PRAGMA table_xinfo({})", quote_ident(table));
    let rows = sqlx::query(AssertSqlSafe(sql))
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to load columns: {}", crate::error::describe_sqlite_error(&e))))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let pk: i64 = row.get("pk");
            let cid: i64 = row.get("cid");
            let notnull: i64 = row.get("notnull");
            ColumnInfo {
                name: row.get("name"),
                data_type: row.get("type"),
                is_nullable: notnull == 0,
                default: row.get("dflt_value"),
                is_primary_key: pk > 0,
                ordinal_position: cid as i32,
            }
        })
        .collect())
}

/// `PRAGMA index_list` lists every index on `table`, including the
/// implicit ones SQLite creates for `UNIQUE`/`PRIMARY KEY` column
/// constraints — named `sqlite_autoindex_<table>_<n>` and just as real as
/// a user-named index (confirmed live: a `UNIQUE` column constraint
/// creates one of these with no corresponding `CREATE INDEX` in
/// `sqlite_master`, so this is the only place that unique constraint is
/// visible at all), so they are kept rather than filtered out — matching
/// Beekeeper's `listTableIndexes`, which does the same. `origin = 'pk'`
/// marks the index backing an `INTEGER PRIMARY KEY`/`PRIMARY KEY(...)`
/// clause specifically. Column order per index comes from a second
/// `PRAGMA index_xinfo` call per index name.
pub async fn list_indexes(pool: &SqlitePool, table: &str) -> Result<Vec<IndexInfo>, AppError> {
    let sql = format!("PRAGMA index_list({})", quote_ident(table));
    let rows = sqlx::query(AssertSqlSafe(sql))
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to list indexes: {}", crate::error::describe_sqlite_error(&e))))?;

    let mut indexes = Vec::with_capacity(rows.len());
    for row in rows {
        let name: String = row.get("name");
        let unique: i64 = row.get("unique");
        let origin: String = row.get("origin");

        let xinfo_sql = format!("PRAGMA index_xinfo({})", quote_ident(&name));
        let xinfo_rows = sqlx::query(AssertSqlSafe(xinfo_sql))
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::new(format!("Failed to load index columns: {}", crate::error::describe_sqlite_error(&e))))?;

        let columns: Vec<String> = xinfo_rows
            .into_iter()
            .filter(|r| {
                // key = 0 marks an auxiliary rowid/aux column index_xinfo
                // appends after the indexed columns proper — excluded the
                // same way Beekeeper's own listTableIndexes filters on
                // `!!r.name` (these rows report a null column name).
                let key: i64 = r.get("key");
                key != 0
            })
            .map(|r| r.get::<String, _>("name"))
            .collect();

        indexes.push(IndexInfo {
            name,
            columns,
            is_unique: unique != 0,
            is_primary: origin == "pk",
        });
    }
    Ok(indexes)
}

/// SQLite has no `information_schema`/`pg_catalog` constraint catalog at
/// all. Unique constraints are only visible via the index pragmas
/// (`list_indexes`, reused here), and foreign keys via
/// `pragma_foreign_key_list`. The primary key is deliberately NOT sourced
/// from `list_indexes` here — a single-column `INTEGER PRIMARY KEY`
/// (SQLite's rowid alias, the common case, matching this app's own
/// `AUTO_INCREMENT_TYPE` rendering) gets no index row at all in `PRAGMA
/// index_list` (confirmed live: `products.id integer primary key
/// autoincrement` produces zero index rows with `origin = 'pk'`), so
/// `list_indexes`-sourced PK detection silently misses the most common
/// shape. `PRAGMA table_xinfo`'s `pk` column is the one place a rowid
/// alias PK is actually visible, and is used directly instead. There is
/// no server-side CHECK constraint catalog either; a CHECK clause only
/// shows up in `sqlite_master`'s stored `CREATE TABLE` text (see
/// `get_table_ddl`), so this driver does not synthesize
/// `ConstraintKind::Check` rows at all.
pub async fn list_constraints(pool: &SqlitePool, table: &str) -> Result<Vec<ConstraintInfo>, AppError> {
    let mut constraints = Vec::new();

    let pk_columns: Vec<String> = get_table_columns(pool, table)
        .await?
        .into_iter()
        .filter(|c| c.is_primary_key)
        .map(|c| c.name)
        .collect();
    if !pk_columns.is_empty() {
        constraints.push(ConstraintInfo {
            name: format!("pk_{table}"),
            kind: ConstraintKind::PrimaryKey,
            columns: pk_columns,
            referenced_table: None,
            referenced_columns: Vec::new(),
            on_update: None,
            on_delete: None,
            check_expression: None,
        });
    }

    for index in list_indexes(pool, table).await? {
        if index.is_primary {
            // Already covered above via table_xinfo — a composite
            // PRIMARY KEY(a, b) does get an is_primary index row too, but
            // it would otherwise duplicate the pk_<table> entry just
            // pushed.
            continue;
        } else if index.is_unique {
            constraints.push(ConstraintInfo {
                name: index.name,
                kind: ConstraintKind::Unique,
                columns: index.columns,
                referenced_table: None,
                referenced_columns: Vec::new(),
                on_update: None,
                on_delete: None,
                check_expression: None,
            });
        }
    }

    let fk_sql = format!("select * from pragma_foreign_key_list({})", quote_literal(table));
    let fk_rows = sqlx::query(AssertSqlSafe(fk_sql))
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to list foreign keys: {}", crate::error::describe_sqlite_error(&e))))?;

    for row in fk_rows {
        let id: i64 = row.get("id");
        let table_ref: String = row.get("table");
        let from: String = row.get("from");
        let to: String = row.get("to");
        let on_update: String = row.get("on_update");
        let on_delete: String = row.get("on_delete");
        constraints.push(ConstraintInfo {
            name: format!("fk_{table}_{id}"),
            kind: ConstraintKind::ForeignKey,
            columns: vec![from],
            referenced_table: Some(table_ref),
            referenced_columns: vec![to],
            on_update: parse_foreign_key_action(&on_update),
            on_delete: parse_foreign_key_action(&on_delete),
            check_expression: None,
        });
    }

    Ok(constraints)
}

pub async fn get_table_ddl(pool: &SqlitePool, table: &str) -> Result<String, AppError> {
    let row = sqlx::query("select sql from sqlite_master where name = ? and type in ('table', 'view')")
        .bind(table)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to get table DDL: {}", crate::error::describe_sqlite_error(&e))))?;

    match row {
        Some(row) => row
            .try_get::<String, _>("sql")
            .map_err(|e| AppError::new(format!("Unexpected response reading table DDL: {e}"))),
        None => Err(AppError::new(format!("Table '{table}' not found."))),
    }
}

pub fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

fn quote_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
