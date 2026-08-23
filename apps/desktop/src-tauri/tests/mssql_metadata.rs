use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::driver::DatabaseDriver;
use queryon_lib::domain::query::service as query_service;
use queryon_lib::domain::schema::{ColumnEdit, ConstraintKind, DdlStatement, NewColumn};
use queryon_lib::infrastructure::mssql::driver::MssqlDriver;
use queryon_lib::infrastructure::mssql::pool::build_pool;

fn dev_profile() -> ConnectionProfile {
    ConnectionProfile {
        id: "test".to_string(),
        name: "test".to_string(),
        engine: Engine::SqlServer,
        host: "localhost".to_string(),
        port: 14330,
        database: "devdb".to_string(),
        user: "sa".to_string(),
        password: "DevPass123!".to_string(),
        ssl_mode: SslMode::Disable,
        read_only: false,
        ssh_tunnel: None,
    }
}

async fn dev_driver() -> MssqlDriver {
    MssqlDriver::new(build_pool(&dev_profile()).await.expect("failed to build pool"))
}

#[tokio::test]
async fn server_version_reports_a_sql_server_build() {
    let driver = dev_driver().await;
    let version = driver.server_version().await.expect("server_version failed");
    assert!(version.to_lowercase().contains("sql server"));
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
    let columns = driver.get_table_columns("dbo", "products").await.expect("get_table_columns failed");
    let names: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"id"));
    assert!(names.contains(&"category_id"));
    assert!(names.contains(&"price_cents"));
    let id_col = columns.iter().find(|c| c.name == "id").unwrap();
    assert!(id_col.is_primary_key);
}

#[tokio::test]
async fn list_indexes_finds_the_primary_key_index() {
    let driver = dev_driver().await;
    let indexes = driver.list_indexes("dbo", "products").await.expect("list_indexes failed");
    assert!(indexes.iter().any(|i| i.is_primary), "expected a primary-key index, got {indexes:?}");
}

#[tokio::test]
async fn list_constraints_finds_the_primary_and_foreign_keys() {
    let driver = dev_driver().await;
    let constraints = driver.list_constraints("dbo", "order_items").await.expect("list_constraints failed");
    assert!(constraints.iter().any(|c| matches!(c.kind, ConstraintKind::PrimaryKey)));
    let fk = constraints
        .iter()
        .find(|c| matches!(c.kind, ConstraintKind::ForeignKey) && c.referenced_table.as_deref() == Some("orders"));
    assert!(fk.is_some(), "expected a foreign key to orders, got {constraints:?}");
}

#[tokio::test]
async fn get_table_ddl_produces_a_create_table_statement() {
    let driver = dev_driver().await;
    let ddl = driver.get_table_ddl("dbo", "products").await.expect("get_table_ddl failed");
    assert!(ddl.to_lowercase().contains("create table"));
    assert!(ddl.contains("products"));
}

#[tokio::test]
async fn fetch_table_rows_reads_seeded_data() {
    let driver = dev_driver().await;
    let result = driver.fetch_table_rows("dbo", "products", 10, 0, &[], &[]).await.expect("fetch_table_rows failed");
    assert_eq!(result.row_count, 4);
}

#[tokio::test]
async fn count_table_rows_matches_the_seeded_row_count() {
    let driver = dev_driver().await;
    let count = driver.count_table_rows("dbo", "products", &[]).await.expect("count_table_rows failed");
    assert_eq!(count, 4);
}

