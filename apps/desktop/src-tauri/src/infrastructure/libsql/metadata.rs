use libsql::{Connection, Value as LibsqlValue};

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

fn describe(err: libsql::Error) -> AppError {
    AppError::new(crate::error::describe_libsql_error(&err))
}

fn text(value: &LibsqlValue) -> Option<String> {
    match value {
        LibsqlValue::Text(s) => Some(s.clone()),
        LibsqlValue::Integer(i) => Some(i.to_string()),
        LibsqlValue::Real(r) => Some(r.to_string()),
        LibsqlValue::Null => None,
        LibsqlValue::Blob(_) => None,
    }
}

fn integer(value: &LibsqlValue) -> i64 {
    match value {
        LibsqlValue::Integer(i) => *i,
        LibsqlValue::Real(r) => *r as i64,
        LibsqlValue::Text(s) => s.parse().unwrap_or(0),
        _ => 0,
    }
}

pub async fn list_tables(conn: &Connection) -> Result<Vec<TableRef>, AppError> {
    let mut rows = conn
        .query(
            "select name, type from sqlite_master where type in ('table', 'view') and name != 'sqlite_sequence' and name not like 'sqlite_%' order by name",
            (),
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to list tables: {}", crate::error::describe_libsql_error(&e))))?;

    let mut tables = Vec::new();
    while let Some(row) = rows.next().await.map_err(describe)? {
        let name: String = row.get(0).map_err(describe)?;
        let kind: String = row.get(1).map_err(describe)?;
        tables.push(TableRef { schema: "main".to_string(), name, kind, estimated_rows: 0.0 });
    }
    Ok(tables)
}

pub async fn get_table_columns(conn: &Connection, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
    let sql = format!("PRAGMA table_xinfo({})", quote_ident(table));
    let mut rows = conn
        .query(&sql, ())
        .await
        .map_err(|e| AppError::new(format!("Failed to load columns: {}", crate::error::describe_libsql_error(&e))))?;

    let mut columns = Vec::new();
    while let Some(row) = rows.next().await.map_err(describe)? {
        let cid: i64 = row.get(0).map_err(describe)?;
        let name: String = row.get(1).map_err(describe)?;
        let data_type: String = row.get(2).map_err(describe)?;
        let notnull: i64 = row.get(3).map_err(describe)?;
        let default_value = row.get_value(4).map_err(describe)?;
        let pk: i64 = row.get(5).map_err(describe)?;
        columns.push(ColumnInfo {
            name,
            data_type,
            is_nullable: notnull == 0,
            default: text(&default_value),
            is_primary_key: pk > 0,
            ordinal_position: cid as i32,
        });
    }
    Ok(columns)
}

pub async fn list_indexes(conn: &Connection, table: &str) -> Result<Vec<IndexInfo>, AppError> {
    let sql = format!("PRAGMA index_list({})", quote_ident(table));
    let mut rows = conn
        .query(&sql, ())
        .await
        .map_err(|e| AppError::new(format!("Failed to list indexes: {}", crate::error::describe_libsql_error(&e))))?;

    let mut header: Vec<(String, i64, String)> = Vec::new();
    while let Some(row) = rows.next().await.map_err(describe)? {
        let name: String = row.get(1).map_err(describe)?;
        let unique: i64 = row.get(2).map_err(describe)?;
        let origin: String = row.get(3).map_err(describe)?;
        header.push((name, unique, origin));
    }

    let mut indexes = Vec::with_capacity(header.len());
    for (name, unique, origin) in header {
        let xinfo_sql = format!("PRAGMA index_xinfo({})", quote_ident(&name));
        let mut xinfo_rows = conn
            .query(&xinfo_sql, ())
            .await
            .map_err(|e| AppError::new(format!("Failed to load index columns: {}", crate::error::describe_libsql_error(&e))))?;

        let mut columns = Vec::new();
        while let Some(row) = xinfo_rows.next().await.map_err(describe)? {
            let key: i64 = row.get(5).map_err(describe)?;
            if key == 0 {
                continue;
            }
            let col_name = row.get_value(2).map_err(describe)?;
            if let Some(col_name) = text(&col_name) {
                columns.push(col_name);
            }
        }

        indexes.push(IndexInfo { name, columns, is_unique: unique != 0, is_primary: origin == "pk" });
    }
    Ok(indexes)
}

pub async fn list_constraints(conn: &Connection, table: &str) -> Result<Vec<ConstraintInfo>, AppError> {
    let mut constraints = Vec::new();

    let pk_columns: Vec<String> = get_table_columns(conn, table)
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

    for index in list_indexes(conn, table).await? {
        if index.is_primary {
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
    let mut fk_rows = conn
        .query(&fk_sql, ())
        .await
        .map_err(|e| AppError::new(format!("Failed to list foreign keys: {}", crate::error::describe_libsql_error(&e))))?;

    while let Some(row) = fk_rows.next().await.map_err(describe)? {
        let id_value = row.get_value(0).map_err(describe)?;
        let id = integer(&id_value);
        let table_ref: String = row.get(2).map_err(describe)?;
        let from: String = row.get(3).map_err(describe)?;
        let to: String = row.get(4).map_err(describe)?;
        let on_update: String = row.get(5).map_err(describe)?;
        let on_delete: String = row.get(6).map_err(describe)?;
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

pub async fn get_table_ddl(conn: &Connection, table: &str) -> Result<String, AppError> {
    let mut rows = conn
        .query(
            "select sql from sqlite_master where name = ?1 and type in ('table', 'view')",
            [LibsqlValue::Text(table.to_string())],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to get table DDL: {}", crate::error::describe_libsql_error(&e))))?;

    match rows.next().await.map_err(describe)? {
        Some(row) => {
            let sql: String = row.get(0).map_err(describe)?;
            Ok(sql)
        }
        None => Err(AppError::new(format!("Table '{table}' not found."))),
    }
}

pub fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

fn quote_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}
