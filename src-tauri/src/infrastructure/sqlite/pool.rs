use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

use crate::domain::connection::ConnectionProfile;
use crate::error::AppError;

/// `profile.database` holds the SQLite file's path for this engine — see
/// `Engine::Sqlite`'s doc comment. `":memory:"` opens a private in-memory
/// database, matching Beekeeper's own `isTempDB` handling.
///
/// `SqlitePoolOptions::max_connections(1)` is deliberate, not the same
/// `5` every other engine's pool uses: SQLite allows only one writer at a
/// time per file regardless of how many connections are open (extra
/// connections just contend for the same file lock and surface as
/// `SQLITE_BUSY` "database is locked" errors under concurrent writes,
/// confirmed by testing with a higher pool size), and this driver never
/// needs read/write connections to overlap the way a real client-server
/// engine's pool does.
pub async fn build_pool(profile: &ConnectionProfile) -> Result<SqlitePool, AppError> {
    let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", profile.database))
        .map_err(|e| AppError::new(format!("Invalid SQLite file path: {e}")))?
        .create_if_missing(false)
        .foreign_keys(true);

    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|e| AppError::new(format!("Failed to open SQLite database: {e}")))
}
