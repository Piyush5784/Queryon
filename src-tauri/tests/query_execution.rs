use queryon_lib::domain::connection::{ConnectionProfile, SslMode};
use queryon_lib::domain::query::{service as query_service, QueryResult};
use queryon_lib::infrastructure::postgres::driver::PostgresDriver;
use queryon_lib::infrastructure::postgres::pool::build_pool;

fn dev_profile() -> ConnectionProfile {
    ConnectionProfile {
        id: "test".to_string(),
        name: "test".to_string(),
        host: "localhost".to_string(),
        port: 55433,
        database: "devdb".to_string(),
        user: "devuser".to_string(),
        password: "devpass".to_string(),
        ssl_mode: SslMode::Disable,
    }
}

async fn dev_driver() -> PostgresDriver {
    PostgresDriver::new(build_pool(&dev_profile()).expect("failed to build pool"))
}

/// Row cells cross the service boundary as JSON-encoded strings (see
/// TableRowsResult's doc comment); this parses one back for assertions.
fn cell_json(cell: &str) -> serde_json::Value {
    serde_json::from_str(cell).expect("cell should be valid JSON")
}

#[tokio::test]
async fn select_returns_rows_with_columns_and_duration() {
    let driver = dev_driver().await;
    let result = query_service::execute_query(&driver, "select id, email from users order by id limit 2")
        .await
        .expect("select failed");

    match result {
        QueryResult::Rows { columns, rows, row_count, truncated, .. } => {
            assert_eq!(columns, vec!["id".to_string(), "email".to_string()]);
            assert_eq!(rows.len(), 2);
            assert_eq!(row_count, 2);
            assert!(!truncated);
        }
        QueryResult::Affected { .. } => panic!("expected a Rows result for a SELECT"),
    }
}

#[tokio::test]
async fn select_with_no_matching_rows_returns_empty_rows_not_an_error() {
    let driver = dev_driver().await;
    let result = query_service::execute_query(&driver, "select * from users where id = -1")
        .await
        .expect("select failed");

    match result {
        QueryResult::Rows { row_count, .. } => assert_eq!(row_count, 0),
        QueryResult::Affected { .. } => panic!("expected a Rows result for a SELECT"),
    }
}

#[tokio::test]
async fn insert_returns_affected_row_count() {
    let driver = dev_driver().await;
    let result = query_service::execute_query(
        &driver,
        "insert into categories (name) values ('__test_query_insert__')",
    )
    .await
    .expect("insert failed");

    match result {
        QueryResult::Affected { row_count, .. } => assert_eq!(row_count, 1),
        QueryResult::Rows { .. } => panic!("expected an Affected result for an INSERT"),
    }

    // cleanup
    query_service::execute_query(&driver, "delete from categories where name = '__test_query_insert__'")
        .await
        .expect("cleanup delete failed");
}

#[tokio::test]
async fn update_returns_affected_row_count() {
    let driver = dev_driver().await;

    query_service::execute_query(&driver, "insert into categories (name) values ('__test_query_update__')")
        .await
        .expect("setup insert failed");

    let result = query_service::execute_query(
        &driver,
        "update categories set name = '__test_query_update_2__' where name = '__test_query_update__'",
    )
    .await
    .expect("update failed");

    match result {
        QueryResult::Affected { row_count, .. } => assert_eq!(row_count, 1),
        QueryResult::Rows { .. } => panic!("expected an Affected result for an UPDATE"),
    }

    query_service::execute_query(&driver, "delete from categories where name = '__test_query_update_2__'")
        .await
        .expect("cleanup delete failed");
}

#[tokio::test]
async fn delete_returns_affected_row_count() {
    let driver = dev_driver().await;

    query_service::execute_query(&driver, "insert into categories (name) values ('__test_query_delete__')")
        .await
        .expect("setup insert failed");

    let result = query_service::execute_query(
        &driver,
        "delete from categories where name = '__test_query_delete__'",
    )
    .await
    .expect("delete failed");

    match result {
        QueryResult::Affected { row_count, .. } => assert_eq!(row_count, 1),
        QueryResult::Rows { .. } => panic!("expected an Affected result for a DELETE"),
    }
}

#[tokio::test]
async fn syntax_error_produces_a_readable_message() {
    let driver = dev_driver().await;
    let result = query_service::execute_query(&driver, "select * frooom users").await;

    let err = result.expect_err("malformed SQL should fail").to_string();
    assert!(!err.is_empty());
    assert!(!err.contains("db error"), "error message should not leak the raw driver string, got: {err}");
}

#[tokio::test]
async fn referencing_unknown_table_produces_a_readable_message() {
    let driver = dev_driver().await;
    let result = query_service::execute_query(&driver, "select * from table_that_does_not_exist").await;

    let err = result.expect_err("querying a nonexistent table should fail").to_string();
    assert!(!err.is_empty());
    assert!(!err.contains("db error"), "error message should not leak the raw driver string, got: {err}");
}

#[tokio::test]
async fn empty_query_is_rejected_before_hitting_the_database() {
    let driver = dev_driver().await;
    let result = query_service::execute_query(&driver, "   ").await;
    assert!(result.is_err(), "an empty/whitespace-only query should be rejected");
}

#[tokio::test]
async fn json_column_round_trips_through_query_results() {
    let driver = dev_driver().await;
    let result = query_service::execute_query(&driver, "select metadata from users where id = 1")
        .await
        .expect("select failed");

    match result {
        QueryResult::Rows { rows, .. } => {
            assert_eq!(rows.len(), 1);
            assert!(cell_json(&rows[0][0]).is_object(), "metadata column should decode as a JSON object");
        }
        QueryResult::Affected { .. } => panic!("expected a Rows result"),
    }
}
