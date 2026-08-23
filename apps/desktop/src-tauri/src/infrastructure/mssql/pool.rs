use deadpool::managed::{self, Metrics, Pool, RecycleResult};
use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use crate::domain::connection::{ConnectionProfile, SslMode};
use crate::error::AppError;

pub type MssqlClient = Client<Compat<TcpStream>>;
pub type MssqlPool = Pool<MssqlManager>;

/// `tiberius` has no built-in pool — see the roadmap doc's note ("pair
/// with `deadpool` generically, same pattern as Postgres") — so this
/// implements `deadpool::managed::Manager` directly. `Client` has no
/// health-check call of its own; `recycle` runs a trivial `SELECT 1`
/// (the standard idiom for pools around drivers without one, matching
/// `deadpool-postgres`'s own recycle behavior) to catch a connection the
/// server has silently dropped before handing it back out.
pub struct MssqlManager {
    config: Config,
}

impl managed::Manager for MssqlManager {
    type Type = MssqlClient;
    type Error = AppError;

    async fn create(&self) -> Result<MssqlClient, AppError> {
        let tcp = TcpStream::connect(self.config.get_addr())
            .await
            .map_err(|e| AppError::new(clean_mssql_connect_error(&e.to_string())))?;
        tcp.set_nodelay(true).ok();

        Client::connect(self.config.clone(), tcp.compat_write())
            .await
            .map_err(|e| AppError::new(clean_mssql_connect_error(&e.to_string())))
    }

    async fn recycle(&self, client: &mut MssqlClient, _: &Metrics) -> RecycleResult<AppError> {
        client
            .simple_query("SELECT 1")
            .await
            .map_err(|e| managed::RecycleError::Backend(AppError::new(e.to_string())))?;
        Ok(())
    }
}

/// `EncryptionLevel::Required` is TDS's own always-encrypt mode; `Off`
/// disables it entirely. `SslMode::Prefer`/`VerifyCa`/`VerifyFull` don't
/// map to any real distinction tiberius's TLS layer exposes beyond
/// require/trust-any-cert vs. require/verify — the `verify-*` modes are
/// treated as "require encryption, verify the certificate" and every
/// weaker mode as "require encryption, trust any certificate"
/// (`trust_cert()`), since a bare TCP connection to SQL Server without
/// TLS at all is the one mode this app never wants to silently choose.
pub async fn build_pool(profile: &ConnectionProfile) -> Result<MssqlPool, AppError> {
    let mut config = Config::new();
    config.host(&profile.host);
    config.port(profile.port);
    config.database(&profile.database);
    config.authentication(AuthMethod::sql_server(&profile.user, &profile.password));

    match profile.ssl_mode {
        SslMode::Disable => {
            config.trust_cert();
        }
        SslMode::VerifyCa | SslMode::VerifyFull => {}
        SslMode::Prefer | SslMode::Require => {
            config.trust_cert();
        }
    }

    let manager = MssqlManager { config };
    Pool::builder(manager)
        .max_size(5)
        .build()
        .map_err(|e| AppError::new(format!("Failed to create connection pool: {e}")))
}

pub fn clean_mssql_connect_error(raw: &str) -> String {
    if raw.contains("Login failed") {
        "Authentication failed — check your username and password.".to_string()
    } else if raw.contains("Connection refused") {
        "Connection refused — check the host and port, and that the server is running."
            .to_string()
    } else if raw.contains("timed out") || raw.contains("timeout") {
        "Connection timed out — check the host and port, and your network/firewall.".to_string()
    } else if raw.contains("Cannot open database") {
        "Database does not exist.".to_string()
    } else {
        raw.to_string()
    }
}
