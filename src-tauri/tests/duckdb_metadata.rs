use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::driver::DatabaseDriver;
use queryon_lib::domain::query::service as query_service;
use queryon_lib::domain::schema::{ColumnEdit, ConstraintKind, DdlStatement, NewColumn, NewConstraint, NewIndex, AUTO_INCREMENT_TYPE};
use queryon_lib::infrastructure::duckdb::driver::DuckDbDriver;
use queryon_lib::infrastructure::duckdb::pool::build_connection;

fn fresh_db_path(test_name: &str) -> String {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("queryon_duckdb_test_{test_name}_{nanos}.duckdb")).to_string_lossy().to_string()
}

fn profile_for(path: String) -> ConnectionProfile {
    ConnectionProfile {
        id: "test".to_string(),
        name: "test".to_string(),
        engine: Engine::DuckDb,
        host: String::new(),
        port: 0,
        database: path,
        user: String::new(),
        password: String::new(),
        ssl_mode: SslMode::Disable,
        read_only: false,
        ssh_tunnel: None,
    }
}

async fn seeded_driver(test_name: &str) -> DuckDbDriver {
    let path = fresh_db_path(test_name);
    let schema_sql = std::fs::read_to_string(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../docker/initdb-duckdb/01_schema.sql"),
    )
    .expect("failed to read docker/initdb-duckdb/01_schema.sql");

    let handle = build_connection(&profile_for(path)).expect("failed to open duckdb file");
    {
        let conn = handle.conn.lock().unwrap();
        conn.execute_batch(&schema_sql).expect("failed to load schema into duckdb test file");
    }
    DuckDbDriver::new(handle)
}

async fn empty_driver(test_name: &str) -> DuckDbDriver {
    let path = fresh_db_path(test_name);
    let handle = build_connection(&profile_for(path)).expect("failed to open duckdb file");
    DuckDbDriver::new(handle)
}

#[tokio::test]
async fn server_version_reports_a_duckdb_build() {
    let driver = empty_driver("server_version").await;
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

#[tokio::test]
async fn list_indexes_finds_the_seeded_index() {
    let driver = seeded_driver("list_indexes").await;
    let indexes = driver.list_indexes("main", "products").await.expect("list_indexes failed");
    assert!(indexes.iter().any(|i| i.name == "idx_products_category_id"));
}

#[tokio::test]
async fn list_constraints_finds_foreign_key_and_check() {
    let driver = seeded_driver("list_constraints").await;
    let constraints = driver.list_constraints("main", "products").await.expect("list_constraints failed");
    assert!(constraints.iter().any(|c| c.kind == ConstraintKind::ForeignKey && c.referenced_table.as_deref() == Some("categories")));
    assert!(constraints.iter().any(|c| c.kind == ConstraintKind::Check));
}

#[tokio::test]
async fn fetch_table_rows_reads_seeded_data() {
    let driver = seeded_driver("fetch_table_rows").await;
    let result = driver.fetch_table_rows("main", "products", 100, 0, &[], &[]).await.expect("fetch_table_rows failed");
    assert!(result.row_count > 0);
}

#[tokio::test]
async fn execute_query_runs_arbitrary_sql() {
    let driver = seeded_driver("execute_query").await;
    let result = query_service::execute_query(&driver, "select count(*) from categories").await.expect("execute_query failed");
    match result {
        queryon_lib::domain::query::QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 1),
        queryon_lib::domain::query::QueryResult::Affected { .. } => panic!("expected a Rows result"),
    }
}

#[tokio::test]
async fn insert_update_delete_round_trip() {
    use serde_json::json;
    use std::collections::HashMap;

    let driver = seeded_driver("crud").await;

    let mut values: HashMap<String, serde_json::Value> = HashMap::new();
    values.insert("name".to_string(), json!("__test_category__"));
    driver.insert_row("main", "categories", &values).await.expect("insert_row failed");

    let categories = driver.fetch_table_rows("main", "categories", 1000, 0, &[], &[]).await.expect("fetch categories failed");
    let name_idx = categories.columns.iter().position(|c| c == "name").unwrap();
    let row = categories
        .rows
        .iter()
        .find(|r| r[name_idx] == "\"__test_category__\"")
        .expect("expected the row just inserted")
        .clone();
    let row_map: HashMap<String, serde_json::Value> = categories
        .columns
        .iter()
        .zip(row.iter())
        .map(|(c, v)| (c.clone(), serde_json::from_str(v).unwrap()))
        .collect();

    driver.update_cell_text("main", "categories", &row_map, "name", Some("__test_category_updated__")).await.expect("update_cell_text failed");

    let mut row_map_updated = row_map.clone();
    row_map_updated.insert("name".to_string(), json!("__test_category_updated__"));
    let deleted = driver.delete_rows("main", "categories", &[row_map_updated]).await.expect("delete_rows failed");
    assert_eq!(deleted, 1);
}

#[tokio::test]
async fn execute_ddl_create_table_uses_sequence_for_auto_increment() {
    let driver = empty_driver("create_table_seq").await;

    let create = vec![DdlStatement::CreateTable {
        table: "widgets".to_string(),
        columns: vec![
            NewColumn { name: "id".to_string(), data_type: AUTO_INCREMENT_TYPE.to_string(), is_nullable: false, default: None },
            NewColumn { name: "label".to_string(), data_type: "VARCHAR".to_string(), is_nullable: false, default: None },
        ],
    }];
    let batch = driver.execute_ddl("main", &create).await.expect("create table execute_ddl failed");
    assert!(batch.results.iter().all(|r| r.success), "{:?}", batch.results);

    let columns = driver.get_table_columns("main", "widgets").await.expect("get_table_columns failed");
    let id_col = columns.iter().find(|c| c.name == "id").expect("id column missing");
    assert!(id_col.is_primary_key);
}

