use mongodb::Client;

use crate::domain::connection::ConnectionProfile;
use crate::error::AppError;

pub async fn build_client(profile: &ConnectionProfile) -> Result<(Client, String), AppError> {
    let uri = profile.host.trim();
    if uri.is_empty() {
        return Err(AppError::new(
            "A MongoDB connection needs a connection string, e.g. mongodb+srv://user:pass@cluster.mongodb.net/mydb.",
        ));
    }

    let client = Client::with_uri_str(uri)
        .await
        .map_err(|e| AppError::new(format!("Failed to reach MongoDB server: {}", crate::error::describe_mongodb_error(&e))))?;

    let default_database = if !profile.database.trim().is_empty() {
        profile.database.trim().to_string()
    } else {
        client
            .default_database()
            .map(|db| db.name().to_string())
            .unwrap_or_default()
    };

    Ok((client, default_database))
}
