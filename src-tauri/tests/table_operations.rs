use std::collections::HashMap;

use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::schema::service as schema_service;
use queryon_lib::domain::table::service as table_service;
use queryon_lib::infrastructure::postgres::driver::PostgresDriver;
use queryon_lib::infrastructure::postgres::pool::build_pool;
use serde_json::json;

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
    }
}

async fn dev_pool() -> deadpool_postgres::Pool {
    build_pool(&dev_profile()).expect("failed to build pool")
}

/// Row cells cross the service boundary as JSON-encoded strings (see
/// TableRowsResult's doc comment); this parses one back for assertions.
fn cell_json(cell: &str) -> serde_json::Value {
    serde_json::from_str(cell).expect("cell should be valid JSON")
}

#[tokio::test]
async fn list_tables_includes_known_seed_tables() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());
    let tables = schema_service::list_tables(&driver)
        .await
        .expect("list_tables failed");

    let names: Vec<&str> = tables.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"users"), "expected 'users' table, got {names:?}");
    assert!(names.contains(&"orders"), "expected 'orders' table, got {names:?}");
    assert!(names.contains(&"events"), "expected 'events' table, got {names:?}");

    for table in &tables {
        assert_eq!(table.schema, "public");
        assert!(matches!(table.kind.as_str(), "table" | "view" | "materialized_view"));
    }
}

#[tokio::test]
async fn get_table_columns_identifies_primary_key() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());
    let columns = schema_service::get_table_columns(&driver, "public", "users")
        .await
        .expect("get_table_columns failed");

    let id_col = columns
        .iter()
        .find(|c| c.name == "id")
        .expect("expected an 'id' column on users");
    assert!(id_col.is_primary_key, "'id' should be the primary key");

    let email_col = columns
        .iter()
        .find(|c| c.name == "email")
        .expect("expected an 'email' column on users");
    assert!(!email_col.is_primary_key);
    assert!(!email_col.is_nullable);
}

#[tokio::test]
async fn get_table_columns_unknown_table_returns_empty() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());
    let columns = schema_service::get_table_columns(&driver, "public", "table_that_does_not_exist")
        .await
        .expect("query itself should not error for an unknown table");
    assert!(columns.is_empty());
}

#[tokio::test]
async fn fetch_table_rows_respects_limit() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());
    let result = table_service::fetch_table_rows(&driver, "public", "users", 2, 0, &[], &[])
        .await
        .expect("fetch_table_rows failed");

    assert_eq!(result.row_count, 2);
    assert!(result.columns.contains(&"email".to_string()));
    assert!(result.columns.contains(&"id".to_string()));
}

#[tokio::test]
async fn fetch_table_rows_paginates_without_overlap() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());

    let page1 = table_service::fetch_table_rows(&driver, "public", "users", 2, 0, &[], &[])
        .await
        .expect("page 1 fetch failed");
    let page2 = table_service::fetch_table_rows(&driver, "public", "users", 2, 2, &[], &[])
        .await
        .expect("page 2 fetch failed");

    let id_index = page1
        .columns
        .iter()
        .position(|c| c == "id")
        .expect("expected an id column");

    let page1_ids: Vec<_> = page1.rows.iter().map(|r| r[id_index].clone()).collect();
    let page2_ids: Vec<_> = page2.rows.iter().map(|r| r[id_index].clone()).collect();

    for id in &page1_ids {
        assert!(!page2_ids.contains(id), "page 2 should not repeat page 1's rows");
    }
}

#[tokio::test]
async fn fetch_table_rows_clamps_oversized_limit() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());
    // MAX_PAGE_SIZE is 10,000; requesting far more should not error or
    // hang, and should not return more rows than actually exist.
    let result = table_service::fetch_table_rows(&driver, "public", "users", 100_000, 0, &[], &[])
        .await
        .expect("fetch_table_rows failed");
    assert!(result.row_count < 1000);
}

#[tokio::test]
async fn fetch_table_rows_rejects_invalid_identifier() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());
    let result = table_service::fetch_table_rows(&driver, "public", "users; drop table users;--", 10, 0, &[], &[]).await;
    assert!(result.is_err(), "malicious identifier should be rejected, not executed");
}

#[tokio::test]
async fn update_json_cell_writes_and_is_readable_back() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(3));

    table_service::update_json_cell(
        &driver,
        "public",
        "users",
        &row,
        "metadata",
        &json!({ "plan": "enterprise", "seats": 42 }),
    )
    .await
    .expect("update_cell failed");

    let result = table_service::fetch_table_rows(&driver, "public", "users", 10, 0, &[], &[])
        .await
        .expect("fetch after update failed");

    let id_index = result.columns.iter().position(|c| c == "id").unwrap();
    let metadata_index = result.columns.iter().position(|c| c == "metadata").unwrap();

    let updated_row = result
        .rows
        .iter()
        .find(|r| cell_json(&r[id_index]) == json!(3))
        .expect("expected to find the row we just updated");

    assert_eq!(
        cell_json(&updated_row[metadata_index]),
        json!({ "plan": "enterprise", "seats": 42 })
    );

    // Restore original seed value so the test is repeatable.
    table_service::update_json_cell(&driver, "public", "users", &row, "metadata", &json!({ "plan": "free" }))
        .await
        .expect("failed to restore original value");
}

