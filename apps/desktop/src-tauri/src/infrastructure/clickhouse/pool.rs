use reqwest::Client;

use crate::domain::connection::{ConnectionProfile, SslMode};
use crate::error::AppError;

#[derive(Clone)]
pub struct ClickHouseClient {
    pub(super) http: Client,
    pub(super) base_url: String,
    pub(super) user: String,
    pub(super) password: String,
    pub(super) database: String,
}

impl ClickHouseClient {
    pub fn database(&self) -> &str {
        &self.database
    }
}

pub async fn build_client(profile: &ConnectionProfile) -> Result<ClickHouseClient, AppError> {
    let scheme = if profile.ssl_mode == SslMode::Disable { "http" } else { "https" };
    let base_url = format!("{scheme}://{}:{}", profile.host, profile.port);

    let http = Client::builder()
        .build()
        .map_err(|e| AppError::new(format!("Failed to build HTTP client: {e}")))?;

    let client = ClickHouseClient {
        http,
        base_url,
        user: profile.user.clone(),
        password: profile.password.clone(),
        database: profile.database.clone(),
    };

    client.ping().await?;
    Ok(client)
}
