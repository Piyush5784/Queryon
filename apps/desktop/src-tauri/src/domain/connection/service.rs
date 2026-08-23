use std::sync::Arc;
use std::time::Instant;

use tauri::{AppHandle, Wry};

use crate::domain::driver::DatabaseDriver;
use crate::error::AppError;
use crate::infrastructure::clickhouse::driver::ClickHouseDriver;
use crate::infrastructure::duckdb::driver::DuckDbDriver;
use crate::infrastructure::mssql::driver::MssqlDriver;
use crate::infrastructure::mysql::driver::MySqlDriver;
use crate::infrastructure::postgres::driver::PostgresDriver;
use crate::infrastructure::sqlite::driver::SqliteDriver;
use crate::infrastructure::ssh::{self, SshTunnel};
use crate::infrastructure::storage::{credential_vault, profile_store};

use super::models::{ConnectionProfile, Engine, SavedConnectionProfile};

pub async fn open_pool_and_verify(
    profile: &ConnectionProfile,
) -> Result<(Arc<dyn DatabaseDriver>, String, Option<SshTunnel>), AppError> {
    let build_start = Instant::now();

    let (dial_profile, tunnel) = match &profile.ssh_tunnel {
        Some(tunnel_config) => {
            let tunnel = ssh::open_tunnel(tunnel_config, &profile.host, profile.port).await?;
            let mut dial_profile = profile.clone();
            dial_profile.host = tunnel.local_addr.ip().to_string();
            dial_profile.port = tunnel.local_addr.port();
            (dial_profile, Some(tunnel))
        }
        None => (profile.clone(), None),
    };

    let driver: Arc<dyn DatabaseDriver> = match dial_profile.engine {
        Engine::Postgres | Engine::Neon | Engine::CockroachDb | Engine::GreengageDb => {
            let pool = crate::infrastructure::postgres::pool::build_pool(&dial_profile)?;
            Arc::new(PostgresDriver::new(pool))
        }
        Engine::MySql | Engine::TiDb => {
            let pool = crate::infrastructure::mysql::pool::build_pool(&dial_profile).await?;
            Arc::new(MySqlDriver::new(pool, dial_profile.database.clone()))
        }
        Engine::MariaDb => {
            let pool = crate::infrastructure::mysql::pool::build_pool(&dial_profile).await?;
            Arc::new(MySqlDriver::new_mariadb(pool, dial_profile.database.clone()))
        }
        Engine::StarRocks => {
            let pool = crate::infrastructure::mysql::pool::build_pool(&dial_profile).await?;
            Arc::new(MySqlDriver::new_starrocks(pool, dial_profile.database.clone()))
        }
        Engine::Sqlite => {
            let pool = crate::infrastructure::sqlite::pool::build_pool(&dial_profile).await?;
            Arc::new(SqliteDriver::new(pool))
        }
        Engine::SqlServer => {
            let pool = crate::infrastructure::mssql::pool::build_pool(&dial_profile).await?;
            Arc::new(MssqlDriver::new(pool))
        }
        Engine::ClickHouse => {
            let client = crate::infrastructure::clickhouse::pool::build_client(&dial_profile).await?;
            Arc::new(ClickHouseDriver::new(client))
        }
        Engine::DuckDb => {
            let handle = crate::infrastructure::duckdb::pool::build_connection(&dial_profile)?;
            Arc::new(DuckDbDriver::new(handle))
        }
    };
    log::info!("db_connect: build_pool took {:?}", build_start.elapsed());

    let verify_start = Instant::now();
    let server_version = driver.server_version().await?;
    log::info!("db_connect: fetch_server_version took {:?}", verify_start.elapsed());

    Ok((driver, server_version, tunnel))
}

pub fn save_connection(app: &AppHandle<Wry>, profile: &ConnectionProfile) -> Result<(), AppError> {
    credential_vault::save_password(&profile.id, &profile.password)?;
    if let Some(tunnel) = &profile.ssh_tunnel {
        let secret = match &tunnel.auth {
            super::models::SshAuth::Password { password } => password,
            super::models::SshAuth::PrivateKey { passphrase, .. } => passphrase,
        };
        credential_vault::save_ssh_secret(&profile.id, secret)?;
    } else {
        credential_vault::delete_ssh_secret(&profile.id)?;
    }
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
    let ssh_secret = if saved.ssh_tunnel.is_some() {
        Some(credential_vault::load_ssh_secret(connection_id)?)
    } else {
        None
    };
    Ok(saved.with_password(password, ssh_secret))
}

pub fn delete_saved_connection(app: &AppHandle<Wry>, connection_id: &str) -> Result<(), AppError> {
    credential_vault::delete_password(connection_id)?;
    credential_vault::delete_ssh_secret(connection_id)?;
    profile_store::remove(app, connection_id)
}

pub fn rename_saved_connection(
    app: &AppHandle<Wry>,
    connection_id: &str,
    name: &str,
) -> Result<(), AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::new("Connection name cannot be empty."));
    }
    profile_store::rename(app, connection_id, name)
}
