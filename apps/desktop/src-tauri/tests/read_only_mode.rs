use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::query::service as query_service;
use queryon_lib::infrastructure::postgres::driver::PostgresDriver;
use queryon_lib::infrastructure::postgres::pool::build_pool;
use queryon_lib::state::ConnectionRegistry;

fn dev_profile(read_only: bool) -> ConnectionProfile {
    ConnectionProfile {
        id: "test".to_string(),
        name: "test".to_string(),
        engine: Engine::Postgres,
        host: "localhost".to_string(),
        port: 55434,
        database: "devdb".to_string(),
        user: "devuser".to_string(),
        password: "devpass".to_string(),
        ssl_mode: SslMode::Disable,
        read_only,
        ssh_tunnel: None,
    }
}

#[tokio::test]
async fn require_writable_rejects_a_read_only_connection() {
    let pool = build_pool(&dev_profile(true)).expect("failed to build pool");
    let driver = PostgresDriver::new(pool);
    let registry = ConnectionRegistry::default();
    registry.insert("conn1".to_string(), std::sync::Arc::new(driver), true);

    let result = registry.require_writable("conn1");
    let err = result
        .err()
        .expect("a read-only connection should reject require_writable");
    assert!(
        err.to_string().contains("read-only"),
        "error message should mention read-only mode"
    );

    // Reads still work through `get`.
    assert!(registry.get("conn1").is_some());
}

#[tokio::test]
async fn require_writable_allows_a_normal_connection() {
    let pool = build_pool(&dev_profile(false)).expect("failed to build pool");
    let driver = PostgresDriver::new(pool);
    let registry = ConnectionRegistry::default();
    registry.insert("conn1".to_string(), std::sync::Arc::new(driver), false);

    assert!(registry.require_writable("conn1").is_ok());
}

#[test]
fn require_writable_rejects_an_unknown_connection() {
    let registry = ConnectionRegistry::default();
    let result = registry.require_writable("does-not-exist");
    let err = result
        .err()
        .expect("an unknown connection id should be rejected");
    assert!(err.to_string().contains("Not connected"));
}

#[test]
fn is_read_only_statement_accepts_reads() {
    assert!(query_service::is_read_only_statement("select * from users"));
    assert!(query_service::is_read_only_statement("  SELECT 1"));
    assert!(query_service::is_read_only_statement("explain select 1"));
    assert!(query_service::is_read_only_statement("show tables"));
    assert!(query_service::is_read_only_statement("describe users"));
}

#[test]
fn is_read_only_statement_rejects_writes() {
    assert!(!query_service::is_read_only_statement(
        "insert into users values (1)"
    ));
    assert!(!query_service::is_read_only_statement(
        "update users set x = 1"
    ));
    assert!(!query_service::is_read_only_statement("delete from users"));
    assert!(!query_service::is_read_only_statement("drop table users"));
    assert!(!query_service::is_read_only_statement("truncate users"));
    // A write-carrying CTE is not treated as a read, even though it
    // reads like one at a glance.
    assert!(!query_service::is_read_only_statement(
        "with x as (insert into users default values returning id) select * from x"
    ));
}
