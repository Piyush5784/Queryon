use trino_rust_client::client::{Client, ClientBuilder};

use crate::domain::connection::ConnectionProfile;
use crate::error::AppError;

pub const DEFAULT_CATALOG: &str = "system";
pub const DEFAULT_SCHEMA: &str = "runtime";

pub fn build_client(profile: &ConnectionProfile) -> Result<Client, AppError> {
    let host = profile.host.trim();
    if host.is_empty() {
        return Err(AppError::new("A Trino connection needs a host, e.g. localhost."));
    }
    let user = if profile.user.trim().is_empty() { "queryon".to_string() } else { profile.user.trim().to_string() };
    let port = if profile.port == 0 { 8080 } else { profile.port };
    let catalog = if profile.database.trim().is_empty() { DEFAULT_CATALOG.to_string() } else { profile.database.trim().to_string() };

    let mut builder = ClientBuilder::new(user.clone(), host.to_string()).port(port).catalog(catalog);
    if !profile.password.trim().is_empty() {
        builder = builder.auth(trino_rust_client::auth::Auth::Basic(user, Some(profile.password.clone()))).auth_http_insecure(true);
    }

    builder.build().map_err(|e| AppError::new(format!("Failed to build Trino client: {}", crate::error::describe_trino_error(&e))))
}