#[tokio::test]
async fn execute_ddl_add_constraint_is_rejected() {
    let driver = seeded_driver("add_constraint_rejected").await;

    let add_check = vec![DdlStatement::AddConstraint {
        table: "products".to_string(),
        constraint: NewConstraint {
            name: "chk_test".to_string(),
            kind: ConstraintKind::Check,
            columns: vec![],
            referenced_table: None,
            referenced_columns: vec![],
            on_update: None,
            on_delete: None,
            check_expression: Some("price_cents > 0".to_string()),
        },
    }];
    let batch = driver.execute_ddl("main", &add_check).await.expect("execute_ddl failed");
    assert!(!batch.results[0].success);
    assert!(batch.results[0].error.as_deref().unwrap_or("").contains("ALTER TABLE ADD CONSTRAINT"));
}

async fn standalone_table_driver(test_name: &str, table: &str) -> DuckDbDriver {
    let driver = empty_driver(test_name).await;
    let create = vec![DdlStatement::CreateTable {
        table: table.to_string(),
        columns: vec![
            NewColumn { name: "id".to_string(), data_type: AUTO_INCREMENT_TYPE.to_string(), is_nullable: false, default: None },
            NewColumn { name: "label".to_string(), data_type: "VARCHAR".to_string(), is_nullable: false, default: None },
        ],
    }];
    let batch = driver.execute_ddl("main", &create).await.expect("create table execute_ddl failed");
    assert!(batch.results.iter().all(|r| r.success), "{:?}", batch.results);
    driver
}

#[tokio::test]
async fn execute_ddl_add_and_drop_column_round_trip() {
    let driver = standalone_table_driver("add_drop_column", "widgets").await;

    let add = vec![DdlStatement::AddColumn {
        table: "widgets".to_string(),
        column: NewColumn { name: "notes".to_string(), data_type: "VARCHAR".to_string(), is_nullable: true, default: None },
    }];
    let batch = driver.execute_ddl("main", &add).await.expect("add column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("main", "widgets").await.expect("get_table_columns failed");
    assert!(columns.iter().any(|c| c.name == "notes"));

    let drop = vec![DdlStatement::DropColumn { table: "widgets".to_string(), column: "notes".to_string() }];
    let batch = driver.execute_ddl("main", &drop).await.expect("drop column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("main", "widgets").await.expect("get_table_columns failed");
    assert!(!columns.iter().any(|c| c.name == "notes"));
}

#[tokio::test]
async fn execute_ddl_alter_column_rename_and_retype_together() {
    let driver = standalone_table_driver("alter_column", "widgets").await;

    let add = vec![DdlStatement::AddColumn {
        table: "widgets".to_string(),
        column: NewColumn { name: "temp_col".to_string(), data_type: "VARCHAR".to_string(), is_nullable: true, default: None },
    }];
    driver.execute_ddl("main", &add).await.expect("add column execute_ddl failed");

    let alter = vec![DdlStatement::AlterColumn {
        table: "widgets".to_string(),
        edit: ColumnEdit {
            current_name: "temp_col".to_string(),
            column: NewColumn { name: "renamed_col".to_string(), data_type: "VARCHAR".to_string(), is_nullable: true, default: None },
        },
    }];
    let batch = driver.execute_ddl("main", &alter).await.expect("alter column execute_ddl failed");
    assert!(batch.results.iter().all(|r| r.success), "{:?}", batch.results);

    let columns = driver.get_table_columns("main", "widgets").await.expect("get_table_columns failed");
    assert!(columns.iter().any(|c| c.name == "renamed_col"), "{columns:?}");
    assert!(!columns.iter().any(|c| c.name == "temp_col"));
}

#[tokio::test]
async fn execute_ddl_add_index_and_drop_index_round_trip() {
    let driver = seeded_driver("add_drop_index").await;

    let add = vec![DdlStatement::AddIndex {
        table: "categories".to_string(),
        index: NewIndex { name: "idx_categories_name_test".to_string(), columns: vec!["name".to_string()], is_unique: false },
    }];
    let batch = driver.execute_ddl("main", &add).await.expect("add index execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let indexes = driver.list_indexes("main", "categories").await.expect("list_indexes failed");
    assert!(indexes.iter().any(|i| i.name == "idx_categories_name_test"));

    let drop = vec![DdlStatement::DropIndex { table: "categories".to_string(), index: "idx_categories_name_test".to_string() }];
    let batch = driver.execute_ddl("main", &drop).await.expect("drop index execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);
}

#[tokio::test]
async fn foreign_key_violation_is_rejected() {
    let driver = seeded_driver("fk_violation").await;
    use serde_json::json;
    use std::collections::HashMap;

    let mut values: HashMap<String, serde_json::Value> = HashMap::new();
    values.insert("category_id".to_string(), json!(999999));
    values.insert("sku".to_string(), json!("BAD-SKU"));
    values.insert("name".to_string(), json!("Bad Product"));
    values.insert("price_cents".to_string(), json!(100));

    let result = driver.insert_row("main", "products", &values).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn begin_transaction_is_unsupported() {
    let driver = empty_driver("transactions").await;
    let result = driver.begin_transaction("test-tab").await;
    assert!(result.is_err());
}
