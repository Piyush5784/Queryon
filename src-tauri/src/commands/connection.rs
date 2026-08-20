use tauri::{AppHandle, State, Wry};

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
    let (driver, server_version) = service::open_pool_and_verify(&profile).await?;
    registry.insert(profile.id.clone(), driver);

    Ok(ConnectionInfo {
        id: profile.id,
        server_version,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn db_test_connection(profile: ConnectionProfile) -> Result<ConnectionInfo, AppError> {
    let (_driver, server_version) = service::open_pool_and_verify(&profile).await?;

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
pub fn db_list_saved_connections(app: AppHandle<Wry>) -> Result<Vec<SavedConnectionProfile>, AppError> {
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
    let (driver, server_version) = service::open_pool_and_verify(&profile).await?;
    registry.insert(profile.id.clone(), driver);

    Ok(ConnectionInfo {
        id: profile.id,
        server_version,
    })
}

#[tauri::command]
#[specta::specta]
pub fn db_delete_saved_connection(app: AppHandle<Wry>, connection_id: String) -> Result<(), AppError> {
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
