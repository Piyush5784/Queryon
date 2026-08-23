use serde_json::Value as JsonValue;

use crate::domain::schema::{ColumnInfo, ConstraintInfo, IndexInfo, TableRef};
use crate::error::AppError;

use super::pool::ClickHouseClient;

fn text(row: &serde_json::Map<String, JsonValue>, key: &str) -> String {
    row.get(key).and_then(|v| v.as_str()).unwrap_or_default().to_string()
}

fn as_u8(row: &serde_json::Map<String, JsonValue>, key: &str) -> u8 {
    row.get(key).and_then(|v| v.as_u64()).unwrap_or(0) as u8
}

fn strip_nullable(type_name: &str) -> (String, bool) {
    if let Some(inner) = type_name.strip_prefix("Nullable(").and_then(|s| s.strip_suffix(')')) {
        (inner.to_string(), true)
    } else {
        (type_name.to_string(), false)
    }
}

pub async fn list_tables(client: &ClickHouseClient) -> Result<Vec<TableRef>, AppError> {
    let outcome = client
        .query(
            "select name, engine, total_rows from system.tables where database = currentDatabase() order by name",
        )
        .await?;

    Ok(outcome
        .rows
        .into_iter()
        .map(|row| {
            let engine = text(&row, "engine");
            let kind = if engine.contains("View") { "view" } else { "table" };
            let estimated_rows = row.get("total_rows").and_then(|v| v.as_f64()).unwrap_or(0.0);
            TableRef {
                schema: client.database().to_string(),
                name: text(&row, "name"),
                kind: kind.to_string(),
                estimated_rows,
            }
        })
        .collect())
}

pub async fn get_table_columns(client: &ClickHouseClient, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
    let sql = format!(
        "select name, type, is_in_primary_key, position, default_kind, default_expression \
         from system.columns where database = currentDatabase() and table = '{table}' order by position"
    );
    let outcome = client.query(&sql).await?;

    Ok(outcome
        .rows
        .into_iter()
        .map(|row| {
            let raw_type = text(&row, "type");
            let (data_type, is_nullable) = strip_nullable(&raw_type);
            let default_kind = text(&row, "default_kind");
            let default_expression = text(&row, "default_expression");
            let default = if default_kind == "DEFAULT" && !default_expression.is_empty() {
                Some(default_expression)
            } else {
                None
            };

            ColumnInfo {
                name: text(&row, "name"),
                data_type,
                is_nullable,
                default,
                is_primary_key: as_u8(&row, "is_in_primary_key") == 1,
                ordinal_position: row.get("position").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            }
        })
        .collect())
}

pub async fn list_indexes(client: &ClickHouseClient, table: &str) -> Result<Vec<IndexInfo>, AppError> {
    let sql = format!("show index from `{table}` in `{}`", client.database());
    let outcome = client.query(&sql).await?;

    let mut indexes: Vec<IndexInfo> = Vec::new();
    for row in outcome.rows {
        let key_name = text(&row, "key_name");
        let is_primary = key_name == "PRIMARY";
        let column = if is_primary { text(&row, "pk_col") } else { text(&row, "expression") };

        if let Some(existing) = indexes.iter_mut().find(|idx: &&mut IndexInfo| idx.name == key_name) {
            if !column.is_empty() {
                existing.columns.push(column);
            }
        } else {
            indexes.push(IndexInfo {
                name: key_name,
                columns: if column.is_empty() { Vec::new() } else { vec![column] },
                is_unique: is_primary,
                is_primary,
            });
        }
    }
    Ok(indexes)
}

pub async fn list_constraints(client: &ClickHouseClient, table: &str) -> Result<Vec<ConstraintInfo>, AppError> {
    let ddl = get_table_ddl(client, table).await?;
    let mut constraints = Vec::new();

    for line in ddl.lines() {
        let trimmed = line.trim().trim_end_matches(',');
        if let Some(rest) = trimmed.strip_prefix("CONSTRAINT ") {
            if let Some((name, expr)) = rest.split_once(" CHECK ") {
                constraints.push(ConstraintInfo {
                    name: name.trim().trim_matches('`').to_string(),
                    kind: crate::domain::schema::ConstraintKind::Check,
                    columns: Vec::new(),
                    referenced_table: None,
                    referenced_columns: Vec::new(),
                    on_update: None,
                    on_delete: None,
                    check_expression: Some(expr.trim().to_string()),
                });
            }
        }
    }
    Ok(constraints)
}

pub async fn get_table_ddl(client: &ClickHouseClient, table: &str) -> Result<String, AppError> {
    let sql = format!("show create table `{table}`");
    let outcome = client.query(&sql).await?;
    let row = outcome
        .rows
        .into_iter()
        .next()
        .ok_or_else(|| AppError::new(format!("Table '{table}' not found.")))?;
    Ok(row
        .values()
        .next()
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string())
}
