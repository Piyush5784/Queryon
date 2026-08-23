use queryon_lib::domain::connection::{SshAuth, SshTunnelConfig};
use queryon_lib::infrastructure::ssh;

fn bastion_config() -> SshTunnelConfig {
    SshTunnelConfig {
        host: "localhost".to_string(),
        port: 22222,
        username: "tunneluser".to_string(),
        auth: SshAuth::Password { password: "tunnelpass".to_string() },
    }
}

/// `queryon-pg` (the real Postgres container) is only reachable by name
/// from inside the `queryon-ssh-bastion` container's network, not from
/// this test's own host — exactly mirroring how a real production
/// database sits on a private network only the bastion can reach. If
/// the tunnel actually works, connecting through it to "queryon-pg:5432"
/// succeeds even though a direct connection from here would fail.
#[tokio::test]
async fn open_tunnel_forwards_to_a_host_only_reachable_from_the_bastion() {
    let config = bastion_config();
    let tunnel = ssh::open_tunnel(&config, "queryon-pg", 5432)
        .await
        .expect("tunnel should establish and authenticate");

    let stream = tokio::net::TcpStream::connect(tunnel.local_addr)
        .await
        .expect("should be able to connect to the local forwarded port");
    drop(stream);

    tunnel.shutdown();
}

#[tokio::test]
async fn open_tunnel_errors_on_wrong_password() {
    let mut config = bastion_config();
    config.auth = SshAuth::Password { password: "wrong-password".to_string() };

    let result = ssh::open_tunnel(&config, "queryon-pg", 5432).await;
    assert!(result.is_err(), "wrong SSH password should fail to authenticate");
}

#[tokio::test]
async fn open_tunnel_errors_on_unreachable_bastion() {
    let mut config = bastion_config();
    config.port = 1; // nothing listens here

    let result = ssh::open_tunnel(&config, "queryon-pg", 5432).await;
    assert!(result.is_err(), "an unreachable SSH host should fail to connect");
}

/// End-to-end: forward through the tunnel and actually speak the
/// Postgres wire protocol on the other end, proving the byte-for-byte
/// relay (not just the initial TCP accept) works both directions.
#[tokio::test]
async fn tunnel_carries_a_real_postgres_handshake() {
    let config = bastion_config();
    let tunnel = ssh::open_tunnel(&config, "queryon-pg", 5432)
        .await
        .expect("tunnel should establish");

    let profile = queryon_lib::domain::connection::ConnectionProfile {
        id: "tunnel-test".to_string(),
        name: "tunnel-test".to_string(),
        engine: queryon_lib::domain::connection::Engine::Postgres,
        host: tunnel.local_addr.ip().to_string(),
        port: tunnel.local_addr.port(),
        database: "devdb".to_string(),
        user: "devuser".to_string(),
        password: "devpass".to_string(),
        ssl_mode: queryon_lib::domain::connection::SslMode::Disable,
        read_only: false,
        ssh_tunnel: None,
    };

    let pool = queryon_lib::infrastructure::postgres::pool::build_pool(&profile)
        .expect("pool should build against the local forwarded port");
    let driver = queryon_lib::infrastructure::postgres::driver::PostgresDriver::new(pool);

    let version = queryon_lib::domain::driver::DatabaseDriver::server_version(&driver)
        .await
        .expect("should be able to query the real Postgres server through the tunnel");
    assert!(version.starts_with("16"), "expected a Postgres 16.x version string, got {version:?}");

    tunnel.shutdown();
}

/// Sends a real Postgres SSL-negotiation request through the tunnel and
/// checks the exact single-byte reply comes back — proves the relay is
/// byte-exact in both directions, not just able to open a TCP session.
#[tokio::test]
async fn tunnel_relays_bytes_exactly() {
    let config = bastion_config();
    let tunnel = ssh::open_tunnel(&config, "queryon-pg", 5432)
        .await
        .expect("tunnel should establish");

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stream = tokio::net::TcpStream::connect(tunnel.local_addr)
        .await
        .expect("should connect to the local forwarded port");

    // Minimal Postgres startup packet requesting SSL (code 80877103), like real clients send first.
    let ssl_request: [u8; 8] = [0, 0, 0, 8, 4, 210, 22, 47];
    stream.write_all(&ssl_request).await.expect("write should succeed");
    stream.flush().await.expect("flush should succeed");

    let mut buf = [0u8; 1];
    tokio::time::timeout(std::time::Duration::from_secs(5), stream.read_exact(&mut buf))
        .await
        .expect("read should not time out")
        .expect("read should succeed");
    // devdb's queryon-pg container runs with SSL disabled, so Postgres
    // replies 'N' (not supported) rather than 'S' (switch to TLS).
    assert_eq!(buf[0], b'N', "expected Postgres's plain SSL-declined reply");

    tunnel.shutdown();
}

#[tokio::test]
async fn open_tunnel_authenticates_with_a_private_key() {
    let mut config = bastion_config();
    config.auth = SshAuth::PrivateKey {
        key_path: concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test_ssh_key").to_string(),
        passphrase: String::new(),
    };

    let tunnel = ssh::open_tunnel(&config, "queryon-pg", 5432)
        .await
        .expect("tunnel should establish and authenticate with the private key");
    tunnel.shutdown();
}
