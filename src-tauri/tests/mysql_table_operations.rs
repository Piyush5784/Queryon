use std::collections::HashMap;

use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::schema::service as schema_service;
use queryon_lib::domain::table::service as table_service;
use queryon_lib::infrastructure::mysql::driver::MySqlDriver;
use queryon_lib::infrastructure::mysql::pool::build_pool;
use serde_json::json;

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
    }
}

async fn dev_driver() -> MySqlDriver {
    let pool = build_pool(&dev_profile()).await.expect("failed to build pool");
    MySqlDriver::new(pool, "devdb".to_string())
}

/// Row cells cross the service boundary as JSON-encoded strings (see
/// TableRowsResult's doc comment); this parses one back for assertions.
fn cell_json(cell: &str) -> serde_json::Value {
    serde_json::from_str(cell).expect("cell should be valid JSON")
}

#[tokio::test]
async fn list_tables_includes_known_seed_tables() {
    let driver = dev_driver().await;
    let tables = schema_service::list_tables(&driver)
        .await
        .expect("list_tables failed");

    let names: Vec<&str> = tables.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"users"), "expected 'users' table, got {names:?}");
    assert!(names.contains(&"orders"), "expected 'orders' table, got {names:?}");

    for table in &tables {
        assert_eq!(table.schema, "devdb");
        assert_eq!(table.kind, "table");
    }
}

#[tokio::test]
async fn get_table_columns_identifies_primary_key() {
    let driver = dev_driver().await;
    let columns = schema_service::get_table_columns(&driver, "devdb", "users")
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
    let driver = dev_driver().await;
    let columns = schema_service::get_table_columns(&driver, "devdb", "table_that_does_not_exist")
        .await
        .expect("query itself should not error for an unknown table");
    assert!(columns.is_empty());
}

#[tokio::test]
async fn fetch_table_rows_respects_limit() {
    let driver = dev_driver().await;
    let result = table_service::fetch_table_rows(&driver, "devdb", "users", 2, 0)
        .await
        .expect("fetch_table_rows failed");

    assert_eq!(result.row_count, 2);
    assert!(result.columns.contains(&"email".to_string()));
    assert!(result.columns.contains(&"id".to_string()));
}

#[tokio::test]
async fn fetch_table_rows_paginates_without_overlap() {
    let driver = dev_driver().await;

    let page1 = table_service::fetch_table_rows(&driver, "devdb", "users", 2, 0)
        .await
        .expect("page 1 fetch failed");
    let page2 = table_service::fetch_table_rows(&driver, "devdb", "users", 2, 2)
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
async fn fetch_table_rows_rejects_invalid_identifier() {
    let driver = dev_driver().await;
    let result = table_service::fetch_table_rows(&driver, "devdb", "users; drop table users;--", 10, 0).await;
    assert!(result.is_err(), "malicious identifier should be rejected, not executed");
}

#[tokio::test]
async fn update_json_cell_writes_and_is_readable_back() {
    let driver = dev_driver().await;

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(3));

    table_service::update_json_cell(
        &driver,
        "devdb",
        "users",
        &row,
        "metadata",
        &json!({ "plan": "enterprise", "seats": 42 }),
    )
    .await
    .expect("update_cell failed");

    let result = table_service::fetch_table_rows(&driver, "devdb", "users", 10, 0)
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
    table_service::update_json_cell(&driver, "devdb", "users", &row, "metadata", &json!({ "plan": "free" }))
        .await
        .expect("failed to restore original value");
}

#[tokio::test]
async fn update_json_cell_fails_without_matching_primary_key_value() {
    let driver = dev_driver().await;

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(999_999_999));

    let result = table_service::update_json_cell(
        &driver,
        "devdb",
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
    let driver = dev_driver().await;

    let row: HashMap<String, serde_json::Value> = HashMap::new();

    let result =
        table_service::update_json_cell(&driver, "devdb", "users", &row, "metadata", &json!({})).await;

    assert!(result.is_err(), "missing primary key value in the row map should fail fast");
}

#[tokio::test]
async fn update_cell_text_writes_a_text_column() {
    let driver = dev_driver().await;

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(2));

    table_service::update_cell_text(&driver, "devdb", "users", &row, "full_name", Some("Bob Updated"))
        .await
        .expect("update_cell_text failed");

    let result = table_service::fetch_table_rows(&driver, "devdb", "users", 10, 0)
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

    table_service::update_cell_text(&driver, "devdb", "users", &row, "full_name", Some("Bob Martinez"))
        .await
        .expect("failed to restore original value");
}

#[tokio::test]
async fn update_cell_text_writes_a_boolean_column() {
    let driver = dev_driver().await;

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(2));

    table_service::update_cell_text(&driver, "devdb", "users", &row, "is_active", Some("0"))
        .await
        .expect("update_cell_text failed");

    let result = table_service::fetch_table_rows(&driver, "devdb", "users", 10, 0)
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

    table_service::update_cell_text(&driver, "devdb", "users", &row, "is_active", Some("1"))
        .await
        .expect("failed to restore original value");
}

#[tokio::test]
async fn update_cell_text_sets_null() {
    let driver = dev_driver().await;

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(2));

    table_service::update_cell_text(&driver, "devdb", "orders", &row, "notes", None)
        .await
        .expect("update_cell_text with null failed");

    let result = table_service::fetch_table_rows(&driver, "devdb", "orders", 10, 0)
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
async fn update_cell_text_rejects_unknown_column() {
    let driver = dev_driver().await;

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(1));

    let result =
        table_service::update_cell_text(&driver, "devdb", "users", &row, "not_a_real_column", Some("x")).await;

    assert!(result.is_err(), "updating a nonexistent column should fail, not silently no-op");
}

