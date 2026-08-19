use tauri::State;

use crate::domain::schema::{service, ColumnInfo, TableRef};
use crate::error::AppError;
use crate::state::ConnectionRegistry;

fn pool_for(
    registry: &State<'_, ConnectionRegistry>,
    connection_id: &str,
) -> Result<deadpool_postgres::Pool, AppError> {
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
    let pool = pool_for(&registry, &connection_id)?;
    service::list_tables(&pool).await
}

#[tauri::command]
#[specta::specta]
pub async fn db_get_table_columns(
    connection_id: String,
    schema: String,
    table: String,
    registry: State<'_, ConnectionRegistry>,
) -> Result<Vec<ColumnInfo>, AppError> {
    let pool = pool_for(&registry, &connection_id)?;
    service::get_table_columns(&pool, &schema, &table).await
}