#[tokio::test]
async fn update_json_cell_fails_without_matching_primary_key_value() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(999_999_999));

    let result = table_service::update_json_cell(
        &driver,
        "public",
        "users",
        &row,
        "metadata",
        &json!({ "should": "not apply" }),
    )
    .await;

    assert!(result.is_err(), "updating a non-existent row should fail, not silently no-op");
}

#[tokio::test]
async fn update_json_cell_fails_when_row_is_missing_primary_key_value() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());

    let row: HashMap<String, serde_json::Value> = HashMap::new();

    let result =
        table_service::update_json_cell(&driver, "public", "users", &row, "metadata", &json!({})).await;

    assert!(result.is_err(), "missing primary key value in the row map should fail fast");
}

#[tokio::test]
async fn update_cell_text_writes_a_text_column() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(2));

    table_service::update_cell_text(&driver, "public", "users", &row, "full_name", Some("Bob Updated"))
        .await
        .expect("update_cell_text failed");

    let result = table_service::fetch_table_rows(&driver, "public", "users", 10, 0, &[], &[])
        .await
        .expect("fetch after update failed");

    let id_index = result.columns.iter().position(|c| c == "id").unwrap();
    let name_index = result.columns.iter().position(|c| c == "full_name").unwrap();

    let updated_row = result
        .rows
        .iter()
        .find(|r| cell_json(&r[id_index]) == json!(2))
        .expect("expected to find the row we just updated");
    assert_eq!(cell_json(&updated_row[name_index]), json!("Bob Updated"));

    table_service::update_cell_text(&driver, "public", "users", &row, "full_name", Some("Bob Martinez"))
        .await
        .expect("failed to restore original value");
}

#[tokio::test]
async fn update_cell_text_writes_a_boolean_column() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(2));

    table_service::update_cell_text(&driver, "public", "users", &row, "is_active", Some("false"))
        .await
        .expect("update_cell_text failed");

    let result = table_service::fetch_table_rows(&driver, "public", "users", 10, 0, &[], &[])
        .await
        .expect("fetch after update failed");

    let id_index = result.columns.iter().position(|c| c == "id").unwrap();
    let active_index = result.columns.iter().position(|c| c == "is_active").unwrap();

    let updated_row = result
        .rows
        .iter()
        .find(|r| cell_json(&r[id_index]) == json!(2))
        .expect("expected to find the row we just updated");
    assert_eq!(cell_json(&updated_row[active_index]), json!(false));

    table_service::update_cell_text(&driver, "public", "users", &row, "is_active", Some("true"))
        .await
        .expect("failed to restore original value");
}

#[tokio::test]
async fn update_cell_text_sets_null() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(2));

    table_service::update_cell_text(&driver, "public", "orders", &row, "notes", None)
        .await
        .expect("update_cell_text with null failed");

    let result = table_service::fetch_table_rows(&driver, "public", "orders", 10, 0, &[], &[])
        .await
        .expect("fetch after update failed");

    let id_index = result.columns.iter().position(|c| c == "id").unwrap();
    let notes_index = result.columns.iter().position(|c| c == "notes").unwrap();

    let updated_row = result
        .rows
        .iter()
        .find(|r| cell_json(&r[id_index]) == json!(2))
        .expect("expected to find the row we just updated");
    assert_eq!(cell_json(&updated_row[notes_index]), json!(null));
}

#[tokio::test]
async fn update_cell_text_rejects_invalid_value_for_column_type() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(1));

    let result =
        table_service::update_cell_text(&driver, "public", "products", &row, "price_cents", Some("not-a-number"))
            .await;

    assert!(result.is_err(), "a non-numeric string cast to an integer column should fail");
}

#[tokio::test]
async fn update_cell_text_rejects_unknown_column() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(1));

    let result =
        table_service::update_cell_text(&driver, "public", "users", &row, "not_a_real_column", Some("x")).await;

    assert!(result.is_err(), "updating a nonexistent column should fail, not silently no-op");
}

async fn insert_throwaway_category(pool: &deadpool_postgres::Pool, name: &str) -> i64 {
    let client = pool.get().await.unwrap();
    let row = client
        .query_one(
            "insert into categories (name) values ($1) returning id",
            &[&name],
        )
        .await
        .unwrap();
    row.get::<_, i64>(0)
}