async fn insert_throwaway_category(pool: &sqlx::mysql::MySqlPool, name: &str) -> i64 {
    let result = sqlx::query("insert into categories (name) values (?)")
        .bind(name)
        .execute(pool)
        .await
        .unwrap();
    result.last_insert_id() as i64
}

#[tokio::test]
async fn delete_rows_removes_a_single_row() {
    let pool = build_pool(&dev_profile()).await.unwrap();
    let driver = MySqlDriver::new(pool.clone(), "devdb".to_string());
    let id = insert_throwaway_category(&pool, "__test_delete_single__").await;

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(id));

    let affected = table_service::delete_rows(&driver, "devdb", "categories", &[row])
        .await
        .expect("delete_rows failed");
    assert_eq!(affected, 1);

    let result = table_service::fetch_table_rows(&driver, "devdb", "categories", 500, 0)
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
    let pool = build_pool(&dev_profile()).await.unwrap();
    let driver = MySqlDriver::new(pool.clone(), "devdb".to_string());
    let id1 = insert_throwaway_category(&pool, "__test_delete_multi_1__").await;
    let id2 = insert_throwaway_category(&pool, "__test_delete_multi_2__").await;

    let mut row1 = HashMap::new();
    row1.insert("id".to_string(), json!(id1));
    let mut row2 = HashMap::new();
    row2.insert("id".to_string(), json!(id2));

    let affected = table_service::delete_rows(&driver, "devdb", "categories", &[row1, row2])
        .await
        .expect("delete_rows failed");
    assert_eq!(affected, 2);

    let result = table_service::fetch_table_rows(&driver, "devdb", "categories", 500, 0)
        .await
        .unwrap();
    let id_index = result.columns.iter().position(|c| c == "id").unwrap();
    assert!(!result.rows.iter().any(|r| cell_json(&r[id_index]) == json!(id1)));
    assert!(!result.rows.iter().any(|r| cell_json(&r[id_index]) == json!(id2)));
}

#[tokio::test]
async fn delete_rows_with_empty_input_is_a_noop() {
    let driver = dev_driver().await;
    let affected = table_service::delete_rows(&driver, "devdb", "categories", &[])
        .await
        .expect("delete_rows with empty input should succeed as a no-op");
    assert_eq!(affected, 0);
}

#[tokio::test]
async fn delete_rows_fails_for_nonexistent_row() {
    let driver = dev_driver().await;

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(999_999_999));

    let result = table_service::delete_rows(&driver, "devdb", "categories", &[row]).await;
    assert!(result.is_err(), "deleting a row that doesn't exist should fail, not silently no-op");
}

#[tokio::test]
async fn update_cell_text_gives_clean_message_for_foreign_key_violation() {
    let driver = dev_driver().await;

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(1));

    let result =
        table_service::update_cell_text(&driver, "devdb", "orders", &row, "user_id", Some("999999999")).await;

    let err = result.expect_err("assigning orders.user_id to a nonexistent user should fail").to_string();
    assert!(
        err.to_lowercase().contains("match") || err.to_lowercase().contains("referenced"),
        "expected a foreign-key-shaped message, got: {err}"
    );
}

#[tokio::test]
async fn update_cell_text_gives_clean_message_for_not_null_violation() {
    let driver = dev_driver().await;

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(1));

    let result = table_service::update_cell_text(&driver, "devdb", "orders", &row, "user_id", None).await;

    let err = result.expect_err("setting a not-null column to null should fail").to_string();
    assert!(
        err.to_lowercase().contains("empty") || err.to_lowercase().contains("null"),
        "expected a not-null-shaped message, got: {err}"
    );
}

async fn insert_throwaway_user(pool: &sqlx::mysql::MySqlPool, email: &str) -> i64 {
    let result = sqlx::query("insert into users (email, full_name, metadata) values (?, 'Throwaway', '{}')")
        .bind(email)
        .execute(pool)
        .await
        .unwrap();
    result.last_insert_id() as i64
}

#[tokio::test]
async fn update_cell_text_gives_clean_message_for_unique_violation() {
    let pool = build_pool(&dev_profile()).await.unwrap();
    let driver = MySqlDriver::new(pool.clone(), "devdb".to_string());
    let taken_email = format!("__taken_{}@example.com", uuid_like_suffix());
    insert_throwaway_user(&pool, &taken_email).await;
    let colliding_id = insert_throwaway_user(&pool, &format!("__free_{}@example.com", uuid_like_suffix())).await;

    let mut row = HashMap::new();
    row.insert("id".to_string(), json!(colliding_id));

    let result = table_service::update_cell_text(&driver, "devdb", "users", &row, "email", Some(&taken_email)).await;

    sqlx::query("delete from users where full_name = 'Throwaway'")
        .execute(&pool)
        .await
        .expect("cleanup delete failed");

    let err = result.expect_err("colliding with an existing unique email should fail").to_string();
    assert!(
        err.to_lowercase().contains("already exists") || err.to_lowercase().contains("unique"),
        "expected a unique-violation-shaped message, got: {err}"
    );
}

fn uuid_like_suffix() -> String {
    format!("{:x}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())
}
