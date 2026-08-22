use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::query::service as query_service;
use queryon_lib::domain::query::QueryResult;
use queryon_lib::infrastructure::postgres::driver::PostgresDriver;
use queryon_lib::infrastructure::postgres::pool::build_pool;
use queryon_lib::state::QueryResultCache;

fn dev_profile() -> ConnectionProfile {
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

async fn dev_driver() -> PostgresDriver {
    PostgresDriver::new(build_pool(&dev_profile()).expect("failed to build pool"))
}

#[tokio::test]
async fn execute_query_for_tab_returns_only_the_first_page_and_the_true_total() {
    let driver = dev_driver().await;
    let cache = QueryResultCache::default();
    let tab_id = "paginate-tab";
    let sql = "select generate_series as n from generate_series(1, 2500)";

    let result = query_service::execute_query_for_tab(&driver, tab_id, sql)
        .await
        .expect("query failed");
    cache.store(tab_id.to_string(), sql.to_string());

    match result {
        QueryResult::Rows { rows, total_row_count, .. } => {
            assert_eq!(rows.len(), 1000, "first page should be capped at PAGE_SIZE");
            assert_eq!(total_row_count, Some(2500));
        }
        QueryResult::Affected { .. } => panic!("expected a Rows result"),
    }

    let page = query_service::page_query_result(&driver, &cache, tab_id, 1000, 1000)
        .await
        .expect("second page should be servable by re-running the query");
    assert_eq!(page.rows.len(), 1000);
    assert_eq!(page.total_row_count, 2500);
    // First row of this page should be n = 1001, proving this is a fresh
    // LIMIT/OFFSET query against the database, not a slice of page one.
    let first_cell: i64 = serde_json::from_str(&page.rows[0][0]).unwrap();
    assert_eq!(first_cell, 1001);

    let last_page = query_service::page_query_result(&driver, &cache, tab_id, 2000, 1000)
        .await
        .expect("final partial page should be servable");
    assert_eq!(last_page.rows.len(), 500);

    let out_of_range = query_service::page_query_result(&driver, &cache, tab_id, 3000, 1000)
        .await
        .expect("offset past the end should still succeed with an empty page");
    assert!(out_of_range.rows.is_empty());
    assert_eq!(out_of_range.total_row_count, 2500);
}

#[tokio::test]
async fn page_query_result_errors_when_nothing_is_cached() {
    let driver = dev_driver().await;
    let cache = QueryResultCache::default();
    let result = query_service::page_query_result(&driver, &cache, "no-such-tab", 0, 100).await;
    assert!(result.is_err());
}
