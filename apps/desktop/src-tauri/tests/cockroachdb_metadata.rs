use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::driver::DatabaseDriver;
use queryon_lib::domain::query::service as query_service;
use queryon_lib::domain::schema::{DdlStatement, NewColumn};
use queryon_lib::infrastructure::postgres::driver::PostgresDriver;
use queryon_lib::infrastructure::postgres::pool::build_pool;

fn dev_profile() -> ConnectionProfile {
    ConnectionProfile {
        id: "test".to_string(),
        name: "test".to_string(),
        engine: Engine::CockroachDb,
        host: "localhost".to_string(),
        port: 26257,
        database: "devdb".to_string(),
        user: "devuser".to_string(),
        // --insecure mode has no passwords at all — any value is
        // accepted for a user that exists, see docker-compose.yml.
        password: "".to_string(),
        ssl_mode: SslMode::Disable,
        read_only: false,
        ssh_tunnel: None,
    }
}

async fn dev_driver() -> PostgresDriver {
    PostgresDriver::new(build_pool(&dev_profile()).expect("failed to build pool"))
}

/// CockroachDb reuses `PostgresDriver` wholesale (see `Engine`'s doc
/// comment) — this just proves a real connection and query round-trip
/// works against CockroachDB's Postgres-wire-protocol server, the way
/// Beekeeper Studio's `CockroachClient extends PostgresClient` assumes.
/// `server_version` runs `SHOW server_version`, which CockroachDB
/// deliberately answers with an *emulated Postgres version* (e.g.
/// "13.0.0") rather than its own build string, for client-compatibility
/// reasons — so this only checks the query succeeds, not its content.
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
    assert!(names.contains(&"price"));
}

/// The one query most likely to need CockroachDB-specific handling per
/// Beekeeper's source (`cockroach.ts`'s `listTableIndexes` uses `SHOW
/// INDEXES FROM` instead of Postgres's `pg_indexes`/`pg_index` catalog
/// tables) — this proves whether `PostgresDriver`'s existing
/// `pg_catalog`-based query, unmodified, already works against
/// CockroachDB's catalog shim or actually needs that divergence.
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
    assert_eq!(result.row_count, 2);
}

#[tokio::test]
async fn list_constraints_finds_the_primary_and_foreign_keys() {
    let driver = dev_driver().await;
    let constraints = driver
        .list_constraints("public", "products")
        .await
        .expect("list_constraints failed");
    assert!(constraints.iter().any(|c| c.name.ends_with("_pkey")));
    assert!(constraints.iter().any(|c| c.name.ends_with("_fkey")));
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
    assert_eq!(count, 2);
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
    values.insert("name".to_string(), json!("Doohickey"));
    values.insert(
        "category_id".to_string(),
        serde_json::from_str(&category_id).expect("category id should decode"),
    );
    values.insert("price".to_string(), json!("4.99"));
    driver.insert_row("public", "products", &values).await.expect("insert_row failed");

    let after_insert = driver
        .count_table_rows("public", "products", &[])
        .await
        .expect("count after insert failed");
    assert_eq!(after_insert, 3);

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
    assert_eq!(after_delete, 2);
}

/// Smoke test, not the full DDL suite `ddl_operations.rs` runs against
/// Postgres — this milestone is about connect/browse/query/edit-row
/// parity for CockroachDB (Tier 1 in Docs/Phase-2-Database-Support.md),
/// so this just proves add/drop column round-trips correctly rather
/// than re-covering every DDL edge case for a second engine.
#[tokio::test]
async fn execute_ddl_add_and_drop_column_round_trips() {
    let driver = dev_driver().await;

    let add = vec![DdlStatement::AddColumn {
        table: "products".to_string(),
        column: NewColumn {
            name: "notes".to_string(),
            data_type: "text".to_string(),
            is_nullable: true,
            default: None,
        },
    }];
    let batch = driver.execute_ddl("public", &add).await.expect("add column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("public", "products").await.expect("get_table_columns failed");
    assert!(columns.iter().any(|c| c.name == "notes"));

    let drop = vec![DdlStatement::DropColumn { table: "products".to_string(), column: "notes".to_string() }];
    let batch = driver.execute_ddl("public", &drop).await.expect("drop column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("public", "products").await.expect("get_table_columns failed");
    assert!(!columns.iter().any(|c| c.name == "notes"));
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
