use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::driver::DatabaseDriver;
use queryon_lib::domain::query::service as query_service;
use queryon_lib::domain::query::QueryResult;
use queryon_lib::error::AppError;
use queryon_lib::infrastructure::mysql::driver::MySqlDriver;
use queryon_lib::infrastructure::mysql::pool::build_pool as build_mysql_pool;
use queryon_lib::infrastructure::postgres::driver::PostgresDriver;
use queryon_lib::infrastructure::postgres::pool::build_pool as build_pg_pool;

async fn exec(driver: &dyn DatabaseDriver, sql: &str) -> Result<QueryResult, AppError> {
    query_service::execute_query(driver, sql).await
}

async fn exec_for_tab(
    driver: &dyn DatabaseDriver,
    tab_id: &str,
    sql: &str,
) -> Result<QueryResult, AppError> {
    query_service::execute_query_for_tab(driver, tab_id, sql).await
}

fn pg_profile() -> ConnectionProfile {
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
        read_only: false,
        ssh_tunnel: None,
    }
}

async fn pg_driver() -> PostgresDriver {
    PostgresDriver::new(build_pg_pool(&pg_profile()).expect("failed to build pool"))
}

fn mysql_profile() -> ConnectionProfile {
    ConnectionProfile {
        id: "test".to_string(),
        name: "test".to_string(),
        engine: Engine::MySql,
        host: "localhost".to_string(),
        port: 33066,
        database: "devdb".to_string(),
        user: "devuser".to_string(),
        password: "devpass".to_string(),
        ssl_mode: SslMode::Disable,
        read_only: false,
        ssh_tunnel: None,
    }
}

async fn mysql_driver() -> MySqlDriver {
    let pool = build_mysql_pool(&mysql_profile())
        .await
        .expect("failed to build pool");
    MySqlDriver::new(pool, "devdb".to_string())
}

async fn raw_exec(driver: &dyn DatabaseDriver, sql: &str) {
    exec(driver, sql)
        .await
        .expect("scratch table setup/teardown SQL failed");
}

#[tokio::test]
async fn postgres_commit_persists_changes_made_in_the_transaction() {
    let driver = pg_driver().await;
    raw_exec(&driver, "drop table if exists txn_pg_commit_test;").await;
    raw_exec(&driver, "create table txn_pg_commit_test (id serial primary key, name text);").await;

    let tab_id = "tab-pg-commit";
    assert!(!driver.has_active_transaction(tab_id));

    query_service::begin_transaction(&driver, tab_id)
        .await
        .expect("begin_transaction failed");
    assert!(driver.has_active_transaction(tab_id));

    exec_for_tab(&driver, tab_id, "insert into txn_pg_commit_test (name) values ('a')")
        .await
        .expect("insert inside transaction failed");

    // Not yet visible on a different (pooled) connection, since it
    // hasn't been committed.
    let result = exec(&driver, "select count(*) from txn_pg_commit_test")
        .await
        .expect("count query failed");
    assert_row_count(&result, 0);

    query_service::commit_transaction(&driver, tab_id)
        .await
        .expect("commit_transaction failed");
    assert!(!driver.has_active_transaction(tab_id));

    let result = exec(&driver, "select count(*) from txn_pg_commit_test")
        .await
        .expect("count query failed");
    assert_row_count(&result, 1);

    raw_exec(&driver, "drop table txn_pg_commit_test;").await;
}

#[tokio::test]
async fn postgres_rollback_discards_changes_made_in_the_transaction() {
    let driver = pg_driver().await;
    raw_exec(&driver, "drop table if exists txn_pg_rollback_test;").await;
    raw_exec(&driver, "create table txn_pg_rollback_test (id serial primary key, name text);").await;

    let tab_id = "tab-pg-rollback";
    query_service::begin_transaction(&driver, tab_id)
        .await
        .expect("begin_transaction failed");

    exec_for_tab(&driver, tab_id, "insert into txn_pg_rollback_test (name) values ('a')")
        .await
        .expect("insert inside transaction failed");

    query_service::rollback_transaction(&driver, tab_id)
        .await
        .expect("rollback_transaction failed");
    assert!(!driver.has_active_transaction(tab_id));

    let result = exec(&driver, "select count(*) from txn_pg_rollback_test")
        .await
        .expect("count query failed");
    assert_row_count(&result, 0);

    raw_exec(&driver, "drop table txn_pg_rollback_test;").await;
}

