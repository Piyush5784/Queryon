use tauri::{AppHandle, State, Wry};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};

use crate::domain::connection::service;
use crate::domain::connection::{ConnectionInfo, ConnectionProfile, SavedConnectionProfile};
use crate::error::AppError;
use crate::state::ConnectionRegistry;

#[tauri::command]
#[specta::specta]
pub async fn db_connect(
    profile: ConnectionProfile,
    registry: State<'_, ConnectionRegistry>,
) -> Result<ConnectionInfo, AppError> {
    let (driver, server_version, tunnel) = service::open_pool_and_verify(&profile).await?;
    match tunnel {
        Some(tunnel) => registry.insert_with_tunnel(profile.id.clone(), driver, profile.read_only, tunnel),
        None => registry.insert(profile.id.clone(), driver, profile.read_only),
    }

    Ok(ConnectionInfo {
        id: profile.id,
        server_version,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn db_test_connection(profile: ConnectionProfile) -> Result<ConnectionInfo, AppError> {
    let (_driver, server_version, tunnel) = service::open_pool_and_verify(&profile).await?;
    if let Some(tunnel) = tunnel {
        tunnel.shutdown();
    }

    Ok(ConnectionInfo {
        id: profile.id,
        server_version,
    })
}

#[tauri::command]
#[specta::specta]
pub fn db_disconnect(connection_id: String, registry: State<'_, ConnectionRegistry>) {
    registry.remove(&connection_id);
}

#[tauri::command]
#[specta::specta]
pub fn db_list_active_connections(registry: State<'_, ConnectionRegistry>) -> Vec<String> {
    registry.ids()
}

#[tauri::command]
#[specta::specta]
pub fn db_save_connection(app: AppHandle<Wry>, profile: ConnectionProfile) -> Result<(), AppError> {
    service::save_connection(&app, &profile)
}

#[tauri::command]
#[specta::specta]
pub fn db_list_saved_connections(
    app: AppHandle<Wry>,
) -> Result<Vec<SavedConnectionProfile>, AppError> {
    service::list_saved_connections(&app)
}

#[tauri::command]
#[specta::specta]
pub async fn db_connect_saved(
    app: AppHandle<Wry>,
    connection_id: String,
    registry: State<'_, ConnectionRegistry>,
) -> Result<ConnectionInfo, AppError> {
    let profile = service::load_saved_connection(&app, &connection_id).await?;
    let (driver, server_version, tunnel) = service::open_pool_and_verify(&profile).await?;
    match tunnel {
        Some(tunnel) => registry.insert_with_tunnel(profile.id.clone(), driver, profile.read_only, tunnel),
        None => registry.insert(profile.id.clone(), driver, profile.read_only),
    }

    Ok(ConnectionInfo {
        id: profile.id,
        server_version,
    })
}

#[tauri::command]
#[specta::specta]
pub fn db_delete_saved_connection(
    app: AppHandle<Wry>,
    connection_id: String,
) -> Result<(), AppError> {
    service::delete_saved_connection(&app, &connection_id)
}

#[tauri::command]
#[specta::specta]
pub fn db_rename_saved_connection(
    app: AppHandle<Wry>,
    connection_id: String,
    name: String,
) -> Result<(), AppError> {
    service::rename_saved_connection(&app, &connection_id, &name)
}

/// Opens the OS's native file picker for choosing an SSH private key
/// file. Returns `None` if the user cancels.
#[tauri::command]
#[specta::specta]
pub async fn ssh_pick_key_file(app: AppHandle<Wry>) -> Result<Option<String>, AppError> {
    let builder = FileDialogBuilder::new(app.dialog().clone());

    let (tx, rx) = tokio::sync::oneshot::channel();
    builder.pick_file(move |path| {
        let _ = tx.send(path);
    });

    let chosen = rx
        .await
        .map_err(|_| AppError::new("File picker closed unexpectedly."))?;

    let Some(path) = chosen else {
        return Ok(None);
    };

    let path = path
        .into_path()
        .map_err(|e| AppError::new(format!("Invalid file: {e}")))?;

    Ok(Some(path.to_string_lossy().into_owned()))
}

#[tauri::command]
#[specta::specta]
pub async fn db_pick_sqlite_file(app: AppHandle<Wry>) -> Result<Option<String>, AppError> {
    let builder = FileDialogBuilder::new(app.dialog().clone())
        .add_filter("SQLite database", &["db", "sqlite", "sqlite3"])
        .add_filter("All files", &["*"]);

    let (tx, rx) = tokio::sync::oneshot::channel();
    builder.pick_file(move |path| {
        let _ = tx.send(path);
    });

    let chosen = rx
        .await
        .map_err(|_| AppError::new("File picker closed unexpectedly."))?;

    let Some(path) = chosen else {
        return Ok(None);
    };

    let path = path
        .into_path()
        .map_err(|e| AppError::new(format!("Invalid file: {e}")))?;

    Ok(Some(path.to_string_lossy().into_owned()))
}

#[tauri::command]
#[specta::specta]
pub async fn db_pick_duckdb_file(app: AppHandle<Wry>) -> Result<Option<String>, AppError> {
    let builder = FileDialogBuilder::new(app.dialog().clone())
        .add_filter("DuckDB database", &["duckdb", "db"])
        .add_filter("All files", &["*"]);

    let (tx, rx) = tokio::sync::oneshot::channel();
    builder.pick_file(move |path| {
        let _ = tx.send(path);
    });

    let chosen = rx
        .await
        .map_err(|_| AppError::new("File picker closed unexpectedly."))?;

    let Some(path) = chosen else {
        return Ok(None);
    };

    let path = path
        .into_path()
        .map_err(|e| AppError::new(format!("Invalid file: {e}")))?;

    Ok(Some(path.to_string_lossy().into_owned()))
}