#[tokio::test]
async fn insert_update_and_delete_round_trip() {
    use serde_json::json;
    use std::collections::HashMap;

    let driver = dev_driver().await;

    let mut categories = driver.fetch_table_rows("dbo", "categories", 10, 0, &[], &[]).await.expect("fetch categories failed");
    let name_idx = categories.columns.iter().position(|c| c == "name").unwrap();
    let id_idx = categories.columns.iter().position(|c| c == "id").unwrap();
    let books_row = categories.rows.drain(..).find(|r| r[name_idx] == "\"Books\"").expect("expected the seeded Books category");
    let category_id = books_row[id_idx].clone();

    let mut values: HashMap<String, serde_json::Value> = HashMap::new();
    values.insert(
        "sku".to_string(),
        json!(format!(
            "TEST-{}",
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        )),
    );
    values.insert("name".to_string(), json!("Doohickey"));
    values.insert("category_id".to_string(), serde_json::from_str(&category_id).unwrap());
    values.insert("price_cents".to_string(), json!(499));
    driver.insert_row("dbo", "products", &values).await.expect("insert_row failed");

    let after_insert = driver.count_table_rows("dbo", "products", &[]).await.expect("count after insert failed");
    assert_eq!(after_insert, 5);

    let mut products = driver.fetch_table_rows("dbo", "products", 10, 0, &[], &[]).await.expect("fetch products failed");
    let name_idx = products.columns.iter().position(|c| c == "name").unwrap();
    let row = products.rows.drain(..).find(|r| r[name_idx] == "\"Doohickey\"").expect("expected the row just inserted");
    let row_map: HashMap<String, serde_json::Value> = products
        .columns
        .iter()
        .zip(row.iter())
        .map(|(c, v)| (c.clone(), serde_json::from_str(v).unwrap()))
        .collect();

    driver.update_cell_text("dbo", "products", &row_map, "name", Some("Updated Doohickey")).await.expect("update_cell_text failed");

    let deleted = driver.delete_rows("dbo", "products", &[row_map]).await.expect("delete_rows failed");
    assert_eq!(deleted, 1);

    let after_delete = driver.count_table_rows("dbo", "products", &[]).await.expect("count after delete failed");
    assert_eq!(after_delete, 4);
}

#[tokio::test]
async fn execute_query_runs_arbitrary_sql() {
    let driver = dev_driver().await;
    let result = query_service::execute_query(&driver, "select count(*) as c from categories").await.expect("execute_query failed");
    match result {
        queryon_lib::domain::query::QueryResult::Rows { rows, .. } => {
            assert_eq!(rows.len(), 1);
        }
        queryon_lib::domain::query::QueryResult::Affected { .. } => panic!("expected a Rows result"),
    }
}

#[tokio::test]
async fn execute_ddl_add_and_drop_column_round_trips() {
    let driver = dev_driver().await;

    let add = vec![DdlStatement::AddColumn {
        table: "products".to_string(),
        column: NewColumn { name: "notes".to_string(), data_type: "nvarchar(max)".to_string(), is_nullable: true, default: None },
    }];
    let batch = driver.execute_ddl("dbo", &add).await.expect("add column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("dbo", "products").await.expect("get_table_columns failed");
    assert!(columns.iter().any(|c| c.name == "notes"));

    let drop = vec![DdlStatement::DropColumn { table: "products".to_string(), column: "notes".to_string() }];
    let batch = driver.execute_ddl("dbo", &drop).await.expect("drop column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("dbo", "products").await.expect("get_table_columns failed");
    assert!(!columns.iter().any(|c| c.name == "notes"));
}

/// Exercises the drop-default-constraint-first dance `render_alter_column`
/// implements — `in_stock` has a `DEFAULT 1` constraint from the seed
/// schema, so this only passes if that constraint is dropped before the
/// `ALTER COLUMN`, matching what real SQL Server requires (confirmed
/// live: this fails with "dependent on column" otherwise).
#[tokio::test]
async fn execute_ddl_alter_column_on_a_column_with_a_default_constraint() {
    let driver = dev_driver().await;

    let alter = vec![DdlStatement::AlterColumn {
        table: "products".to_string(),
        edit: ColumnEdit {
            current_name: "in_stock".to_string(),
            column: NewColumn { name: "in_stock".to_string(), data_type: "nvarchar(10)".to_string(), is_nullable: true, default: None },
        },
    }];
    let batch = driver.execute_ddl("dbo", &alter).await.expect("alter column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("dbo", "products").await.expect("get_table_columns failed");
    let col = columns.iter().find(|c| c.name == "in_stock").unwrap();
    assert!(col.data_type.to_lowercase().contains("nvarchar"));
}

/// Exercises the drop-unique-index-first dance `render_drop_column`
/// implements — `sku` backs a `UNIQUE` constraint from the seed schema,
/// so dropping it only passes if that index is dropped first (confirmed
/// live: fails with "dependent on column" otherwise).
#[tokio::test]
async fn execute_ddl_drop_column_on_a_unique_column() {
    let driver = dev_driver().await;

    let drop = vec![DdlStatement::DropColumn { table: "products".to_string(), column: "sku".to_string() }];
    let batch = driver.execute_ddl("dbo", &drop).await.expect("drop column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("dbo", "products").await.expect("get_table_columns failed");
    assert!(!columns.iter().any(|c| c.name == "sku"));
}