#[tokio::test]
async fn postgres_begin_twice_on_same_tab_errors() {
    let driver = pg_driver().await;
    let tab_id = "tab-pg-double-begin";

    query_service::begin_transaction(&driver, tab_id)
        .await
        .expect("first begin_transaction failed");

    let second = query_service::begin_transaction(&driver, tab_id).await;
    assert!(second.is_err(), "beginning a second transaction on the same tab should fail");

    query_service::rollback_transaction(&driver, tab_id)
        .await
        .expect("cleanup rollback failed");
}

#[tokio::test]
async fn postgres_commit_without_active_transaction_errors() {
    let driver = pg_driver().await;
    let result = query_service::commit_transaction(&driver, "tab-pg-no-txn").await;
    assert!(result.is_err(), "committing with no active transaction should fail");
}

#[tokio::test]
async fn mysql_commit_persists_changes_made_in_the_transaction() {
    let driver = mysql_driver().await;
    raw_exec(&driver, "drop table if exists txn_mysql_commit_test;").await;
    raw_exec(
        &driver,
        "create table txn_mysql_commit_test (id int primary key auto_increment, name varchar(255));",
    )
    .await;

    let tab_id = "tab-mysql-commit";
    assert!(!driver.has_active_transaction(tab_id));

    query_service::begin_transaction(&driver, tab_id)
        .await
        .expect("begin_transaction failed");
    assert!(driver.has_active_transaction(tab_id));

    exec_for_tab(
        &driver,
        tab_id,
        "insert into txn_mysql_commit_test (name) values ('a')",
    )
    .await
    .expect("insert inside transaction failed");

    let result = exec(&driver, "select count(*) from txn_mysql_commit_test")
        .await
        .expect("count query failed");
    assert_row_count(&result, 0);

    query_service::commit_transaction(&driver, tab_id)
        .await
        .expect("commit_transaction failed");
    assert!(!driver.has_active_transaction(tab_id));

    let result = exec(&driver, "select count(*) from txn_mysql_commit_test")
        .await
        .expect("count query failed");
    assert_row_count(&result, 1);

    raw_exec(&driver, "drop table txn_mysql_commit_test;").await;
}

#[tokio::test]
async fn mysql_rollback_discards_changes_made_in_the_transaction() {
    let driver = mysql_driver().await;
    raw_exec(&driver, "drop table if exists txn_mysql_rollback_test;").await;
    raw_exec(
        &driver,
        "create table txn_mysql_rollback_test (id int primary key auto_increment, name varchar(255));",
    )
    .await;

    let tab_id = "tab-mysql-rollback";
    query_service::begin_transaction(&driver, tab_id)
        .await
        .expect("begin_transaction failed");

    exec_for_tab(
        &driver,
        tab_id,
        "insert into txn_mysql_rollback_test (name) values ('a')",
    )
    .await
    .expect("insert inside transaction failed");

    query_service::rollback_transaction(&driver, tab_id)
        .await
        .expect("rollback_transaction failed");
    assert!(!driver.has_active_transaction(tab_id));

    let result = exec(&driver, "select count(*) from txn_mysql_rollback_test")
        .await
        .expect("count query failed");
    assert_row_count(&result, 0);

    raw_exec(&driver, "drop table txn_mysql_rollback_test;").await;
}

fn assert_row_count(result: &queryon_lib::domain::query::QueryResult, expected: u32) {
    use queryon_lib::domain::query::QueryResult;
    match result {
        QueryResult::Rows { rows, .. } => {
            let count: i64 = serde_json::from_str(&rows[0][0]).expect("count cell should be valid JSON");
            assert_eq!(count as u32, expected);
        }
        QueryResult::Affected { .. } => panic!("expected a rows result from a count query"),
    }
}
