use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::driver::DatabaseDriver;
use queryon_lib::domain::query::service as query_service;
use queryon_lib::domain::schema::{DdlStatement, NewColumn};
use queryon_lib::infrastructure::trino::driver::TrinoDriver;
use queryon_lib::infrastructure::trino::pool::build_client;

const TRINO_HOST: &str = "localhost";
const TRINO_PORT: u16 = 48080;
const CATALOG: &str = "memory";
const SCHEMA: &str = "default";

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

fn test_profile() -> ConnectionProfile {
    ConnectionProfile {
        id: "test".to_string(),
        name: "test".to_string(),
        engine: Engine::Trino,
        host: TRINO_HOST.to_string(),
        port: TRINO_PORT,
        database: CATALOG.to_string(),
        user: "queryon_test".to_string(),
        password: String::new(),
        ssl_mode: SslMode::Disable,
        read_only: false,
        ssh_tunnel: None,
    }
}

fn new_driver() -> TrinoDriver {
    let client = build_client(&test_profile()).expect("failed to build trino client");
    TrinoDriver::new(client)
}

struct SeededSchema {
    driver: TrinoDriver,
    products: String,
    tables: Vec<String>,
}

impl SeededSchema {
    async fn cleanup(self) {
        for table in self.tables.iter().rev() {
            for attempt in 0..5 {
                let statements = vec![DdlStatement::DropTable { table: table.clone() }];
                match self.driver.execute_ddl(SCHEMA, &statements).await {
                    Ok(batch) if batch.results.first().map(|r| r.success).unwrap_or(false) => break,
                    Ok(_) if attempt < 4 => {
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    }
                    Ok(batch) => eprintln!("cleanup: failed to drop {table}: {:?}", batch.results.first().and_then(|r| r.error.clone())),
                    Err(_) if attempt < 4 => {
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    }
                    Err(e) => eprintln!("cleanup: failed to drop {table}: {e}"),
                }
            }
        }
    }
}

async fn seeded_driver(test_name: &str) -> SeededSchema {
    let suffix = unique_suffix();
    let products = format!("products_{test_name}_{suffix}");

    let driver = new_driver();

    let create = vec![DdlStatement::CreateTable {
        table: products.clone(),
        columns: vec![
            NewColumn { name: "id".to_string(), data_type: "integer".to_string(), is_nullable: false, default: None },
            NewColumn { name: "sku".to_string(), data_type: "varchar".to_string(), is_nullable: false, default: None },
            NewColumn { name: "name".to_string(), data_type: "varchar".to_string(), is_nullable: false, default: None },
            NewColumn { name: "price_cents".to_string(), data_type: "integer".to_string(), is_nullable: false, default: None },
        ],
    }];
    let batch = driver.execute_ddl(SCHEMA, &create).await.expect("create table failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let inserts = [
        ("1", "'ELEC-001'", "'Wireless Mouse'", "2499"),
        ("2", "'ELEC-002'", "'Mechanical Keyboard'", "8999"),
        ("3", "'BOOK-001'", "'The Pragmatic Programmer'", "3999"),
        ("4", "'HOME-001'", "'French Press'", "1899"),
    ];
    for (id, sku, name, price) in inserts {
        let sql = format!("insert into {CATALOG}.{SCHEMA}.\"{products}\" (id, sku, name, price_cents) values ({id}, {sku}, {name}, {price})");
        query_service::execute_query(&driver, &sql).await.expect("seed insert failed");
    }

    let tables = vec![products.clone()];
    SeededSchema { driver, products, tables }
}

#[tokio::test]
async fn server_version_reports_a_trino_build() {
    let schema = seeded_driver("server_version").await;
    let driver = &schema.driver;
    let version = driver.server_version().await.expect("server_version failed");
    assert!(version.starts_with("Trino"));
    schema.cleanup().await;
}

#[tokio::test]
async fn list_tables_sees_the_seeded_table() {
    let schema = seeded_driver("list_tables").await;
    let driver = &schema.driver;
    let products = &schema.products;
    let tables = driver.list_tables().await.expect("list_tables failed");
    let names: Vec<&str> = tables.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&products.as_str()), "expected {products} in {names:?}");
    schema.cleanup().await;
}

#[tokio::test]
async fn get_table_columns_matches_the_seeded_schema() {
    let schema = seeded_driver("get_table_columns").await;
    let driver = &schema.driver;
    let products = &schema.products;
    let columns = driver.get_table_columns(SCHEMA, products).await.expect("get_table_columns failed");
    let names: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"id"));
    assert!(names.contains(&"sku"));
    assert!(names.contains(&"price_cents"));
    schema.cleanup().await;
}

#[tokio::test]
async fn list_indexes_returns_empty_since_memory_connector_has_no_indexes() {
    let schema = seeded_driver("list_indexes").await;
    let driver = &schema.driver;
    let products = &schema.products;
    let indexes = driver.list_indexes(SCHEMA, products).await.expect("list_indexes failed");
    assert!(indexes.is_empty());
    schema.cleanup().await;
}

