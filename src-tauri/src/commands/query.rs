use tauri::State;

use crate::domain::query::{service, QueryResult};
use crate::error::AppError;
use crate::state::ConnectionRegistry;

#[tauri::command]
#[specta::specta]
pub async fn db_execute_query(
    connection_id: String,
    sql: String,
    registry: State<'_, ConnectionRegistry>,
) -> Result<QueryResult, AppError> {
    let pool = registry
        .get(&connection_id)
        .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))?;

    service::execute_query(&pool, &sql).await
}
