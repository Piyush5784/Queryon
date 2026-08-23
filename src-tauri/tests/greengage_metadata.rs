use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::driver::DatabaseDriver;
use queryon_lib::domain::query::service as query_service;
use queryon_lib::infrastructure::postgres::driver::PostgresDriver;
use queryon_lib::infrastructure::postgres::pool::build_pool;

fn dev_profile() -> ConnectionProfile {
    ConnectionProfile {
        id: "test".to_string(),
        name: "test".to_string(),
        engine: Engine::GreengageDb,
        host: "localhost".to_string(),
        port: 55433,
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

/// GreengageDb reuses `PostgresDriver` wholesale (see `Engine`'s doc
/// comment) — proves a real connection round-trips against a live
/// `woblerr/greengage:6.31.0` container, the way Beekeeper's
/// `GreengageClient extends PostgresClient` (zero overrides) assumes.
/// GPDB6 answers with an emulated Postgres 9.4-era version string.
#[tokio::test]
async fn server_version_reports_a_pg_compatible_version_string() {
    let driver = dev_driver().await;
    let version = driver.server_version().await.expect("server_version failed");
    assert!(!version.is_empty());
}

#[tokio::test]
async fn list_tables_sees_the_seeded_schema() {
    let driver = dev_driver().await;
    let tables = driver.list_tables().await.expect("list_tables failed");
    let names: Vec<&str> = tables.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"categories"));
    assert!(names.contains(&"products"));
}

#[tokio::test]
async fn get_table_columns_matches_the_seeded_schema() {
    let driver = dev_driver().await;
    let columns = driver
        .get_table_columns("public", "products")
        .await
        .expect("get_table_columns failed");
    let names: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"id"));
    assert!(names.contains(&"name"));
    assert!(names.contains(&"category_id"));
    assert!(names.contains(&"price_cents"));
}

#[tokio::test]
async fn list_indexes_finds_the_primary_key_index() {
    let driver = dev_driver().await;
    let indexes = driver
        .list_indexes("public", "products")
        .await
        .expect("list_indexes failed");
    assert!(
        indexes.iter().any(|i| i.is_primary),
        "expected at least one primary-key index, got {indexes:?}"
    );
}

#[tokio::test]
async fn fetch_table_rows_reads_seeded_data() {
    let driver = dev_driver().await;
    let result = driver
        .fetch_table_rows("public", "products", 10, 0, &[], &[])
        .await
        .expect("fetch_table_rows failed");
    assert_eq!(result.row_count, 4);
}

/// `01_schema.sql` for GreengageDB drops the standalone UNIQUE
/// constraints/indexes the shared Postgres schema has (see the schema
/// file's comment) — a real GPDB architectural constraint found by
/// testing, not something Beekeeper's source flags. Constraint listing
/// still finds the primary key.
#[tokio::test]
async fn list_constraints_finds_the_primary_key() {
    let driver = dev_driver().await;
    let constraints = driver
        .list_constraints("public", "products")
        .await
        .expect("list_constraints failed");
    assert!(constraints.iter().any(|c| c.name.ends_with("_pkey")));
}

#[tokio::test]
async fn get_table_ddl_produces_a_create_table_statement() {
    let driver = dev_driver().await;
    let ddl = driver.get_table_ddl("public", "products").await.expect("get_table_ddl failed");
    assert!(ddl.to_lowercase().contains("create table"));
    assert!(ddl.contains("products"));
}

#[tokio::test]
async fn count_table_rows_matches_the_seeded_row_count() {
    let driver = dev_driver().await;
    let count = driver
        .count_table_rows("public", "products", &[])
        .await
        .expect("count_table_rows failed");
    assert_eq!(count, 4);
}

#[tokio::test]
async fn insert_update_and_delete_round_trip() {
    use serde_json::json;
    use std::collections::HashMap;

    let driver = dev_driver().await;

    let mut categories = driver
        .fetch_table_rows("public", "categories", 10, 0, &[], &[])
        .await
        .expect("fetch categories failed");
    let books_row = categories
        .rows
        .drain(..)
        .find(|r| r[categories.columns.iter().position(|c| c == "name").unwrap()] == "\"Books\"")
        .expect("expected the seeded Books category");
    let category_id = books_row[categories.columns.iter().position(|c| c == "id").unwrap()].clone();

    let mut values: HashMap<String, serde_json::Value> = HashMap::new();
    values.insert("sku".to_string(), json!(format!("TEST-{}", chrono_stamp())));
    values.insert("name".to_string(), json!("Doohickey"));
    values.insert(
        "category_id".to_string(),
        serde_json::from_str(&category_id).expect("category id should decode"),
    );
    values.insert("price_cents".to_string(), json!(499));
    driver.insert_row("public", "products", &values).await.expect("insert_row failed");

    let after_insert = driver
        .count_table_rows("public", "products", &[])
        .await
        .expect("count after insert failed");
    assert_eq!(after_insert, 5);

    let mut products = driver
        .fetch_table_rows("public", "products", 10, 0, &[], &[])
        .await
        .expect("fetch products failed");
    let name_idx = products.columns.iter().position(|c| c == "name").unwrap();
    let row = products
        .rows
        .drain(..)
        .find(|r| r[name_idx] == "\"Doohickey\"")
        .expect("expected the row just inserted");
    let row_map: HashMap<String, serde_json::Value> = products
        .columns
        .iter()
        .zip(row.iter())
        .map(|(c, v)| (c.clone(), serde_json::from_str(v).unwrap()))
        .collect();

    driver
        .update_cell_text("public", "products", &row_map, "name", Some("Updated Doohickey"))
        .await
        .expect("update_cell_text failed");

    let deleted = driver
        .delete_rows("public", "products", &[row_map])
        .await
        .expect("delete_rows failed");
    assert_eq!(deleted, 1);

    let after_delete = driver
        .count_table_rows("public", "products", &[])
        .await
        .expect("count after delete failed");
    assert_eq!(after_delete, 4);
}

fn chrono_stamp() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
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
