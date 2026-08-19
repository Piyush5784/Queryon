use std::collections::HashMap;

use serde_json::Value as JsonValue;

use crate::domain::driver::DatabaseDriver;
use crate::error::AppError;

use super::models::TableRowsResult;

pub async fn fetch_table_rows(
    driver: &dyn DatabaseDriver,
    schema: &str,
    table: &str,
    limit: i64,
    offset: i64,
) -> Result<TableRowsResult, AppError> {
    driver.fetch_table_rows(schema, table, limit, offset).await
}

pub async fn update_json_cell(
    driver: &dyn DatabaseDriver,
    schema: &str,
    table: &str,
    row: &HashMap<String, JsonValue>,
    column: &str,
    value: &JsonValue,
) -> Result<(), AppError> {
    driver.update_json_cell(schema, table, row, column, value).await
}

pub async fn update_cell_text(
    driver: &dyn DatabaseDriver,
    schema: &str,
    table: &str,
    row: &HashMap<String, JsonValue>,
    column: &str,
    new_value: Option<&str>,
) -> Result<(), AppError> {
    driver.update_cell_text(schema, table, row, column, new_value).await
}

pub async fn delete_rows(
    driver: &dyn DatabaseDriver,
    schema: &str,
    table: &str,
    rows: &[HashMap<String, JsonValue>],
) -> Result<u64, AppError> {
    driver.delete_rows(schema, table, rows).await
}
