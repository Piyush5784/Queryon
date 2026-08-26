use serde_json::Value as JsonValue;
use trino_rust_client::client::Client;
use trino_rust_client::Row;

use crate::domain::schema::{ColumnInfo, ConstraintInfo, IndexInfo, TableRef};
use crate::error::AppError;

pub fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

fn quote_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

async fn query_rows(client: &Client, sql: &str) -> Result<Vec<Vec<JsonValue>>, AppError> {
    let dataset = client
        .get_all::<Row>(sql)
        .await
        .map_err(|e| AppError::new(crate::error::describe_trino_error(&e)))?;
    Ok(dataset.into_vec().into_iter().map(Row::into_json).collect())
}

fn text_at(row: &[JsonValue], index: usize) -> String {
    match row.get(index) {
        Some(JsonValue::String(s)) => s.clone(),
        Some(other) => other.to_string(),
        None => String::new(),
    }
}

pub async fn list_tables(client: &Client, schema: &str) -> Result<Vec<TableRef>, AppError> {
    let sql = format!(
        "select table_name, table_type from information_schema.tables where table_schema = {}",
        quote_literal(schema)
    );
    let rows = query_rows(client, &sql).await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let name = text_at(&row, 0);
            let kind = if text_at(&row, 1).eq_ignore_ascii_case("VIEW") { "view" } else { "table" };
            TableRef { schema: schema.to_string(), name, kind: kind.to_string(), estimated_rows: 0.0 }
        })
        .collect())
}

pub async fn get_table_columns(client: &Client, schema: &str, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
    let sql = format!(
        "select column_name, data_type, is_nullable, column_default, ordinal_position from information_schema.columns where table_schema = {} and table_name = {} order by ordinal_position",
        quote_literal(schema),
        quote_literal(table)
    );
    let rows = query_rows(client, &sql).await?;
    if rows.is_empty() {
        return Err(AppError::new(format!("Table '{table}' not found.")));
    }
    Ok(rows
        .into_iter()
        .map(|row| {
            let name = text_at(&row, 0);
            let data_type = text_at(&row, 1);
            let is_nullable = text_at(&row, 2).eq_ignore_ascii_case("YES");
            let default = match row.get(3) {
                Some(JsonValue::Null) | None => None,
                Some(JsonValue::String(s)) => Some(s.clone()),
                Some(other) => Some(other.to_string()),
            };
            let ordinal_position = row.get(4).and_then(JsonValue::as_i64).unwrap_or(0) as i32;
            ColumnInfo { name, data_type, is_nullable, default, is_primary_key: false, ordinal_position }
        })
        .collect())
}

pub async fn list_indexes(_client: &Client, _schema: &str, _table: &str) -> Result<Vec<IndexInfo>, AppError> {
    Ok(Vec::new())
}

pub async fn list_constraints(_client: &Client, _schema: &str, _table: &str) -> Result<Vec<ConstraintInfo>, AppError> {
    Ok(Vec::new())
}

pub async fn get_table_ddl(client: &Client, schema: &str, table: &str) -> Result<String, AppError> {
    let sql = format!("show create table {}.{}", quote_ident(schema), quote_ident(table));
    let rows = query_rows(client, &sql).await?;
    let row = rows.first().ok_or_else(|| AppError::new(format!("Table '{table}' not found.")))?;
    Ok(text_at(row, 0))
}
