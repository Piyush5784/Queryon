use tauri::State;

use crate::domain::connection::service;
use crate::domain::connection::{ConnectionInfo, ConnectionProfile};
use crate::error::AppError;
use crate::state::ConnectionRegistry;

#[tauri::command]
#[specta::specta]
pub async fn db_connect(
    profile: ConnectionProfile,
    registry: State<'_, ConnectionRegistry>,
) -> Result<ConnectionInfo, AppError> {
    let (pool, server_version) = service::open_pool_and_verify(&profile).await?;
    registry.insert(profile.id.clone(), pool);

    Ok(ConnectionInfo {
        id: profile.id,
        server_version,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn db_test_connection(profile: ConnectionProfile) -> Result<ConnectionInfo, AppError> {
    let (_pool, server_version) = service::open_pool_and_verify(&profile).await?;

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
