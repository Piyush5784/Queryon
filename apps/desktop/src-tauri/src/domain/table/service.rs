use std::collections::HashMap;

use serde_json::Value as JsonValue;

use crate::domain::driver::DatabaseDriver;
use crate::error::AppError;

use super::models::{TableFilter, TableRowsResult, TableSort};

pub async fn fetch_table_rows(
    driver: &dyn DatabaseDriver,
    schema: &str,
    table: &str,
    limit: i64,
    offset: i64,
    filters: &[TableFilter],
    sort: &[TableSort],
) -> Result<TableRowsResult, AppError> {
    driver.fetch_table_rows(schema, table, limit, offset, filters, sort).await
}

pub async fn count_table_rows(
    driver: &dyn DatabaseDriver,
    schema: &str,
    table: &str,
    filters: &[TableFilter],
) -> Result<u64, AppError> {
    driver.count_table_rows(schema, table, filters).await
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

pub async fn insert_row(
    driver: &dyn DatabaseDriver,
    schema: &str,
    table: &str,
    values: &HashMap<String, JsonValue>,
) -> Result<(), AppError> {
    driver.insert_row(schema, table, values).await
}
