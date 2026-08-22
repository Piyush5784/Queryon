use std::time::Instant;

use tauri::{AppHandle, State, Wry};

use crate::domain::query::{service, QueryHistoryEntry, QueryResult, QueryResultPage, SavedQuery};
use crate::error::AppError;
use crate::state::{ConnectionRegistry, QueryResultCache};

#[tauri::command]
#[specta::specta]
pub async fn db_execute_query(
    app: AppHandle<Wry>,
    connection_id: String,
    sql: String,
    tab_id: Option<String>,
    registry: State<'_, ConnectionRegistry>,
    result_cache: State<'_, QueryResultCache>,
) -> Result<QueryResult, AppError> {
    let driver = if service::is_read_only_statement(&sql) {
        registry
            .get(&connection_id)
            .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))?
    } else {
        registry.require_writable(&connection_id)?
    };

    let start = Instant::now();
    let outcome = match &tab_id {
        Some(tab_id) => service::execute_query_for_tab(driver.as_ref(), tab_id, &sql).await,
        None => service::execute_query(driver.as_ref(), &sql).await,
    };
    let duration_ms = start.elapsed().as_millis() as u32;

    match outcome {
        Ok(result) => {
            if let Some(tab_id) = &tab_id {
                result_cache.store(tab_id.clone(), sql.clone());
            }
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
pub async fn db_fetch_query_result_page(
    connection_id: String,
    tab_id: String,
    offset: u32,
    limit: u32,
    registry: State<'_, ConnectionRegistry>,
    result_cache: State<'_, QueryResultCache>,
) -> Result<QueryResultPage, AppError> {
    let driver = registry
        .get(&connection_id)
        .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))?;
    service::page_query_result(driver.as_ref(), &result_cache, &tab_id, offset as u64, limit as u64).await
}

#[tauri::command]
#[specta::specta]
pub fn db_clear_query_result_cache(tab_id: String, result_cache: State<'_, QueryResultCache>) {
    result_cache.remove(&tab_id);
}

#[tauri::command]
#[specta::specta]
pub async fn db_cancel_query(
    connection_id: String,
    tab_id: String,
    registry: State<'_, ConnectionRegistry>,
) -> Result<(), AppError> {
    let driver = registry
        .get(&connection_id)
        .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))?;
    service::cancel_query(driver.as_ref(), &tab_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn db_begin_transaction(
    connection_id: String,
    tab_id: String,
    registry: State<'_, ConnectionRegistry>,
) -> Result<(), AppError> {
    let driver = registry.require_writable(&connection_id)?;
    service::begin_transaction(driver.as_ref(), &tab_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn db_commit_transaction(
    connection_id: String,
    tab_id: String,
    registry: State<'_, ConnectionRegistry>,
) -> Result<(), AppError> {
    let driver = registry
        .get(&connection_id)
        .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))?;
    service::commit_transaction(driver.as_ref(), &tab_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn db_rollback_transaction(
    connection_id: String,
    tab_id: String,
    registry: State<'_, ConnectionRegistry>,
) -> Result<(), AppError> {
    let driver = registry
        .get(&connection_id)
        .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))?;
    service::rollback_transaction(driver.as_ref(), &tab_id).await
}

#[tauri::command]
#[specta::specta]
pub fn db_has_active_transaction(
    connection_id: String,
    tab_id: String,
    registry: State<'_, ConnectionRegistry>,
) -> bool {
    registry
        .get(&connection_id)
        .map(|driver| driver.has_active_transaction(&tab_id))
        .unwrap_or(false)
}

#[tauri::command]
#[specta::specta]
pub fn db_save_query(app: AppHandle<Wry>, query: SavedQuery) -> Result<(), AppError> {
    service::save_query(&app, query)
}

#[tauri::command]
#[specta::specta]
pub fn db_list_saved_queries(
    app: AppHandle<Wry>,
    connection_id: String,
) -> Result<Vec<SavedQuery>, AppError> {
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
