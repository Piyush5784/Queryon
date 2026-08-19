use std::sync::Arc;
use std::time::Instant;

use tauri::{AppHandle, Wry};

use crate::domain::driver::DatabaseDriver;
use crate::error::AppError;
use crate::infrastructure::postgres::driver::PostgresDriver;
use crate::infrastructure::postgres::pool::build_pool;
use crate::infrastructure::storage::{credential_vault, profile_store};

use super::models::{ConnectionProfile, SavedConnectionProfile};

/// The one place an engine is chosen — every other layer (state, commands,
/// domain services) talks to connections only through `DatabaseDriver`.
pub async fn open_pool_and_verify(
    profile: &ConnectionProfile,
) -> Result<(Arc<dyn DatabaseDriver>, String), AppError> {
    let build_start = Instant::now();
    let pool = build_pool(profile)?;
    log::info!("db_connect: build_pool took {:?}", build_start.elapsed());

    let driver: Arc<dyn DatabaseDriver> = Arc::new(PostgresDriver::new(pool));

    let verify_start = Instant::now();
    let server_version = driver.server_version().await?;
    log::info!("db_connect: fetch_server_version took {:?}", verify_start.elapsed());

    Ok((driver, server_version))
}

pub fn save_connection(app: &AppHandle<Wry>, profile: &ConnectionProfile) -> Result<(), AppError> {
    credential_vault::save_password(&profile.id, &profile.password)?;
    profile_store::upsert(app, SavedConnectionProfile::from_profile(profile))
}

pub fn list_saved_connections(app: &AppHandle<Wry>) -> Result<Vec<SavedConnectionProfile>, AppError> {
    profile_store::list(app)
}

pub async fn load_saved_connection(
    app: &AppHandle<Wry>,
    connection_id: &str,
) -> Result<ConnectionProfile, AppError> {
    let profiles = profile_store::list(app)?;
    let saved = profiles
        .into_iter()
        .find(|p| p.id == connection_id)
        .ok_or_else(|| AppError::new("No saved connection found with this id."))?;
    let password = credential_vault::load_password(connection_id)?;
    Ok(saved.with_password(password))
}

pub fn delete_saved_connection(app: &AppHandle<Wry>, connection_id: &str) -> Result<(), AppError> {
    credential_vault::delete_password(connection_id)?;
    profile_store::remove(app, connection_id)
}
