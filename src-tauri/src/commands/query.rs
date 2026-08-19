use std::time::Instant;

use tauri::{AppHandle, State, Wry};

use crate::domain::query::{service, QueryHistoryEntry, QueryResult, SavedQuery};
use crate::error::AppError;
use crate::state::ConnectionRegistry;

#[tauri::command]
#[specta::specta]
pub async fn db_execute_query(
    app: AppHandle<Wry>,
    connection_id: String,
    sql: String,
    registry: State<'_, ConnectionRegistry>,
) -> Result<QueryResult, AppError> {
    let driver = registry
        .get(&connection_id)
        .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))?;

    let start = Instant::now();
    let outcome = service::execute_query(driver.as_ref(), &sql).await;
    let duration_ms = start.elapsed().as_millis() as u32;

    match outcome {
        Ok(result) => {
            service::log_history_success(&app, &connection_id, &sql, &result, duration_ms);
            Ok(result)
        }
        Err(err) => {
            service::log_history_error(&app, &connection_id, &sql, &err.to_string(), duration_ms);
            Err(err)
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn db_save_query(app: AppHandle<Wry>, query: SavedQuery) -> Result<(), AppError> {
    service::save_query(&app, query)
}

#[tauri::command]
#[specta::specta]
pub fn db_list_saved_queries(app: AppHandle<Wry>, connection_id: String) -> Result<Vec<SavedQuery>, AppError> {
    service::list_saved_queries(&app, &connection_id)
}

#[tauri::command]
#[specta::specta]
pub fn db_delete_saved_query(app: AppHandle<Wry>, query_id: String) -> Result<(), AppError> {
    service::delete_saved_query(&app, &query_id)
}

#[tauri::command]
#[specta::specta]
pub fn db_list_query_history(
    app: AppHandle<Wry>,
    connection_id: String,
) -> Result<Vec<QueryHistoryEntry>, AppError> {
    service::list_query_history(&app, &connection_id)
}

#[tauri::command]
#[specta::specta]
pub fn db_clear_query_history(app: AppHandle<Wry>, connection_id: String) -> Result<(), AppError> {
    service::clear_query_history(&app, &connection_id)
}
