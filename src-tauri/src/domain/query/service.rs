use std::time::Instant;

use tauri::{AppHandle, Wry};

use crate::domain::driver::DatabaseDriver;
use crate::error::AppError;
use crate::infrastructure::storage::{query_history_store, saved_query_store};

use super::models::{
    encode_cell, QueryHistoryEntry, QueryHistoryStatus, QueryResult, RawQueryResult, SavedQuery,
};

const MAX_RESULT_ROWS: usize = 5000;

pub async fn execute_query(driver: &dyn DatabaseDriver, sql: &str) -> Result<QueryResult, AppError> {
    if sql.trim().is_empty() {
        return Err(AppError::new("Cannot execute an empty query."));
    }

    let start = Instant::now();
    let raw = driver.execute_query(sql, MAX_RESULT_ROWS).await?;
    let duration_ms = start.elapsed().as_millis() as u32;

    Ok(to_query_result(raw, duration_ms))
}

fn to_query_result(raw: RawQueryResult, duration_ms: u32) -> QueryResult {
    match raw {
        RawQueryResult::Rows { columns, rows, row_count } => {
            let truncated = row_count > MAX_RESULT_ROWS;
            let encoded_rows = rows
                .into_iter()
                .map(|row| row.into_iter().map(encode_cell).collect())
                .collect();
            QueryResult::Rows {
                columns,
                rows: encoded_rows,
                row_count: row_count as u32,
                truncated,
                duration_ms,
            }
        }
        RawQueryResult::Affected { row_count } => {
            QueryResult::Affected { row_count: row_count as u32, duration_ms }
        }
    }
}

pub fn log_history_success(
    app: &AppHandle<Wry>,
    connection_id: &str,
    sql: &str,
    result: &QueryResult,
    duration_ms: u32,
) {
    let row_count = match result {
        QueryResult::Rows { row_count, .. } | QueryResult::Affected { row_count, .. } => *row_count,
    };
    let entry = QueryHistoryEntry {
        id: format!("hist_{}", now_millis()),
        connection_id: connection_id.to_string(),
        sql: sql.to_string(),
        status: QueryHistoryStatus::Success,
        error_message: None,
        row_count: Some(row_count),
        duration_ms,
        ran_at: now_iso8601(),
    };
    if let Err(e) = query_history_store::append(app, entry) {
        log::warn!("Failed to record query history: {e}");
    }
}

pub fn log_history_error(
    app: &AppHandle<Wry>,
    connection_id: &str,
    sql: &str,
    error_message: &str,
    duration_ms: u32,
) {
    let entry = QueryHistoryEntry {
        id: format!("hist_{}", now_millis()),
        connection_id: connection_id.to_string(),
        sql: sql.to_string(),
        status: QueryHistoryStatus::Error,
        error_message: Some(error_message.to_string()),
        row_count: None,
        duration_ms,
        ran_at: now_iso8601(),
    };
    if let Err(e) = query_history_store::append(app, entry) {
        log::warn!("Failed to record query history: {e}");
    }
}

fn now_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn now_iso8601() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub fn save_query(app: &AppHandle<Wry>, query: SavedQuery) -> Result<(), AppError> {
    saved_query_store::upsert(app, query)
}

pub fn list_saved_queries(app: &AppHandle<Wry>, connection_id: &str) -> Result<Vec<SavedQuery>, AppError> {
    saved_query_store::list_for_connection(app, connection_id)
}

pub fn delete_saved_query(app: &AppHandle<Wry>, query_id: &str) -> Result<(), AppError> {
    saved_query_store::remove(app, query_id)
}

pub fn list_query_history(
    app: &AppHandle<Wry>,
    connection_id: &str,
) -> Result<Vec<QueryHistoryEntry>, AppError> {
    query_history_store::list_for_connection(app, connection_id)
}

pub fn clear_query_history(app: &AppHandle<Wry>, connection_id: &str) -> Result<(), AppError> {
    query_history_store::clear_for_connection(app, connection_id)
}
