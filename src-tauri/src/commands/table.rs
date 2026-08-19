use std::collections::HashMap;

use serde_json::Value as JsonValue;
use tauri::State;

use crate::domain::table::{service, TableRowsResult};
use crate::error::AppError;
use crate::state::ConnectionRegistry;

#[tauri::command]
pub async fn db_fetch_table_rows(
    connection_id: String,
    schema: String,
    table: String,
    limit: i64,
    offset: i64,
    registry: State<'_, ConnectionRegistry>,
) -> Result<TableRowsResult, AppError> {
    let pool = registry
        .get(&connection_id)
        .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))?;

    service::fetch_table_rows(&pool, &schema, &table, limit, offset).await
}

#[tauri::command]
pub async fn db_update_json_cell(
    connection_id: String,
    schema: String,
    table: String,
    row: HashMap<String, JsonValue>,
    column: String,
    value: JsonValue,
    registry: State<'_, ConnectionRegistry>,
) -> Result<(), AppError> {
    let pool = registry
        .get(&connection_id)
        .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))?;

    service::update_json_cell(&pool, &schema, &table, &row, &column, &value).await
}

#[tauri::command]
pub async fn db_update_cell_text(
    connection_id: String,
    schema: String,
    table: String,
    row: HashMap<String, JsonValue>,
    column: String,
    value: Option<String>,
    registry: State<'_, ConnectionRegistry>,
) -> Result<(), AppError> {
    let pool = registry
        .get(&connection_id)
        .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))?;

    service::update_cell_text(&pool, &schema, &table, &row, &column, value.as_deref()).await
}

#[tauri::command]
pub async fn db_delete_rows(
    connection_id: String,
    schema: String,
    table: String,
    rows: Vec<HashMap<String, JsonValue>>,
    registry: State<'_, ConnectionRegistry>,
) -> Result<u64, AppError> {
    let pool = registry
        .get(&connection_id)
        .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))?;

    service::delete_rows(&pool, &schema, &table, &rows).await
}
