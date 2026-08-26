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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Engine {
    Postgres,
    Neon,
    CockroachDb,
    GreengageDb,
    MySql,
    MariaDb,
    TiDb,
    Sqlite,
    SqlServer,
    StarRocks,
    ClickHouse,
    DuckDb,
    LibSql,
    Trino,
    MongoDb,
}

fn default_read_only() -> bool {
    false
}

#[derive(Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum SshAuth {
    #[serde(rename_all = "camelCase")]
    Password { password: String },
    #[serde(rename_all = "camelCase")]
    PrivateKey {
        key_path: String,
        /// Empty string means the key file is unencrypted.
        passphrase: String,
    },
}

impl std::fmt::Debug for SshAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SshAuth::Password { .. } => f.debug_struct("Password").field("password", &"<redacted>").finish(),
            SshAuth::PrivateKey { key_path, .. } => f
                .debug_struct("PrivateKey")
                .field("key_path", key_path)
                .field("passphrase", &"<redacted>")
                .finish(),
        }
    }
}

#[derive(Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SshTunnelConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: SshAuth,
}

impl std::fmt::Debug for SshTunnelConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SshTunnelConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("username", &self.username)
            .field("auth", &self.auth)
            .finish()
    }
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
    #[serde(default = "default_read_only")]
    pub read_only: bool,
    #[serde(default)]
    pub ssh_tunnel: Option<SshTunnelConfig>,
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
            .field("read_only", &self.read_only)
            .field("ssh_tunnel", &self.ssh_tunnel)
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SshAuthKind {
    Password,
    PrivateKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SavedSshTunnelConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_kind: SshAuthKind,
    /// Only meaningful when `auth_kind` is `PrivateKey` — a file path is
    /// not a secret, so unlike the passphrase it's fine to keep in the
    /// plain connections store.
    pub key_path: Option<String>,
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
    #[serde(default = "default_read_only")]
    pub read_only: bool,
    #[serde(default)]
    pub ssh_tunnel: Option<SavedSshTunnelConfig>,
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
            read_only: profile.read_only,
            ssh_tunnel: profile.ssh_tunnel.as_ref().map(|t| SavedSshTunnelConfig {
                host: t.host.clone(),
                port: t.port,
                username: t.username.clone(),
                auth_kind: match &t.auth {
                    SshAuth::Password { .. } => SshAuthKind::Password,
                    SshAuth::PrivateKey { .. } => SshAuthKind::PrivateKey,
                },
                key_path: match &t.auth {
                    SshAuth::PrivateKey { key_path, .. } => Some(key_path.clone()),
                    SshAuth::Password { .. } => None,
                },
            }),
        }
    }

    pub fn with_password(self, password: String, ssh_secret: Option<String>) -> ConnectionProfile {
        let ssh_tunnel = self.ssh_tunnel.map(|t| SshTunnelConfig {
            host: t.host,
            port: t.port,
            username: t.username,
            auth: match t.auth_kind {
                SshAuthKind::Password => SshAuth::Password {
                    password: ssh_secret.unwrap_or_default(),
                },
                SshAuthKind::PrivateKey => SshAuth::PrivateKey {
                    key_path: t.key_path.unwrap_or_default(),
                    passphrase: ssh_secret.unwrap_or_default(),
                },
            },
        });

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
            read_only: self.read_only,
            ssh_tunnel,
        }
    }
}