#[tokio::test]
async fn list_constraints_returns_empty_since_memory_connector_has_no_constraints() {
    let schema = seeded_driver("list_constraints").await;
    let driver = &schema.driver;
    let products = &schema.products;
    let constraints = driver.list_constraints(SCHEMA, products).await.expect("list_constraints failed");
    assert!(constraints.is_empty());
    schema.cleanup().await;
}

#[tokio::test]
async fn get_table_ddl_returns_the_stored_create_table_text() {
    let schema = seeded_driver("get_table_ddl").await;
    let driver = &schema.driver;
    let products = &schema.products;
    let ddl = driver.get_table_ddl(SCHEMA, products).await.expect("get_table_ddl failed");
    assert!(ddl.to_lowercase().contains("create table"));
    schema.cleanup().await;
}

#[tokio::test]
async fn fetch_table_rows_reads_seeded_data() {
    let schema = seeded_driver("fetch_rows").await;
    let driver = &schema.driver;
    let products = &schema.products;
    let result = driver.fetch_table_rows(SCHEMA, products, 10, 0, &[], &[]).await.expect("fetch_table_rows failed");
    assert_eq!(result.row_count, 4);
    schema.cleanup().await;
}

#[tokio::test]
async fn count_table_rows_matches_the_seeded_row_count() {
    let schema = seeded_driver("count_rows").await;
    let driver = &schema.driver;
    let products = &schema.products;
    let count = driver.count_table_rows(SCHEMA, products, &[]).await.expect("count_table_rows failed");
    assert_eq!(count, 4);
    schema.cleanup().await;
}

#[tokio::test]
async fn insert_row_adds_a_new_row() {
    use serde_json::json;
    use std::collections::HashMap;

    let schema = seeded_driver("insert_row").await;
    let driver = &schema.driver;
    let products = &schema.products;

    let mut values: HashMap<String, serde_json::Value> = HashMap::new();
    values.insert("id".to_string(), json!(5));
    values.insert("sku".to_string(), json!(format!("TEST-{}", unique_suffix())));
    values.insert("name".to_string(), json!("Doohickey"));
    values.insert("price_cents".to_string(), json!(499));
    driver.insert_row(SCHEMA, products, &values).await.expect("insert_row failed");

    let after_insert = driver.count_table_rows(SCHEMA, products, &[]).await.expect("count after insert failed");
    assert_eq!(after_insert, 5);
    schema.cleanup().await;
}

#[tokio::test]
async fn execute_query_runs_arbitrary_sql() {
    let schema = seeded_driver("execute_query").await;
    let driver = &schema.driver;
    let products = &schema.products;
    let sql = format!("select count(*) from {CATALOG}.{SCHEMA}.\"{products}\"");
    let result = query_service::execute_query(driver, &sql).await.expect("execute_query failed");
    match result {
        queryon_lib::domain::query::QueryResult::Rows { rows, .. } => {
            assert_eq!(rows.len(), 1);
        }
        queryon_lib::domain::query::QueryResult::Affected { .. } => panic!("expected a Rows result"),
    }
    schema.cleanup().await;
}

#[tokio::test]
async fn execute_ddl_add_column_succeeds() {
    let schema = seeded_driver("add_column").await;
    let driver = &schema.driver;
    let products = &schema.products;

    let add = vec![DdlStatement::AddColumn {
        table: products.clone(),
        column: NewColumn { name: "notes".to_string(), data_type: "varchar".to_string(), is_nullable: true, default: None },
    }];
    let batch = driver.execute_ddl(SCHEMA, &add).await.expect("add column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns(SCHEMA, products).await.expect("get_table_columns failed");
    assert!(columns.iter().any(|c| c.name == "notes"));
    schema.cleanup().await;
}

#[tokio::test]
async fn execute_ddl_rename_table_succeeds() {
    let schema = seeded_driver("rename_table").await;
    let driver = &schema.driver;
    let products = schema.products.clone();
    let new_name = format!("{products}_renamed");

    let rename = vec![DdlStatement::RenameTable { table: products.clone(), new_name: new_name.clone() }];
    let batch = driver.execute_ddl(SCHEMA, &rename).await.expect("rename table execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let tables = driver.list_tables().await.expect("list_tables failed");
    let names: Vec<&str> = tables.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&new_name.as_str()));
    assert!(!names.contains(&products.as_str()));

    let cleanup_drop = vec![DdlStatement::DropTable { table: new_name }];
    driver.execute_ddl(SCHEMA, &cleanup_drop).await.expect("cleanup drop failed");

    drop(schema);
}

#[tokio::test]
async fn execute_ddl_drop_column_fails_cleanly_since_memory_connector_does_not_support_it() {
    let schema = seeded_driver("drop_column_unsupported").await;
    let driver = &schema.driver;
    let products = &schema.products;

    let drop = vec![DdlStatement::DropColumn { table: products.clone(), column: "sku".to_string() }];
    let result = driver.execute_ddl(SCHEMA, &drop).await.expect("execute_ddl call itself should not error");
    assert!(!result.results[0].success);
    let error = result.results[0].error.as_deref().unwrap_or_default();
    assert!(error.to_lowercase().contains("drop"), "expected a clean unsupported-drop-column error, got: {error}");
    schema.cleanup().await;
}

