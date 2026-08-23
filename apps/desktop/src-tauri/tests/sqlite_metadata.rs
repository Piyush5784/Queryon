use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::driver::DatabaseDriver;
use queryon_lib::domain::query::service as query_service;
use queryon_lib::domain::schema::{DdlStatement, NewColumn};
use queryon_lib::infrastructure::sqlite::driver::SqliteDriver;
use queryon_lib::infrastructure::sqlite::pool::build_pool;

/// SQLite is embedded, not client-server — there is no long-lived
/// container to seed once and reuse (see every other `tests/*_metadata.rs`
/// file). Each test builds its own fresh file at a unique temp path
/// instead, using the exact schema `docker/initdb-sqlite/01_schema.sql`
/// defines, and runs it directly via `sqlx` rather than shelling out to
/// the `sqlite3` CLI so the test suite has no external tool dependency.
fn fresh_db_path(test_name: &str) -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir()
        .join(format!("queryon_sqlite_test_{test_name}_{nanos}.db"))
        .to_string_lossy()
        .to_string()
}

async fn seeded_driver(test_name: &str) -> SqliteDriver {
    let path = fresh_db_path(test_name);
    let schema_sql = std::fs::read_to_string(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../docker/initdb-sqlite/01_schema.sql"),
    )
    .expect("failed to read docker/initdb-sqlite/01_schema.sql");

    let profile = ConnectionProfile {
        id: "test".to_string(),
        name: "test".to_string(),
        engine: Engine::Sqlite,
        host: String::new(),
        port: 0,
        database: path,
        user: String::new(),
        password: String::new(),
        ssl_mode: SslMode::Disable,
        read_only: false,
        ssh_tunnel: None,
    };

    // `create_if_missing(false)` in `pool::build_pool` matches every
    // other engine's "connect to something that already exists"
    // semantics, so the file is created here first via a
    // `create_if_missing(true)` bootstrap connection, then the schema is
    // loaded, then the real driver connects the normal way.
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;
    let bootstrap = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::from_str(&format!("sqlite:{}", profile.database))
                .unwrap()
                .create_if_missing(true),
        )
        .await
        .expect("failed to create sqlite test file");
    sqlx::raw_sql(sqlx::AssertSqlSafe(schema_sql))
        .execute(&bootstrap)
        .await
        .expect("failed to load schema into sqlite test file");
    bootstrap.close().await;

    SqliteDriver::new(build_pool(&profile).await.expect("failed to build pool"))
}

#[tokio::test]
async fn server_version_reports_a_sqlite_build() {
    let driver = seeded_driver("server_version").await;
    let version = driver.server_version().await.expect("server_version failed");
    assert!(!version.is_empty());
}

#[tokio::test]
async fn list_tables_sees_the_seeded_schema() {
    let driver = seeded_driver("list_tables").await;
    let tables = driver.list_tables().await.expect("list_tables failed");
    let names: Vec<&str> = tables.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"categories"));
    assert!(names.contains(&"products"));
    assert!(!names.contains(&"sqlite_sequence"), "internal bookkeeping table should be filtered out");
}

#[tokio::test]
async fn get_table_columns_matches_the_seeded_schema() {
    let driver = seeded_driver("get_table_columns").await;
    let columns = driver.get_table_columns("main", "products").await.expect("get_table_columns failed");
    let names: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"id"));
    assert!(names.contains(&"category_id"));
    assert!(names.contains(&"price_cents"));
    let id_col = columns.iter().find(|c| c.name == "id").unwrap();
    assert!(id_col.is_primary_key);
}

/// A single-column `INTEGER PRIMARY KEY` (SQLite's rowid alias, what
/// `products.id` uses) gets no row at all in `PRAGMA index_list` —
/// confirmed live, it is simply not backed by a separate index the way a
/// composite `PRIMARY KEY(a, b)` or a `UNIQUE` column is. So
/// `list_indexes` correctly reports none `is_primary` here; the PK is
/// only visible via `list_constraints`/`get_table_columns`'s
/// `is_primary_key`, covered by other tests.
#[tokio::test]
async fn list_indexes_finds_the_implicit_unique_index() {
    let driver = seeded_driver("list_indexes").await;
    let indexes = driver.list_indexes("main", "products").await.expect("list_indexes failed");
    assert!(
        indexes.iter().any(|i| i.is_unique && !i.is_primary),
        "expected the implicit sqlite_autoindex_ from the `sku text unique` column constraint, got {indexes:?}"
    );
}

