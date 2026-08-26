use std::sync::Arc;

use tauri::{AppHandle, State, Wry};

use crate::domain::connection::{service, ConnectionInfo, ConnectionProfile};
use crate::domain::document::{CollectionRef, DatabaseRef, DocumentDriver, DocumentPage};
use crate::error::AppError;
use crate::state::DocumentConnectionRegistry;

fn driver_for(
    registry: &State<'_, DocumentConnectionRegistry>,
    connection_id: &str,
) -> Result<Arc<dyn DocumentDriver>, AppError> {
    registry
        .get(connection_id)
        .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))
}

#[tauri::command]
#[specta::specta]
pub async fn doc_connect(
    profile: ConnectionProfile,
    registry: State<'_, DocumentConnectionRegistry>,
) -> Result<ConnectionInfo, AppError> {
    let (driver, server_version, default_database) = service::open_document_connection(&profile).await?;
    registry.insert(profile.id.clone(), driver, default_database);

    Ok(ConnectionInfo {
        id: profile.id,
        server_version,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn doc_test_connection(profile: ConnectionProfile) -> Result<ConnectionInfo, AppError> {
    let (_driver, server_version, _default_database) = service::open_document_connection(&profile).await?;

    Ok(ConnectionInfo {
        id: profile.id,
        server_version,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn doc_connect_saved(
    app: AppHandle<Wry>,
    connection_id: String,
    registry: State<'_, DocumentConnectionRegistry>,
) -> Result<ConnectionInfo, AppError> {
    let profile = service::load_saved_connection(&app, &connection_id).await?;
    let (driver, server_version, default_database) = service::open_document_connection(&profile).await?;
    registry.insert(profile.id.clone(), driver, default_database);

    Ok(ConnectionInfo {
        id: profile.id,
        server_version,
    })
}

#[tauri::command]
#[specta::specta]
pub fn doc_disconnect(connection_id: String, registry: State<'_, DocumentConnectionRegistry>) {
    registry.remove(&connection_id);
}

#[tauri::command]
#[specta::specta]
pub fn doc_list_active_connections(registry: State<'_, DocumentConnectionRegistry>) -> Vec<String> {
    registry.ids()
}

#[tauri::command]
#[specta::specta]
pub fn doc_default_database(
    connection_id: String,
    registry: State<'_, DocumentConnectionRegistry>,
) -> Option<String> {
    registry.default_database(&connection_id)
}

#[tauri::command]
#[specta::specta]
pub async fn doc_list_databases(
    connection_id: String,
    registry: State<'_, DocumentConnectionRegistry>,
) -> Result<Vec<DatabaseRef>, AppError> {
    let driver = driver_for(&registry, &connection_id)?;
    driver.list_databases().await
}

#[tauri::command]
#[specta::specta]
pub async fn doc_list_collections(
    connection_id: String,
    database: String,
    registry: State<'_, DocumentConnectionRegistry>,
) -> Result<Vec<CollectionRef>, AppError> {
    let driver = driver_for(&registry, &connection_id)?;
    driver.list_collections(&database).await
}

#[tauri::command]
#[specta::specta]
pub async fn doc_list_documents(
    connection_id: String,
    database: String,
    collection: String,
    limit: i32,
    skip: i32,
    registry: State<'_, DocumentConnectionRegistry>,
) -> Result<DocumentPage, AppError> {
    let driver = driver_for(&registry, &connection_id)?;
    driver
        .list_documents(&database, &collection, limit as i64, skip as i64)
        .await
}

#[tauri::command]
#[specta::specta]
pub async fn doc_get_document(
    connection_id: String,
    database: String,
    collection: String,
    id: String,
    registry: State<'_, DocumentConnectionRegistry>,
) -> Result<Option<String>, AppError> {
    let driver = driver_for(&registry, &connection_id)?;
    let value = driver.get_document(&database, &collection, &id).await?;
    match value {
        Some(value) => {
            let encoded = serde_json::to_string(&value)
                .map_err(|e| AppError::new(format!("Failed to encode document: {e}")))?;
            Ok(Some(encoded))
        }
        None => Ok(None),
    }
}
