use crate::domain::connection::ConnectionProfile;
use crate::error::AppError;

pub async fn build_connection(profile: &ConnectionProfile) -> Result<libsql::Connection, AppError> {
    let url = profile.host.trim();
    if url.is_empty() {
        return Err(AppError::new("A LibSQL connection needs a server URL, e.g. http://localhost:58082."));
    }

    let database = libsql::Builder::new_remote(url.to_string(), profile.password.clone())
        .build()
        .await
        .map_err(|e| AppError::new(format!("Failed to reach LibSQL server: {}", crate::error::describe_libsql_error(&e))))?;

    database
        .connect()
        .map_err(|e| AppError::new(format!("Failed to open LibSQL connection: {}", crate::error::describe_libsql_error(&e))))
}
