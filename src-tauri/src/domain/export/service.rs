use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde_json::Value as JsonValue;
use tauri::{AppHandle, Emitter, Wry};

use crate::domain::driver::DatabaseDriver;
use crate::domain::query::RawQueryResult;

use super::encoder::ExportWriter;
use super::models::{ExportEvent, QueryExportRequest, RowsExportRequest, TableExportRequest};

const PROGRESS_EVENT: &str = "export:progress";
const DONE_EVENT: &str = "export:done";
const CANCELLED_EVENT: &str = "export:cancelled";
const ERROR_EVENT: &str = "export:error";

fn emit(app: &AppHandle<Wry>, event: &ExportEvent) {
    let name = match event {
        ExportEvent::Progress { .. } => PROGRESS_EVENT,
        ExportEvent::Done { .. } => DONE_EVENT,
        ExportEvent::Cancelled { .. } => CANCELLED_EVENT,
        ExportEvent::Error { .. } => ERROR_EVENT,
    };
    let _ = app.emit(name, event);
}

fn cleanup_on_abort(path: &PathBuf, delete_on_abort: bool) {
    if delete_on_abort {
        let _ = std::fs::remove_file(path);
    }
}

pub async fn run_table_export(
    app: AppHandle<Wry>,
    job_id: String,
    request: TableExportRequest,
    driver: Arc<dyn DatabaseDriver>,
    cancel_flag: Arc<AtomicBool>,
) {
    let path = PathBuf::from(&request.directory).join(&request.file_name);

    if let Err(e) = std::fs::create_dir_all(&request.directory) {
        emit(
            &app,
            &ExportEvent::Error { job_id, message: format!("Could not create folder: {e}") },
        );
        return;
    }

    let file = match File::create(&path) {
        Ok(f) => f,
        Err(e) => {
            emit(&app, &ExportEvent::Error { job_id, message: format!("Failed to create file: {e}") });
            return;
        }
    };

    let columns = match driver.get_table_columns(&request.schema, &request.table).await {
        Ok(cols) => cols.into_iter().map(|c| c.name).collect::<Vec<_>>(),
        Err(e) => {
            emit(&app, &ExportEvent::Error { job_id, message: e.to_string() });
            return;
        }
    };

    let mut writer = match ExportWriter::new(
        BufWriter::new(file),
        request.format,
        request.pretty_print,
        columns,
        request.table.clone(),
    ) {
        Ok(w) => w,
        Err(e) => {
            emit(&app, &ExportEvent::Error { job_id, message: e.to_string() });
            cleanup_on_abort(&path, request.delete_on_abort);
            return;
        }
    };

    let chunk_size = request.chunk_size.clamp(50, 5000) as i64;
    let mut offset: i64 = 0;
    let mut total_written: u64 = 0;

    loop {
        if cancel_flag.load(Ordering::Relaxed) {
            emit(&app, &ExportEvent::Cancelled { job_id });
            cleanup_on_abort(&path, request.delete_on_abort);
            return;
        }

        let page = match driver
            .fetch_table_rows(
                &request.schema,
                &request.table,
                chunk_size,
                offset,
                &request.filters,
                &request.sort,
            )
            .await
        {
            Ok(p) => p,
            Err(e) => {
                emit(&app, &ExportEvent::Error { job_id, message: e.to_string() });
                cleanup_on_abort(&path, request.delete_on_abort);
                return;
            }
        };

        let decoded: Vec<Vec<JsonValue>> = page
            .rows
            .iter()
            .map(|row| {
                row.iter()
                    .map(|cell| serde_json::from_str(cell).unwrap_or(JsonValue::Null))
                    .collect()
            })
            .collect();

        if let Err(e) = writer.write_rows(&decoded) {
            emit(&app, &ExportEvent::Error { job_id, message: e.to_string() });
            cleanup_on_abort(&path, request.delete_on_abort);
            return;
        }

        total_written += decoded.len() as u64;
        emit(
            &app,
            &ExportEvent::Progress { job_id: job_id.clone(), rows_written: total_written, total_rows: None },
        );

        if !page.has_more {
            break;
        }
        offset += chunk_size;
    }

    match writer.finish() {
        Ok(_) => emit(
            &app,
            &ExportEvent::Done { job_id, rows_written: total_written, path: path.to_string_lossy().into_owned() },
        ),
        Err(e) => {
            emit(&app, &ExportEvent::Error { job_id, message: e.to_string() });
            cleanup_on_abort(&path, request.delete_on_abort);
        }
    }
}