#[tokio::test]
async fn list_constraints_finds_the_primary_key_and_foreign_key() {
    let driver = seeded_driver("list_constraints").await;
    let constraints = driver.list_constraints("main", "order_items").await.expect("list_constraints failed");
    assert!(constraints.iter().any(|c| matches!(c.kind, queryon_lib::domain::schema::ConstraintKind::PrimaryKey)));
    let fk = constraints
        .iter()
        .find(|c| matches!(c.kind, queryon_lib::domain::schema::ConstraintKind::ForeignKey) && c.referenced_table.as_deref() == Some("orders"));
    assert!(fk.is_some(), "expected a foreign key to orders, got {constraints:?}");
}

#[tokio::test]
async fn get_table_ddl_returns_the_stored_create_table_text() {
    let driver = seeded_driver("get_table_ddl").await;
    let ddl = driver.get_table_ddl("main", "products").await.expect("get_table_ddl failed");
    assert!(ddl.to_lowercase().contains("create table"));
    assert!(ddl.contains("products"));
}

#[tokio::test]
async fn fetch_table_rows_reads_seeded_data() {
    let driver = seeded_driver("fetch_rows").await;
    let result = driver
        .fetch_table_rows("main", "products", 10, 0, &[], &[])
        .await
        .expect("fetch_table_rows failed");
    assert_eq!(result.row_count, 4);
}

#[tokio::test]
async fn count_table_rows_matches_the_seeded_row_count() {
    let driver = seeded_driver("count_rows").await;
    let count = driver.count_table_rows("main", "products", &[]).await.expect("count_table_rows failed");
    assert_eq!(count, 4);
}

