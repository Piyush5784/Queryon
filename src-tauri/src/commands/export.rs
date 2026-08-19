use tauri::{AppHandle, Manager, State, Wry};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};

use crate::domain::export::{service, RowsExportRequest, TableExportRequest};
use crate::error::AppError;
use crate::state::{ConnectionRegistry, ExportJobRegistry};

/// Returns the OS's Downloads folder (or the home directory as a
/// fallback) to prepopulate the export dialog's output directory field.
#[tauri::command]
#[specta::specta]
pub fn export_default_directory(app: AppHandle<Wry>) -> Result<String, AppError> {
    let path = app
        .path()
        .download_dir()
        .or_else(|_| app.path().home_dir())
        .map_err(|e| AppError::new(format!("Could not resolve a default folder: {e}")))?;

    Ok(path.to_string_lossy().into_owned())
}

/// Opens the OS's native folder picker, defaulted to `initial_directory`
/// when given. Returns `None` if the user cancels.
#[tauri::command]
#[specta::specta]
pub async fn export_pick_directory(
    app: AppHandle<Wry>,
    initial_directory: Option<String>,
) -> Result<Option<String>, AppError> {
    let mut builder = FileDialogBuilder::new(app.dialog().clone());
    if let Some(dir) = initial_directory.filter(|d| !d.is_empty()) {
        builder = builder.set_directory(dir);
    }

    let (tx, rx) = tokio::sync::oneshot::channel();
    builder.pick_folder(move |path| {
        let _ = tx.send(path);
    });

    let chosen = rx
        .await
        .map_err(|_| AppError::new("Folder picker closed unexpectedly."))?;

    let Some(path) = chosen else {
        return Ok(None);
    };

    let path = path
        .into_path()
        .map_err(|e| AppError::new(format!("Invalid folder: {e}")))?;

    Ok(Some(path.to_string_lossy().into_owned()))
}

/// Starts a background chunked export of a table's rows, streaming
/// directly from the database. Returns immediately with a job id; progress
/// arrives via `export:progress`/`export:done`/`export:error`/
/// `export:cancelled` events carrying that same job id.
#[tauri::command]
#[specta::specta]
pub fn export_run_table(
    app: AppHandle<Wry>,
    job_id: String,
    request: TableExportRequest,
    connections: State<'_, ConnectionRegistry>,
    jobs: State<'_, ExportJobRegistry>,
) -> Result<(), AppError> {
    let driver = connections
        .get(&request.connection_id)
        .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))?;

    let cancel_flag = jobs.register(job_id.clone());

    tauri::async_runtime::spawn(async move {
        service::run_table_export(app, job_id, request, driver, cancel_flag).await;
    });

    Ok(())
}

/// Starts a background export of already-fetched rows (e.g. a query
/// result). No DB access — just writes what the frontend already has, in
/// chunks, off the main thread so the UI never blocks.
#[tauri::command]
#[specta::specta]
pub fn export_run_rows(
    app: AppHandle<Wry>,
    job_id: String,
    request: RowsExportRequest,
    jobs: State<'_, ExportJobRegistry>,
) -> Result<(), AppError> {
    let cancel_flag = jobs.register(job_id.clone());

    tauri::async_runtime::spawn(async move {
        service::run_rows_export(app, job_id, request, cancel_flag).await;
    });

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn export_cancel(job_id: String, jobs: State<'_, ExportJobRegistry>) {
    jobs.cancel(&job_id);
}
