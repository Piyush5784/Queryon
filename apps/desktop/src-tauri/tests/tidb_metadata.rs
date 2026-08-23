use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::driver::DatabaseDriver;
use queryon_lib::domain::query::service as query_service;
use queryon_lib::infrastructure::mysql::driver::MySqlDriver;
use queryon_lib::infrastructure::mysql::pool::build_pool;

fn dev_profile() -> ConnectionProfile {
    ConnectionProfile {
        id: "test".to_string(),
        name: "test".to_string(),
        engine: Engine::TiDb,
        host: "localhost".to_string(),
        port: 44000,
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

/// TiDb reuses `MySqlDriver` wholesale — Beekeeper's `TiDBClient extends
/// MysqlClient` has no overrides at all, and this proves a real
/// connection and query round-trip works against a live TiDB server the
/// same way.
#[tokio::test]
async fn server_version_reports_a_tidb_build() {
    let driver = dev_driver().await;
    let version = driver.server_version().await.expect("server_version failed");
    assert!(
        version.to_lowercase().contains("tidb"),
        "expected a TiDB version string, got {version:?}"
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

#[tokio::test]
async fn get_table_columns_matches_the_seeded_schema() {
    let driver = dev_driver().await;
    let columns = driver.get_table_columns("devdb", "products").await.expect("get_table_columns failed");
    let names: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"id"));
    assert!(names.contains(&"category_id"));
    assert!(names.contains(&"price_cents"));
}

#[tokio::test]
async fn list_indexes_finds_the_primary_key_index() {
    let driver = dev_driver().await;
    let indexes = driver.list_indexes("devdb", "products").await.expect("list_indexes failed");
    assert!(indexes.iter().any(|i| i.is_primary), "expected at least one primary-key index, got {indexes:?}");
}

#[tokio::test]
async fn list_constraints_finds_the_foreign_key() {
    let driver = dev_driver().await;
    let constraints = driver.list_constraints("devdb", "products").await.expect("list_constraints failed");
    assert!(!constraints.is_empty());
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

#[tokio::test]
async fn insert_update_delete_round_trip() {
    use serde_json::json;
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    let driver = dev_driver().await;
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let name = format!("__tidb_test_category_{unique}__");
    let updated_name = format!("__tidb_test_category_{unique}_updated__");

    let mut values: HashMap<String, serde_json::Value> = HashMap::new();
    values.insert("name".to_string(), json!(name));
    driver.insert_row("devdb", "categories", &values).await.expect("insert_row failed");

    let categories = driver
        .fetch_table_rows("devdb", "categories", 1000, 0, &[], &[])
        .await
        .expect("fetch categories failed");
    let name_idx = categories.columns.iter().position(|c| c == "name").unwrap();
    let expected_cell = format!("\"{name}\"");
    let row = categories
        .rows
        .iter()
        .find(|r| r[name_idx] == expected_cell)
        .expect("expected the row just inserted")
        .clone();
    let row_map: HashMap<String, serde_json::Value> = categories
        .columns
        .iter()
        .zip(row.iter())
        .map(|(c, v)| (c.clone(), serde_json::from_str(v).unwrap()))
        .collect();

    driver
        .update_cell_text("devdb", "categories", &row_map, "name", Some(&updated_name))
        .await
        .expect("update_cell_text failed");

    let mut row_map_updated = row_map.clone();
    row_map_updated.insert("name".to_string(), json!(updated_name));
    let deleted = driver
        .delete_rows("devdb", "categories", &[row_map_updated])
        .await
        .expect("delete_rows failed");
    assert_eq!(deleted, 1);
}
