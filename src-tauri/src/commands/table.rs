use std::collections::HashMap;
use std::time::Instant;

use serde_json::Value as JsonValue;
use tauri::State;

use crate::domain::table::{service, TableRowsResult};
use crate::error::AppError;
use crate::state::ConnectionRegistry;

/// Command parameters carrying arbitrary row/cell data cross the IPC
/// boundary as JSON-encoded strings rather than structured values —
/// specta cannot export `serde_json::Value` without infinite recursion
/// on its own Array/Object variants (see
/// infrastructure/postgres/executor.rs's `encode_cell`). This decodes
/// them back to `Value` at the command boundary.
fn decode_row(row: HashMap<String, String>) -> Result<HashMap<String, JsonValue>, AppError> {
    row.into_iter()
        .map(|(k, v)| match serde_json::from_str::<JsonValue>(&v) {
            Ok(value) => Ok((k, value)),
            Err(e) => Err(AppError::new(format!("Invalid JSON for column '{k}': {e}"))),
        })
        .collect()
}

#[tauri::command]
#[specta::specta]
pub async fn db_fetch_table_rows(
    connection_id: String,
    schema: String,
    table: String,
    limit: i32,
    offset: i32,
    registry: State<'_, ConnectionRegistry>,
) -> Result<TableRowsResult, AppError> {
    let driver = registry
        .get(&connection_id)
        .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))?;

    let start = Instant::now();
    let mut result =
        service::fetch_table_rows(driver.as_ref(), &schema, &table, limit as i64, offset as i64).await?;
    result.duration_ms = start.elapsed().as_millis() as u32;

    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn db_update_json_cell(
    connection_id: String,
    schema: String,
    table: String,
    row: HashMap<String, String>,
    column: String,
    value: String,
    registry: State<'_, ConnectionRegistry>,
) -> Result<(), AppError> {
    let driver = registry
        .get(&connection_id)
        .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))?;

    let row = decode_row(row)?;
    let value: JsonValue = serde_json::from_str(&value)
        .map_err(|e| AppError::new(format!("Invalid JSON value: {e}")))?;

    service::update_json_cell(driver.as_ref(), &schema, &table, &row, &column, &value).await
}

#[tauri::command]
#[specta::specta]
pub async fn db_update_cell_text(
    connection_id: String,
    schema: String,
    table: String,
    row: HashMap<String, String>,
    column: String,
    value: Option<String>,
    registry: State<'_, ConnectionRegistry>,
) -> Result<(), AppError> {
    let driver = registry
        .get(&connection_id)
        .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))?;

    let row = decode_row(row)?;
    service::update_cell_text(driver.as_ref(), &schema, &table, &row, &column, value.as_deref()).await
}

#[tauri::command]
#[specta::specta]
pub async fn db_delete_rows(
    connection_id: String,
    schema: String,
    table: String,
    rows: Vec<HashMap<String, String>>,
    registry: State<'_, ConnectionRegistry>,
) -> Result<u32, AppError> {
    let driver = registry
        .get(&connection_id)
        .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))?;

    let rows = rows.into_iter().map(decode_row).collect::<Result<Vec<_>, _>>()?;
    let affected = service::delete_rows(driver.as_ref(), &schema, &table, &rows).await?;
    Ok(affected as u32)
}