pub async fn run_rows_export(
    app: AppHandle<Wry>,
    job_id: String,
    request: RowsExportRequest,
    cancel_flag: Arc<AtomicBool>,
) {
    let path = PathBuf::from(&request.directory).join(&request.file_name);

    if let Err(e) = std::fs::create_dir_all(&request.directory) {
        emit(
            &app,
            &ExportEvent::Error { job_id, message: format!("Could not create folder: {e}") },
        );
        return;
    }

    let file = match File::create(&path) {
        Ok(f) => f,
        Err(e) => {
            emit(&app, &ExportEvent::Error { job_id, message: format!("Failed to create file: {e}") });
            return;
        }
    };

    let mut writer = match ExportWriter::new(
        BufWriter::new(file),
        request.format,
        request.pretty_print,
        request.columns.clone(),
        "query_result".to_string(),
    ) {
        Ok(w) => w,
        Err(e) => {
            emit(&app, &ExportEvent::Error { job_id, message: e.to_string() });
            cleanup_on_abort(&path, request.delete_on_abort);
            return;
        }
    };

    const WRITE_CHUNK: usize = 500;
    let mut total_written: u64 = 0;
    let total_rows = request.rows.len() as u64;

    for chunk in request.rows.chunks(WRITE_CHUNK) {
        if cancel_flag.load(Ordering::Relaxed) {
            emit(&app, &ExportEvent::Cancelled { job_id });
            cleanup_on_abort(&path, request.delete_on_abort);
            return;
        }

        let decoded: Vec<Vec<JsonValue>> = chunk
            .iter()
            .map(|row| {
                row.iter()
                    .map(|cell| serde_json::from_str(cell).unwrap_or(JsonValue::Null))
                    .collect()
            })
            .collect();

        if let Err(e) = writer.write_rows(&decoded) {
            emit(&app, &ExportEvent::Error { job_id, message: e.to_string() });
            cleanup_on_abort(&path, request.delete_on_abort);
            return;
        }

        total_written += decoded.len() as u64;
        emit(
            &app,
            &ExportEvent::Progress {
                job_id: job_id.clone(),
                rows_written: total_written,
                total_rows: Some(total_rows),
            },
        );
    }

    match writer.finish() {
        Ok(_) => emit(
            &app,
            &ExportEvent::Done { job_id, rows_written: total_written, path: path.to_string_lossy().into_owned() },
        ),
        Err(e) => {
            emit(&app, &ExportEvent::Error { job_id, message: e.to_string() });
            cleanup_on_abort(&path, request.delete_on_abort);
        }
    }
}

/// Re-runs `request.sql` in chunks via the same `LIMIT`/`OFFSET`
/// pagination `db_fetch_query_result_page` uses (see
/// `DatabaseDriver::execute_query_for_tab`), streaming straight to the
/// export file rather than exporting whatever page happens to be loaded
/// in the results grid — the "whole result" export scope for a query
/// tab, matching how Beekeeper Studio's query export always re-runs the
/// SQL text (see `TabQueryEditor.vue`'s `submitQueryToFile`).
pub async fn run_query_export(
    app: AppHandle<Wry>,
    job_id: String,
    request: QueryExportRequest,
    driver: Arc<dyn DatabaseDriver>,
    cancel_flag: Arc<AtomicBool>,
) {
    let path = PathBuf::from(&request.directory).join(&request.file_name);

    if let Err(e) = std::fs::create_dir_all(&request.directory) {
        emit(
            &app,
            &ExportEvent::Error { job_id, message: format!("Could not create folder: {e}") },
        );
        return;
    }

    let file = match File::create(&path) {
        Ok(f) => f,
        Err(e) => {
            emit(&app, &ExportEvent::Error { job_id, message: format!("Failed to create file: {e}") });
            return;
        }
    };

    let chunk_size = request.chunk_size.clamp(50, 5000) as u64;
    let mut offset: u64 = 0;
    let mut total_written: u64 = 0;
    let mut writer: Option<ExportWriter<BufWriter<File>>> = None;

    loop {
        if cancel_flag.load(Ordering::Relaxed) {
            emit(&app, &ExportEvent::Cancelled { job_id });
            cleanup_on_abort(&path, request.delete_on_abort);
            return;
        }

        let raw = match driver
            .execute_query_for_tab(&request.tab_id, &request.sql, offset, chunk_size)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                emit(&app, &ExportEvent::Error { job_id, message: e.to_string() });
                cleanup_on_abort(&path, request.delete_on_abort);
                return;
            }
        };

        let (columns, rows, total_row_count) = match raw {
            RawQueryResult::Rows { columns, rows, total_row_count } => (columns, rows, total_row_count),
            RawQueryResult::Affected { .. } => {
                emit(
                    &app,
                    &ExportEvent::Error {
                        job_id,
                        message: "This statement doesn't return rows to export.".to_string(),
                    },
                );
                return;
            }
        };

        if writer.is_none() {
            writer = match ExportWriter::new(
                BufWriter::new(file.try_clone().expect("file handle should be cloneable")),
                request.format,
                request.pretty_print,
                columns,
                "query_result".to_string(),
            ) {
                Ok(w) => Some(w),
                Err(e) => {
                    emit(&app, &ExportEvent::Error { job_id, message: e.to_string() });
                    cleanup_on_abort(&path, request.delete_on_abort);
                    return;
                }
            };
        }

        let row_count = rows.len();
        if let Err(e) = writer.as_mut().unwrap().write_rows(&rows) {
            emit(&app, &ExportEvent::Error { job_id, message: e.to_string() });
            cleanup_on_abort(&path, request.delete_on_abort);
            return;
        }

        total_written += row_count as u64;
        emit(
            &app,
            &ExportEvent::Progress {
                job_id: job_id.clone(),
                rows_written: total_written,
                total_rows: total_row_count.map(|n| n as u64),
            },
        );

        let reached_total = total_row_count.map(|total| total_written >= total as u64).unwrap_or(true);
        if row_count == 0 || (row_count as u64) < chunk_size || reached_total {
            break;
        }
        offset += chunk_size;
    }

    let Some(writer) = writer else {
        emit(
            &app,
            &ExportEvent::Done { job_id, rows_written: 0, path: path.to_string_lossy().into_owned() },
        );
        return;
    };

    match writer.finish() {
        Ok(_) => emit(
            &app,
            &ExportEvent::Done { job_id, rows_written: total_written, path: path.to_string_lossy().into_owned() },
        ),
        Err(e) => {
            emit(&app, &ExportEvent::Error { job_id, message: e.to_string() });
            cleanup_on_abort(&path, request.delete_on_abort);
        }
    }
}
