use std::sync::Arc;
use std::time::{Duration, Instant};

use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::driver::DatabaseDriver;
use queryon_lib::infrastructure::mysql::driver::MySqlDriver;
use queryon_lib::infrastructure::mysql::pool::build_pool as build_mysql_pool;
use queryon_lib::infrastructure::postgres::driver::PostgresDriver;
use queryon_lib::infrastructure::postgres::pool::build_pool as build_pg_pool;

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

#[tokio::test]
async fn postgres_cancel_query_interrupts_a_running_query() {
    let driver = Arc::new(pg_driver().await);
    let tab_id = "cancel-pg-tab";

    let run = {
        let driver = driver.clone();
        tokio::spawn(async move {
            let start = Instant::now();
            let result = driver
                .execute_query_for_tab(tab_id, "SELECT pg_sleep(10)", 0, 100)
                .await;
            (result, start.elapsed())
        })
    };

    tokio::time::sleep(Duration::from_millis(500)).await;
    driver
        .cancel_query(tab_id)
        .await
        .expect("cancel_query should succeed while the sleep is running");

    let (result, elapsed) = run.await.expect("task panicked");
    assert!(result.is_err(), "cancelled query should return an error");
    assert!(
        elapsed < Duration::from_secs(5),
        "query should have been interrupted well before the 10s sleep finished, took {elapsed:?}"
    );
}

#[tokio::test]
async fn postgres_cancel_query_without_a_running_query_errors() {
    let driver = pg_driver().await;
    let result = driver.cancel_query("no-such-tab").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn mysql_cancel_query_interrupts_a_running_query() {
    let driver = Arc::new(mysql_driver().await);
    let tab_id = "cancel-mysql-tab";

    // `SELECT SLEEP(n)` isn't suitable here: MySQL's `KILL QUERY` makes it
    // return early with a *successful* result of 1 (interrupted), not an
    // error — a real long-running query (e.g. a heavy cross join) is what
    // actually surfaces "Query execution was interrupted" as an error.
    let heavy_query = "SELECT COUNT(*) FROM information_schema.columns a, \
        information_schema.columns b, information_schema.columns c";

    let run = {
        let driver = driver.clone();
        tokio::spawn(async move {
            let start = Instant::now();
            let result = driver.execute_query_for_tab(tab_id, heavy_query, 0, 100).await;
            (result, start.elapsed())
        })
    };

    tokio::time::sleep(Duration::from_millis(500)).await;
    driver
        .cancel_query(tab_id)
        .await
        .expect("cancel_query should succeed while the query is running");

    let (result, elapsed) = run.await.expect("task panicked");
    assert!(result.is_err(), "cancelled query should return an error");
    assert!(
        elapsed < Duration::from_secs(5),
        "query should have been interrupted quickly, took {elapsed:?}"
    );
}

#[tokio::test]
async fn mysql_cancel_query_without_a_running_query_errors() {
    let driver = mysql_driver().await;
    let result = driver.cancel_query("no-such-tab").await;
    assert!(result.is_err());
}
