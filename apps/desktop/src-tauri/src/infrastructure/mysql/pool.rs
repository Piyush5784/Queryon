use sqlx::mysql::{MySqlConnectOptions, MySqlPool, MySqlPoolOptions, MySqlSslMode};

use crate::domain::connection::{ConnectionProfile, Engine, SslMode};
use crate::error::AppError;

pub async fn build_pool(profile: &ConnectionProfile) -> Result<MySqlPool, AppError> {
    let mut options = MySqlConnectOptions::new()
        .host(&profile.host)
        .port(profile.port)
        .username(&profile.user)
        .password(&profile.password)
        .database(&profile.database)
        .ssl_mode(to_mysql_ssl_mode(profile.ssl_mode));

    if profile.engine == Engine::StarRocks {
        options = options.no_engine_substitution(false).pipes_as_concat(false);
    }

    MySqlPoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .map_err(|e| AppError::new(format!("Failed to create connection pool: {e}")))
}

fn to_mysql_ssl_mode(mode: SslMode) -> MySqlSslMode {
    match mode {
        SslMode::Disable => MySqlSslMode::Disabled,
        SslMode::Prefer => MySqlSslMode::Preferred,
        SslMode::Require => MySqlSslMode::Required,
        SslMode::VerifyCa => MySqlSslMode::VerifyCa,
        SslMode::VerifyFull => MySqlSslMode::VerifyIdentity,
    }
}
