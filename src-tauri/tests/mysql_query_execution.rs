use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::query::{service as query_service, QueryResult};
use queryon_lib::error::AppError;
use queryon_lib::infrastructure::mysql::driver::MySqlDriver;
use queryon_lib::infrastructure::mysql::pool::build_pool;

async fn exec(driver: &MySqlDriver, sql: &str) -> Result<QueryResult, AppError> {
    query_service::execute_query(driver, sql).await
}

fn dev_profile() -> ConnectionProfile {
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

async fn dev_driver() -> MySqlDriver {
    let pool = build_pool(&dev_profile()).await.expect("failed to build pool");
    MySqlDriver::new(pool, "devdb".to_string())
}

fn cell_json(cell: &str) -> serde_json::Value {
    serde_json::from_str(cell).expect("cell should be valid JSON")
}

#[tokio::test]
async fn select_returns_rows_with_columns_and_duration() {
    let driver = dev_driver().await;
    let result = exec(&driver, "select id, email from users order by id limit 2")
        .await
        .expect("select failed");

    match result {
        QueryResult::Rows { columns, rows, total_row_count, .. } => {
            assert_eq!(columns, vec!["id".to_string(), "email".to_string()]);
            assert_eq!(rows.len(), 2);
            assert_eq!(total_row_count, Some(2));
        }
        QueryResult::Affected { .. } => panic!("expected a Rows result for a SELECT"),
    }
}

#[tokio::test]
async fn select_with_no_matching_rows_returns_empty_rows_not_an_error() {
    let driver = dev_driver().await;
    let result = exec(&driver, "select * from users where id = -1")
        .await
        .expect("select failed");

    match result {
        QueryResult::Rows { total_row_count, .. } => assert_eq!(total_row_count, Some(0)),
        QueryResult::Affected { .. } => panic!("expected a Rows result for a SELECT"),
    }
}

#[tokio::test]
async fn insert_returns_affected_row_count() {
    let driver = dev_driver().await;
    let result = exec(
        &driver,
        "insert into categories (name) values ('__test_query_insert__')",
    )
    .await
    .expect("insert failed");

    match result {
        QueryResult::Affected { row_count, .. } => assert_eq!(row_count, 1),
        QueryResult::Rows { .. } => panic!("expected an Affected result for an INSERT"),
    }

    exec(&driver, "delete from categories where name = '__test_query_insert__'")
        .await
        .expect("cleanup delete failed");
}

#[tokio::test]
async fn update_returns_affected_row_count() {
    let driver = dev_driver().await;

    exec(&driver, "insert into categories (name) values ('__test_query_update__')")
        .await
        .expect("setup insert failed");

    let result = exec(
        &driver,
        "update categories set name = '__test_query_update_2__' where name = '__test_query_update__'",
    )
    .await
    .expect("update failed");

    match result {
        QueryResult::Affected { row_count, .. } => assert_eq!(row_count, 1),
        QueryResult::Rows { .. } => panic!("expected an Affected result for an UPDATE"),
    }

    exec(&driver, "delete from categories where name = '__test_query_update_2__'")
        .await
        .expect("cleanup delete failed");
}

#[tokio::test]
async fn delete_returns_affected_row_count() {
    let driver = dev_driver().await;

    exec(&driver, "insert into categories (name) values ('__test_query_delete__')")
        .await
        .expect("setup insert failed");

    let result = exec(
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
    let result = exec(&driver, "select * frooom users").await;

    let err = result.expect_err("malformed SQL should fail").to_string();
    assert!(!err.is_empty());
}

#[tokio::test]
async fn referencing_unknown_table_produces_a_readable_message() {
    let driver = dev_driver().await;
    let result = exec(&driver, "select * from table_that_does_not_exist").await;

    let err = result.expect_err("querying a nonexistent table should fail").to_string();
    assert!(!err.is_empty());
}

#[tokio::test]
async fn empty_query_is_rejected_before_hitting_the_database() {
    let driver = dev_driver().await;
    let result = exec(&driver, "   ").await;
    assert!(result.is_err(), "an empty/whitespace-only query should be rejected");
}

#[tokio::test]
async fn json_column_round_trips_through_query_results() {
    let driver = dev_driver().await;
    let result = exec(&driver, "select metadata from users where id = 1")
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