#[tokio::test]
async fn insert_update_delete_round_trip() {
    use serde_json::json;
    use std::collections::HashMap;

    let driver = seeded_driver("round_trip").await;

    let mut categories = driver.fetch_table_rows("main", "categories", 10, 0, &[], &[]).await.expect("fetch categories failed");
    let name_idx = categories.columns.iter().position(|c| c == "name").unwrap();
    let id_idx = categories.columns.iter().position(|c| c == "id").unwrap();
    let books_row = categories.rows.drain(..).find(|r| r[name_idx] == "\"Books\"").expect("expected the seeded Books category");
    let category_id = books_row[id_idx].clone();

    let mut values: HashMap<String, serde_json::Value> = HashMap::new();
    values.insert("sku".to_string(), json!(format!("TEST-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())));
    values.insert("name".to_string(), json!("Doohickey"));
    values.insert("category_id".to_string(), serde_json::from_str(&category_id).unwrap());
    values.insert("price_cents".to_string(), json!(499));
    driver.insert_row("main", "products", &values).await.expect("insert_row failed");

    let after_insert = driver.count_table_rows("main", "products", &[]).await.expect("count after insert failed");
    assert_eq!(after_insert, 5);

    let mut products = driver.fetch_table_rows("main", "products", 10, 0, &[], &[]).await.expect("fetch products failed");
    let name_idx = products.columns.iter().position(|c| c == "name").unwrap();
    let row = products.rows.drain(..).find(|r| r[name_idx] == "\"Doohickey\"").expect("expected the row just inserted");
    let row_map: HashMap<String, serde_json::Value> = products
        .columns
        .iter()
        .zip(row.iter())
        .map(|(c, v)| (c.clone(), serde_json::from_str(v).unwrap()))
        .collect();

    driver.update_cell_text("main", "products", &row_map, "name", Some("Updated Doohickey")).await.expect("update_cell_text failed");

    let deleted = driver.delete_rows("main", "products", &[row_map]).await.expect("delete_rows failed");
    assert_eq!(deleted, 1);

    let after_delete = driver.count_table_rows("main", "products", &[]).await.expect("count after delete failed");
    assert_eq!(after_delete, 4);
}

#[tokio::test]
async fn execute_query_runs_arbitrary_sql() {
    let driver = seeded_driver("execute_query").await;
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

/// Native path — `notes` isn't part of any PRIMARY KEY/UNIQUE constraint,
/// so this should use the fast `ALTER TABLE ... DROP COLUMN`, not the
/// rebuild dance.
#[tokio::test]
async fn execute_ddl_add_and_drop_column_round_trips() {
    let driver = seeded_driver("add_drop_column").await;

    let add = vec![DdlStatement::AddColumn {
        table: "products".to_string(),
        column: NewColumn { name: "notes".to_string(), data_type: "text".to_string(), is_nullable: true, default: None },
    }];
    let batch = driver.execute_ddl("main", &add).await.expect("add column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("main", "products").await.expect("get_table_columns failed");
    assert!(columns.iter().any(|c| c.name == "notes"));

    let drop = vec![DdlStatement::DropColumn { table: "products".to_string(), column: "notes".to_string() }];
    let batch = driver.execute_ddl("main", &drop).await.expect("drop column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("main", "products").await.expect("get_table_columns failed");
    assert!(!columns.iter().any(|c| c.name == "notes"));
}

/// Rebuild-table path — SQLite has no `ALTER COLUMN` at all, so this
/// exercises the create/copy/drop/rename/reindex dance and checks data +
/// the surviving `idx_products_category_id` index both survive it.
#[tokio::test]
async fn execute_ddl_alter_column_rebuilds_the_table_and_preserves_data_and_indexes() {
    let driver = seeded_driver("alter_column").await;

    let before = driver.fetch_table_rows("main", "products", 10, 0, &[], &[]).await.expect("fetch before failed");
    assert_eq!(before.row_count, 4);

    let alter = vec![DdlStatement::AlterColumn {
        table: "products".to_string(),
        edit: queryon_lib::domain::schema::ColumnEdit {
            current_name: "price_cents".to_string(),
            column: NewColumn { name: "price_cents".to_string(), data_type: "text".to_string(), is_nullable: true, default: None },
        },
    }];
    let batch = driver.execute_ddl("main", &alter).await.expect("alter column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let after = driver.fetch_table_rows("main", "products", 10, 0, &[], &[]).await.expect("fetch after failed");
    assert_eq!(after.row_count, 4, "data should survive the rebuild");

    let columns = driver.get_table_columns("main", "products").await.expect("get_table_columns failed");
    let price_col = columns.iter().find(|c| c.name == "price_cents").unwrap();
    assert_eq!(price_col.data_type.to_lowercase(), "text");

    let indexes = driver.list_indexes("main", "products").await.expect("list_indexes failed");
    assert!(
        indexes.iter().any(|i| i.name == "idx_products_category_id"),
        "expected idx_products_category_id to survive the rebuild, got {indexes:?}"
    );
}

/// `DROP COLUMN` on a column that IS part of a UNIQUE constraint must
/// fall back to the rebuild dance (native `DROP COLUMN` refuses this,
/// confirmed live) — this is the one `DropColumn` path that can't use
/// the fast native statement.
#[tokio::test]
async fn execute_ddl_drop_column_on_a_unique_column_rebuilds_the_table() {
    let driver = seeded_driver("drop_unique_column").await;

    let drop = vec![DdlStatement::DropColumn { table: "categories".to_string(), column: "name".to_string() }];
    let err = driver.execute_ddl("main", &drop).await;
    // categories only has id + name — dropping name would leave a
    // single-column table, which is allowed; the real assertion is that
    // this doesn't error out the way native DROP COLUMN would.
    assert!(err.is_ok());
    let batch = err.unwrap();
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("main", "categories").await.expect("get_table_columns failed");
    assert!(!columns.iter().any(|c| c.name == "name"));
}