#[tokio::test]
async fn delete_rows_removes_a_single_row() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());
    let id = insert_throwaway_category(&pool, "__test_delete_single__").await;

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(id));

    let affected = table_service::delete_rows(&driver, "public", "categories", &[row])
        .await
        .expect("delete_rows failed");
    assert_eq!(affected, 1);

    let result = table_service::fetch_table_rows(&driver, "public", "categories", 500, 0, &[], &[])
        .await
        .unwrap();
    let id_index = result.columns.iter().position(|c| c == "id").unwrap();
    assert!(
        !result.rows.iter().any(|r| cell_json(&r[id_index]) == json!(id)),
        "deleted row should no longer be present"
    );
}

#[tokio::test]
async fn delete_rows_removes_multiple_rows_at_once() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());
    let id1 = insert_throwaway_category(&pool, "__test_delete_multi_1__").await;
    let id2 = insert_throwaway_category(&pool, "__test_delete_multi_2__").await;

    let mut row1 = HashMap::new();
    row1.insert("id".to_string(), json!(id1));
    let mut row2 = HashMap::new();
    row2.insert("id".to_string(), json!(id2));

    let affected = table_service::delete_rows(&driver, "public", "categories", &[row1, row2])
        .await
        .expect("delete_rows failed");
    assert_eq!(affected, 2);

    let result = table_service::fetch_table_rows(&driver, "public", "categories", 500, 0, &[], &[])
        .await
        .unwrap();
    let id_index = result.columns.iter().position(|c| c == "id").unwrap();
    assert!(!result.rows.iter().any(|r| cell_json(&r[id_index]) == json!(id1)));
    assert!(!result.rows.iter().any(|r| cell_json(&r[id_index]) == json!(id2)));
}

#[tokio::test]
async fn delete_rows_with_empty_input_is_a_noop() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());
    let affected = table_service::delete_rows(&driver, "public", "categories", &[])
        .await
        .expect("delete_rows with empty input should succeed as a no-op");
    assert_eq!(affected, 0);
}

#[tokio::test]
async fn delete_rows_fails_for_nonexistent_row() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(999_999_999));

    let result = table_service::delete_rows(&driver, "public", "categories", &[row]).await;
    assert!(result.is_err(), "deleting a row that doesn't exist should fail, not silently no-op");
}

#[tokio::test]
async fn update_cell_text_gives_clean_message_for_foreign_key_violation() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(1));

    let result =
        table_service::update_cell_text(&driver, "public", "orders", &row, "user_id", Some("999999999")).await;

    let err = result.expect_err("assigning orders.user_id to a nonexistent user should fail").to_string();
    assert!(!err.contains("db error"), "error message should not leak the raw driver string, got: {err}");
    assert!(
        err.to_lowercase().contains("match") || err.to_lowercase().contains("referenced"),
        "expected a foreign-key-shaped message, got: {err}"
    );
}

#[tokio::test]
async fn update_cell_text_gives_clean_message_for_not_null_violation() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(1));

    let result = table_service::update_cell_text(&driver, "public", "orders", &row, "user_id", None).await;

    let err = result.expect_err("setting a not-null column to null should fail").to_string();
    assert!(!err.contains("db error"), "error message should not leak the raw driver string, got: {err}");
    assert!(
        err.to_lowercase().contains("empty") || err.to_lowercase().contains("null"),
        "expected a not-null-shaped message, got: {err}"
    );
}

async fn insert_throwaway_user(pool: &deadpool_postgres::Pool, email: &str) -> i64 {
    let client = pool.get().await.unwrap();
    let row = client
        .query_one(
            "insert into users (email, full_name) values ($1, 'Throwaway') returning id",
            &[&email],
        )
        .await
        .unwrap();
    row.get::<_, i64>(0)
}

#[tokio::test]
async fn update_cell_text_gives_clean_message_for_unique_violation() {
    let pool = dev_pool().await;
    let driver = PostgresDriver::new(pool.clone());
    let taken_email = format!("__taken_{}@example.com", uuid_like_suffix());
    insert_throwaway_user(&pool, &taken_email).await;
    let colliding_id = insert_throwaway_user(&pool, &format!("__free_{}@example.com", uuid_like_suffix())).await;

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(colliding_id));

    let result = table_service::update_cell_text(&driver, "public", "users", &row, "email", Some(&taken_email)).await;

    let client = pool.get().await.unwrap();
    client
        .execute("delete from users where full_name = 'Throwaway'", &[])
        .await
        .expect("cleanup delete failed");

    let err = result.expect_err("colliding with an existing unique email should fail").to_string();
    assert!(!err.contains("db error"), "error message should not leak the raw driver string, got: {err}");
    assert!(
        err.to_lowercase().contains("already exists") || err.to_lowercase().contains("unique"),
        "expected a unique-violation-shaped message, got: {err}"
    );
}

fn uuid_like_suffix() -> String {
    format!("{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())
}
