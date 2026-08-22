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
/// `CockroachDb` is the same story: it speaks the Postgres wire protocol
/// and its `pg_catalog` shim is close enough that `PostgresDriver`'s
/// metadata queries work unmodified (verified against a live CockroachDB
/// container in `tests/cockroachdb_metadata.rs`) — matching how
/// Beekeeper Studio's own `CockroachClient extends PostgresClient`
/// (`temp/apps/studio/src/lib/db/clients/cockroach.ts`) only overrides
/// the handful of queries that actually diverge.
///
/// `MariaDb` maps to `MySqlDriver` the same way — it speaks the MySQL
/// wire protocol, matching Beekeeper's `MariaDBClient extends
/// MysqlClient`. One real divergence found by testing against a live
/// MariaDB container: `information_schema.columns.column_default`
/// returns a quoted-string default (e.g. `'it''s a test'`) still wrapped
/// in literal quotes with doubled internal quotes, where real MySQL
/// returns the bare unescaped string — see
/// `infrastructure::mysql::metadata::unquote_mariadb_default` and
/// `tests/mariadb_metadata.rs`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Engine {
    Postgres,
    Neon,
    CockroachDb,
    MySql,
    MariaDb,
}

fn default_read_only() -> bool {
    false
}

/// How to authenticate to the SSH bastion host in `SshTunnelConfig`.
/// Distinct from `ConnectionProfile::password`, which authenticates to
/// the *database* — this authenticates to the *jump server* the tunnel
/// is opened through.
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

/// Routes the database connection through an SSH bastion/jump host via
/// local port forwarding, for databases with no directly reachable
/// address (the common case for production/staging databases sitting in
/// a private network) — see `infrastructure::ssh::tunnel`.
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
    /// When true, every write command (schema DDL, row insert/update/
    /// delete) is rejected at the IPC layer for this connection — see
    /// `state::ConnectionRegistry::require_writable`. A safety net for
    /// connecting to a database you want to look at but not touch,
    /// independent of what the DB user's actual grants allow.
    #[serde(default = "default_read_only")]
    pub read_only: bool,
    /// When set, `host`/`port` above are only ever used as the *target*
    /// the SSH tunnel forwards to — the actual TCP connection this app
    /// makes goes to a local forwarded port instead. See
    /// `domain::connection::service::open_pool_and_verify`.
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

    /// Reassembles a full `ConnectionProfile`, pairing this saved
    /// profile's non-secret fields with the database password and (if
    /// this profile uses a tunnel) the SSH password/passphrase, both
    /// loaded from `credential_vault` by the caller.
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
