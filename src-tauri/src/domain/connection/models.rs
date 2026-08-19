use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
    VerifyCa,
    VerifyFull,
}

/// Which `DatabaseDriver` a profile connects through. `Neon` is not a
/// distinct wire protocol — it's Postgres with Neon-friendly defaults
/// (see `connections/types.ts`'s draft builder) — so it maps to the same
/// `PostgresDriver` as `Postgres` in `domain/connection/service.rs`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Engine {
    Postgres,
    Neon,
    MySql,
}

#[derive(Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionProfile {
    pub id: String,
    pub name: String,
    pub engine: Engine,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: String,
    pub ssl_mode: SslMode,
}

impl std::fmt::Debug for ConnectionProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionProfile")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("engine", &self.engine)
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database", &self.database)
            .field("user", &self.user)
            .field("password", &"<redacted>")
            .field("ssl_mode", &self.ssl_mode)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInfo {
    pub id: String,
    pub server_version: String,
}

fn default_engine() -> Engine {
    Engine::Postgres
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SavedConnectionProfile {
    pub id: String,
    pub name: String,
    #[serde(default = "default_engine")]
    pub engine: Engine,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub ssl_mode: SslMode,
}

impl SavedConnectionProfile {
    pub fn from_profile(profile: &ConnectionProfile) -> Self {
        Self {
            id: profile.id.clone(),
            name: profile.name.clone(),
            engine: profile.engine,
            host: profile.host.clone(),
            port: profile.port,
            database: profile.database.clone(),
            user: profile.user.clone(),
            ssl_mode: profile.ssl_mode,
        }
    }

    pub fn with_password(self, password: String) -> ConnectionProfile {
        ConnectionProfile {
            id: self.id,
            name: self.name,
            engine: self.engine,
            host: self.host,
            port: self.port,
            database: self.database,
            user: self.user,
            password,
            ssl_mode: self.ssl_mode,
        }
    }
}
