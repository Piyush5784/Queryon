use std::sync::Arc;

use tauri::State;

use crate::domain::driver::DatabaseDriver;
use crate::domain::schema::{service, ColumnInfo, TableRef};
use crate::error::AppError;
use crate::state::ConnectionRegistry;

fn driver_for(
    registry: &State<'_, ConnectionRegistry>,
    connection_id: &str,
) -> Result<Arc<dyn DatabaseDriver>, AppError> {
    registry
        .get(connection_id)
        .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))
}

#[tauri::command]
#[specta::specta]
pub async fn db_list_tables(
    connection_id: String,
    registry: State<'_, ConnectionRegistry>,
) -> Result<Vec<TableRef>, AppError> {
    let driver = driver_for(&registry, &connection_id)?;
    service::list_tables(driver.as_ref()).await
}

#[tauri::command]
#[specta::specta]
pub async fn db_get_table_columns(
    connection_id: String,
    schema: String,
    table: String,
    registry: State<'_, ConnectionRegistry>,
) -> Result<Vec<ColumnInfo>, AppError> {
    let driver = driver_for(&registry, &connection_id)?;
    service::get_table_columns(driver.as_ref(), &schema, &table).await
}