#[tokio::test]
async fn execute_ddl_alter_column_fails_cleanly_since_memory_connector_does_not_support_it() {
    let schema = seeded_driver("alter_column_unsupported").await;
    let driver = &schema.driver;
    let products = &schema.products;

    let alter = vec![DdlStatement::AlterColumn {
        table: products.clone(),
        edit: queryon_lib::domain::schema::ColumnEdit {
            current_name: "price_cents".to_string(),
            column: NewColumn { name: "price_cents".to_string(), data_type: "varchar".to_string(), is_nullable: true, default: None },
        },
    }];
    let result = driver.execute_ddl(SCHEMA, &alter).await.expect("execute_ddl call itself should not error");
    assert!(!result.results[0].success);
    let error = result.results[0].error.as_deref().unwrap_or_default();
    assert!(error.to_lowercase().contains("alter") || error.to_lowercase().contains("connector does not support"), "expected a clean unsupported-alter-column error, got: {error}");
    schema.cleanup().await;
}

#[tokio::test]
async fn execute_ddl_add_index_fails_cleanly_since_trino_has_no_index_concept() {
    let schema = seeded_driver("add_index_unsupported").await;
    let driver = &schema.driver;
    let products = &schema.products;

    let add_index = vec![DdlStatement::AddIndex {
        table: products.clone(),
        index: queryon_lib::domain::schema::NewIndex {
            name: "idx_sku".to_string(),
            columns: vec!["sku".to_string()],
            is_unique: false,
        },
    }];
    let result = driver.execute_ddl(SCHEMA, &add_index).await.expect("execute_ddl call itself should not error");
    assert!(!result.results[0].success);
    let error = result.results[0].error.as_deref().unwrap_or_default();
    assert!(error.to_lowercase().contains("index"), "expected a clean unsupported-index error, got: {error}");
    schema.cleanup().await;
}

#[tokio::test]
async fn execute_ddl_add_constraint_fails_cleanly_since_trino_has_no_constraint_concept() {
    let schema = seeded_driver("add_constraint_unsupported").await;
    let driver = &schema.driver;
    let products = &schema.products;

    let add_constraint = vec![DdlStatement::AddConstraint {
        table: products.clone(),
        constraint: queryon_lib::domain::schema::NewConstraint {
            name: "pk_products".to_string(),
            kind: queryon_lib::domain::schema::ConstraintKind::PrimaryKey,
            columns: vec!["id".to_string()],
            referenced_table: None,
            referenced_columns: vec![],
            on_update: None,
            on_delete: None,
            check_expression: None,
        },
    }];
    let result = driver.execute_ddl(SCHEMA, &add_constraint).await.expect("execute_ddl call itself should not error");
    assert!(!result.results[0].success);
    let error = result.results[0].error.as_deref().unwrap_or_default();
    assert!(error.to_lowercase().contains("constraint"), "expected a clean unsupported-constraint error, got: {error}");
    schema.cleanup().await;
}

#[tokio::test]
async fn update_json_cell_fails_cleanly_since_row_editing_is_not_supported() {
    use std::collections::HashMap;
    let schema = seeded_driver("update_json_cell_unsupported").await;
    let driver = &schema.driver;
    let products = &schema.products;
    let row: HashMap<String, serde_json::Value> = HashMap::new();
    let err = driver.update_json_cell(SCHEMA, products, &row, "name", &serde_json::json!("x")).await.unwrap_err();
    assert!(err.to_string().to_lowercase().contains("not supported"));
    schema.cleanup().await;
}

#[tokio::test]
async fn delete_rows_fails_cleanly_since_row_editing_is_not_supported() {
    let schema = seeded_driver("delete_rows_unsupported").await;
    let driver = &schema.driver;
    let products = &schema.products;
    let err = driver.delete_rows(SCHEMA, products, &[]).await.unwrap_err();
    assert!(err.to_string().to_lowercase().contains("not supported"));
    schema.cleanup().await;
}

#[tokio::test]
async fn cancel_query_fails_cleanly_since_cancellation_is_not_supported() {
    let schema = seeded_driver("cancel_query_unsupported").await;
    let driver = &schema.driver;
    let err = driver.cancel_query("some-tab").await.unwrap_err();
    assert!(err.to_string().to_lowercase().contains("not supported"));
    schema.cleanup().await;
}

#[tokio::test]
async fn transaction_lifecycle_begin_commit_and_error_paths() {
    let schema = seeded_driver("transactions").await;
    let driver = &schema.driver;
    let tab_id = "tx-tab";

    assert!(!driver.has_active_transaction(tab_id));
    driver.begin_transaction(tab_id).await.expect("begin_transaction failed");
    assert!(driver.has_active_transaction(tab_id));

    let second_begin = driver.begin_transaction(tab_id).await;
    assert!(second_begin.is_err());

    driver.commit_transaction(tab_id).await.expect("commit_transaction failed");
    assert!(!driver.has_active_transaction(tab_id));

    let commit_again = driver.commit_transaction(tab_id).await;
    assert!(commit_again.is_err());

    schema.cleanup().await;
}
