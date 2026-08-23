use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::driver::DatabaseDriver;
use queryon_lib::domain::query::service as query_service;
use queryon_lib::infrastructure::mysql::driver::MySqlDriver;
use queryon_lib::infrastructure::mysql::pool::build_pool;

fn dev_profile() -> ConnectionProfile {
    ConnectionProfile {
        id: "test".to_string(),
        name: "test".to_string(),
        engine: Engine::MariaDb,
        host: "localhost".to_string(),
        port: 33077,
        database: "devdb".to_string(),
        user: "devuser".to_string(),
        password: "devpass".to_string(),
        ssl_mode: SslMode::Disable,
        read_only: false,
        ssh_tunnel: None,
    }
}

async fn dev_driver() -> MySqlDriver {
    let pool = build_pool(&dev_profile()).await.expect("failed to build pool");
    MySqlDriver::new_mariadb(pool, "devdb".to_string())
}

async fn raw_exec(driver: &MySqlDriver, sql: &str) {
    query_service::execute_query(driver, sql)
        .await
        .expect("scratch table setup/teardown SQL failed");
}

/// MariaDb reuses `MySqlDriver` wholesale (see `Engine`'s doc comment) —
/// this proves a real connection and query round-trip works against a
/// live MariaDB server, matching Beekeeper Studio's own `MariaDBClient
/// extends MysqlClient`.
#[tokio::test]
async fn server_version_reports_a_mariadb_build() {
    let driver = dev_driver().await;
    let version = driver.server_version().await.expect("server_version failed");
    assert!(
        version.to_lowercase().contains("mariadb"),
        "expected a MariaDB version string, got {version:?}"
    );
}

#[tokio::test]
async fn list_tables_sees_the_seeded_schema() {
    let driver = dev_driver().await;
    let tables = driver.list_tables().await.expect("list_tables failed");
    let names: Vec<&str> = tables.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"users"));
    assert!(names.contains(&"products"));
}

/// The one real divergence found by testing directly against a live
/// MariaDB container: `information_schema.columns.column_default`
/// returns a quoted-string default still wrapped in literal quotes with
/// doubled internal quotes (`'it''s a test'`), where real MySQL returns
/// the bare unescaped string for the identical DEFAULT clause. This
/// proves `MySqlDriver::new_mariadb`'s unquoting actually fires and
/// produces the correct bare string.
#[tokio::test]
async fn get_table_columns_unquotes_a_mariadb_string_default() {
    let driver = dev_driver().await;
    raw_exec(&driver, "drop table if exists mariadb_default_test;").await;
    raw_exec(
        &driver,
        "create table mariadb_default_test (id int primary key, note varchar(50) default 'it''s a test')",
    )
    .await;

    let columns = driver
        .get_table_columns("devdb", "mariadb_default_test")
        .await
        .expect("get_table_columns failed");
    let note = columns.iter().find(|c| c.name == "note").expect("expected a note column");
    assert_eq!(note.default.as_deref(), Some("it's a test"));

    raw_exec(&driver, "drop table mariadb_default_test;").await;
}

#[tokio::test]
async fn get_table_columns_leaves_a_plain_default_untouched() {
    let driver = dev_driver().await;
    raw_exec(&driver, "drop table if exists mariadb_plain_default_test;").await;
    raw_exec(
        &driver,
        "create table mariadb_plain_default_test (id int primary key, qty int default 5)",
    )
    .await;

    let columns = driver
        .get_table_columns("devdb", "mariadb_plain_default_test")
        .await
        .expect("get_table_columns failed");
    let qty = columns.iter().find(|c| c.name == "qty").expect("expected a qty column");
    assert_eq!(qty.default.as_deref(), Some("5"));

    raw_exec(&driver, "drop table mariadb_plain_default_test;").await;
}

#[tokio::test]
async fn fetch_table_rows_reads_seeded_data() {
    let driver = dev_driver().await;
    let result = driver
        .fetch_table_rows("devdb", "products", 10, 0, &[], &[])
        .await
        .expect("fetch_table_rows failed");
    assert!(result.row_count > 0);
}

#[tokio::test]
async fn execute_query_runs_arbitrary_sql() {
    let driver = dev_driver().await;
    let result = query_service::execute_query(&driver, "select count(*) from categories")
        .await
        .expect("execute_query failed");
    match result {
        queryon_lib::domain::query::QueryResult::Rows { rows, .. } => {
            assert_eq!(rows.len(), 1);
        }
        queryon_lib::domain::query::QueryResult::Affected { .. } => panic!("expected a Rows result"),
    }
}
