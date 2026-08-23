use std::sync::Mutex;

use duckdb::Connection;

use crate::domain::connection::ConnectionProfile;
use crate::error::AppError;

pub struct DuckDbHandle {
    pub conn: Mutex<Connection>,
}

pub fn build_connection(profile: &ConnectionProfile) -> Result<DuckDbHandle, AppError> {
    let conn = if profile.database == ":memory:" {
        Connection::open_in_memory()
    } else {
        Connection::open(&profile.database)
    }
    .map_err(|e| AppError::new(format!("Failed to open DuckDB database: {e}")))?;

    Ok(DuckDbHandle { conn: Mutex::new(conn) })
}
