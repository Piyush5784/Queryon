use std::collections::HashMap;

use deadpool_postgres::Pool;
use serde_json::Value as JsonValue;

use crate::error::AppError;
use crate::infrastructure::postgres::{executor, metadata};

use super::models::TableRowsResult;

const MAX_PAGE_SIZE: i64 = 500;

pub async fn fetch_table_rows(
    pool: &Pool,
    schema: &str,
    table: &str,
    limit: i64,
    offset: i64,
) -> Result<TableRowsResult, AppError> {
    validate_identifier(schema)?;
    validate_identifier(table)?;

    let limit = limit.clamp(1, MAX_PAGE_SIZE);
    let offset = offset.max(0);

    let client = pool
        .get()
        .await
        .map_err(|e| AppError::new(format!("Connection lost: {e}")))?;

    executor::fetch_rows(&client, schema, table, limit, offset).await
}

pub async fn update_json_cell(
    pool: &Pool,
    schema: &str,
    table: &str,
    row: &HashMap<String, JsonValue>,
    column: &str,
    value: &JsonValue,
) -> Result<(), AppError> {
    let (client, pk_values) = resolve_pk_values(pool, schema, table, column, row).await?;
    executor::update_json_cell(&client, schema, table, &pk_values, column, value).await
}

pub async fn update_cell_text(
    pool: &Pool,
    schema: &str,
    table: &str,
    row: &HashMap<String, JsonValue>,
    column: &str,
    new_value: Option<&str>,
) -> Result<(), AppError> {
    validate_identifier(schema)?;
    validate_identifier(table)?;
    validate_identifier(column)?;

    let client = pool
        .get()
        .await
        .map_err(|e| AppError::new(format!("Connection lost: {e}")))?;

    let columns = metadata::get_table_columns(&client, schema, table).await?;
    let target_column = columns
        .iter()
        .find(|c| c.name == column)
        .ok_or_else(|| AppError::new(format!("Unknown column '{column}'.")))?;

    let pk_values = pk_values_from_row(&columns, row)?;

    executor::update_cell_text(
        &client,
        schema,
        table,
        &pk_values,
        column,
        &target_column.data_type,
        new_value,
    )
    .await
}

pub async fn delete_rows(
    pool: &Pool,
    schema: &str,
    table: &str,
    rows: &[HashMap<String, JsonValue>],
) -> Result<u64, AppError> {
    validate_identifier(schema)?;
    validate_identifier(table)?;

    if rows.is_empty() {
        return Ok(0);
    }

    let client = pool
        .get()
        .await
        .map_err(|e| AppError::new(format!("Connection lost: {e}")))?;

    let columns = metadata::get_table_columns(&client, schema, table).await?;

    let mut rows_pk_values = Vec::with_capacity(rows.len());
    for row in rows {
        rows_pk_values.push(pk_values_from_row(&columns, row)?);
    }

    executor::delete_rows(&client, schema, table, &rows_pk_values).await
}

async fn resolve_pk_values(
    pool: &Pool,
    schema: &str,
    table: &str,
    column: &str,
    row: &HashMap<String, JsonValue>,
) -> Result<(deadpool_postgres::Client, Vec<(String, JsonValue)>), AppError> {
    validate_identifier(schema)?;
    validate_identifier(table)?;
    validate_identifier(column)?;

    let client = pool
        .get()
        .await
        .map_err(|e| AppError::new(format!("Connection lost: {e}")))?;

    let columns = metadata::get_table_columns(&client, schema, table).await?;
    let pk_values = pk_values_from_row(&columns, row)?;

    Ok((client, pk_values))
}

fn pk_values_from_row(
    columns: &[crate::domain::schema::ColumnInfo],
    row: &HashMap<String, JsonValue>,
) -> Result<Vec<(String, JsonValue)>, AppError> {
    let pk_columns: Vec<&str> = columns
        .iter()
        .filter(|c| c.is_primary_key)
        .map(|c| c.name.as_str())
        .collect();

    if pk_columns.is_empty() {
        return Err(AppError::new(
            "This table has no primary key — cannot safely target a single row to update.",
        ));
    }

    let mut pk_values = Vec::with_capacity(pk_columns.len());
    for pk_col in &pk_columns {
        let value = row
            .get(*pk_col)
            .ok_or_else(|| AppError::new(format!("Missing primary key value for '{pk_col}'.")))?;
        pk_values.push((pk_col.to_string(), value.clone()));
    }
    Ok(pk_values)
}

fn validate_identifier(ident: &str) -> Result<(), AppError> {
    let mut chars = ident.chars();
    let first_ok = chars
        .next()
        .map(|c| c.is_ascii_alphabetic() || c == '_')
        .unwrap_or(false);
    let rest_ok = chars.all(|c| c.is_ascii_alphanumeric() || c == '_');

    if ident.is_empty() || !first_ok || !rest_ok {
        return Err(AppError::new(format!("Invalid identifier: {ident}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plain_identifiers() {
        assert!(validate_identifier("users").is_ok());
        assert!(validate_identifier("_private").is_ok());
        assert!(validate_identifier("order_items").is_ok());
        assert!(validate_identifier("Users2").is_ok());
    }

    #[test]
    fn rejects_empty_identifier() {
        assert!(validate_identifier("").is_err());
    }

    #[test]
    fn rejects_identifier_starting_with_digit() {
        assert!(validate_identifier("2fast").is_err());
    }

    #[test]
    fn rejects_sql_injection_attempts() {
        assert!(validate_identifier("users; drop table users;--").is_err());
        assert!(validate_identifier("users\" OR \"1\"=\"1").is_err());
        assert!(validate_identifier("users--").is_err());
        assert!(validate_identifier("users.other").is_err());
        assert!(validate_identifier("users OR 1=1").is_err());
        assert!(validate_identifier("users)").is_err());
    }

    #[test]
    fn rejects_whitespace_and_quotes() {
        assert!(validate_identifier("my table").is_err());
        assert!(validate_identifier("\"users\"").is_err());
        assert!(validate_identifier("'users'").is_err());
    }
}
