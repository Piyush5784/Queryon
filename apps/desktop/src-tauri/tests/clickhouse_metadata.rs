use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::driver::DatabaseDriver;
use queryon_lib::domain::query::service as query_service;
use queryon_lib::domain::schema::{ColumnEdit, ConstraintKind, DdlStatement, NewColumn, NewConstraint, NewIndex, AUTO_INCREMENT_TYPE};
use queryon_lib::infrastructure::clickhouse::driver::ClickHouseDriver;
use queryon_lib::infrastructure::clickhouse::pool::build_client;

fn dev_profile() -> ConnectionProfile {
    ConnectionProfile {
        id: "test".to_string(),
        name: "test".to_string(),
        engine: Engine::ClickHouse,
        host: "localhost".to_string(),
        port: 48123,
        database: "devdb".to_string(),
        user: "devuser".to_string(),
        password: "devpass".to_string(),
        ssl_mode: SslMode::Disable,
        read_only: false,
        ssh_tunnel: None,
    }
}

async fn dev_driver() -> ClickHouseDriver {
    let client = build_client(&dev_profile()).await.expect("failed to build client");
    ClickHouseDriver::new(client)
}

fn unique_table_name(prefix: &str) -> String {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    format!("__ch_{prefix}_{nanos}__")
}

async fn create_categories_table(driver: &ClickHouseDriver, table: &str) {
    let create = vec![DdlStatement::CreateTable {
        table: table.to_string(),
        columns: vec![
            NewColumn { name: "id".to_string(), data_type: AUTO_INCREMENT_TYPE.to_string(), is_nullable: false, default: None },
            NewColumn { name: "name".to_string(), data_type: "String".to_string(), is_nullable: false, default: None },
        ],
    }];
    let batch = driver.execute_ddl("devdb", &create).await.expect("create table execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);
}

async fn drop_table(driver: &ClickHouseDriver, table: &str) {
    let drop = vec![DdlStatement::DropTable { table: table.to_string() }];
    let _ = driver.execute_ddl("devdb", &drop).await;
}

#[tokio::test]
async fn server_version_reports_something() {
    let driver = dev_driver().await;
    let version = driver.server_version().await.expect("server_version failed");
    assert!(!version.is_empty());
}

#[tokio::test]
async fn list_tables_sees_a_created_table() {
    let driver = dev_driver().await;
    let table = unique_table_name("list_tables");
    create_categories_table(&driver, &table).await;

    let tables = driver.list_tables().await.expect("list_tables failed");
    assert!(tables.iter().any(|t| t.name == table));

    drop_table(&driver, &table).await;
}

/// `system.columns.is_in_primary_key` is the real PK-detection source on
/// ClickHouse (confirmed live) — this only passes if `get_table_columns`
/// actually reads that column rather than assuming the first column.
#[tokio::test]
async fn get_table_columns_finds_the_primary_key() {
    let driver = dev_driver().await;
    let table = unique_table_name("pk_cols");
    create_categories_table(&driver, &table).await;

    let columns = driver.get_table_columns("devdb", &table).await.expect("get_table_columns failed");
    let id_col = columns.iter().find(|c| c.name == "id").expect("id column missing");
    assert!(id_col.is_primary_key, "expected id to be detected as primary key, got {columns:?}");
    assert_eq!(id_col.data_type, "UInt64");

    drop_table(&driver, &table).await;
}

/// A column's `type` string wraps as `Nullable(Inner)` — confirmed live —
/// rather than a separate nullability flag. This only passes if
/// `get_table_columns` strips that wrapper into `is_nullable` and
/// reports the inner type as `data_type`.
#[tokio::test]
async fn get_table_columns_detects_nullable_wrapper() {
    let driver = dev_driver().await;
    let table = unique_table_name("nullable_cols");

    let create = vec![DdlStatement::CreateTable {
        table: table.clone(),
        columns: vec![
            NewColumn { name: "id".to_string(), data_type: AUTO_INCREMENT_TYPE.to_string(), is_nullable: false, default: None },
            NewColumn { name: "opt_note".to_string(), data_type: "String".to_string(), is_nullable: true, default: None },
        ],
    }];
    let batch = driver.execute_ddl("devdb", &create).await.expect("create table execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("devdb", &table).await.expect("get_table_columns failed");
    let note_col = columns.iter().find(|c| c.name == "opt_note").expect("opt_note column missing");
    assert!(note_col.is_nullable);
    assert_eq!(note_col.data_type, "String");

    drop_table(&driver, &table).await;
}

#[tokio::test]
async fn list_indexes_finds_primary_and_secondary() {
    let driver = dev_driver().await;
    let table = unique_table_name("indexes");
    create_categories_table(&driver, &table).await;

    let add_index = vec![DdlStatement::AddIndex {
        table: table.clone(),
        index: NewIndex { name: "idx_name".to_string(), columns: vec!["name".to_string()], is_unique: false },
    }];
    let batch = driver.execute_ddl("devdb", &add_index).await.expect("add index execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let indexes = driver.list_indexes("devdb", &table).await.expect("list_indexes failed");
    assert!(indexes.iter().any(|i| i.is_primary), "expected PRIMARY reported, got {indexes:?}");
    assert!(indexes.iter().any(|i| i.name == "idx_name"), "expected idx_name reported, got {indexes:?}");

    drop_table(&driver, &table).await;
}

/// ClickHouse has real CHECK constraint support (confirmed live —
/// enforced on insert) but no FK/UNIQUE via ALTER TABLE ADD CONSTRAINT
/// at all (confirmed live — parser only accepts CHECK/ASSUME there).
#[tokio::test]
async fn list_constraints_finds_check_constraint() {
    let driver = dev_driver().await;
    let table = unique_table_name("constraints");
    create_categories_table(&driver, &table).await;

    let add_check = vec![DdlStatement::AddConstraint {
        table: table.clone(),
        constraint: NewConstraint {
            name: "chk_id_positive".to_string(),
            kind: ConstraintKind::Check,
            columns: vec![],
            referenced_table: None,
            referenced_columns: vec![],
            on_update: None,
            on_delete: None,
            check_expression: Some("id > 0".to_string()),
        },
    }];
    let batch = driver.execute_ddl("devdb", &add_check).await.expect("add constraint execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let constraints = driver.list_constraints("devdb", &table).await.expect("list_constraints failed");
    assert!(constraints.iter().any(|c| c.name == "chk_id_positive" && c.kind == ConstraintKind::Check), "{constraints:?}");

    drop_table(&driver, &table).await;
}

#[tokio::test]
async fn execute_ddl_add_constraint_rejects_foreign_key_and_unique() {
    let driver = dev_driver().await;
    let table = unique_table_name("fk_reject");
    create_categories_table(&driver, &table).await;

    let add_fk = vec![DdlStatement::AddConstraint {
        table: table.clone(),
        constraint: NewConstraint {
            name: "fk_test".to_string(),
            kind: ConstraintKind::ForeignKey,
            columns: vec!["id".to_string()],
            referenced_table: Some(table.clone()),
            referenced_columns: vec!["id".to_string()],
            on_update: None,
            on_delete: None,
            check_expression: None,
        },
    }];
    let batch = driver.execute_ddl("devdb", &add_fk).await.expect("execute_ddl failed");
    assert!(!batch.results[0].success);
    assert!(batch.results[0].error.as_deref().unwrap_or("").contains("foreign key"));

    let add_unique = vec![DdlStatement::AddConstraint {
        table: table.clone(),
        constraint: NewConstraint {
            name: "uq_test".to_string(),
            kind: ConstraintKind::Unique,
            columns: vec!["name".to_string()],
            referenced_table: None,
            referenced_columns: vec![],
            on_update: None,
            on_delete: None,
            check_expression: None,
        },
    }];
    let batch = driver.execute_ddl("devdb", &add_unique).await.expect("execute_ddl failed");
    assert!(!batch.results[0].success);

    drop_table(&driver, &table).await;
}

#[tokio::test]
async fn fetch_table_rows_reads_inserted_data() {
    let driver = dev_driver().await;
    let table = unique_table_name("fetch_rows");
    create_categories_table(&driver, &table).await;

    let mut values = std::collections::HashMap::new();
    values.insert("id".to_string(), serde_json::json!(1));
    values.insert("name".to_string(), serde_json::json!("Electronics"));
    driver.insert_row("devdb", &table, &values).await.expect("insert_row failed");

    let result = driver.fetch_table_rows("devdb", &table, 100, 0, &[], &[]).await.expect("fetch_table_rows failed");
    assert_eq!(result.row_count, 1);

    drop_table(&driver, &table).await;
}

#[tokio::test]
async fn execute_query_runs_arbitrary_sql() {
    let driver = dev_driver().await;
    let result = query_service::execute_query(&driver, "select 1 as one").await.expect("execute_query failed");
    match result {
        queryon_lib::domain::query::QueryResult::Rows { rows, .. } => assert_eq!(rows.len(), 1),
        queryon_lib::domain::query::QueryResult::Affected { .. } => panic!("expected a Rows result"),
    }
}

/// ClickHouse row updates/deletes go through `ALTER TABLE ...
/// UPDATE/DELETE` mutations (confirmed live) rather than plain
/// UPDATE/DELETE statements.
#[tokio::test]
async fn insert_update_delete_round_trip() {
    use serde_json::json;
    use std::collections::HashMap;

    let driver = dev_driver().await;
    let table = unique_table_name("crud");
    create_categories_table(&driver, &table).await;

    let mut values: HashMap<String, serde_json::Value> = HashMap::new();
    values.insert("id".to_string(), json!(1));
    values.insert("name".to_string(), json!("Books"));
    driver.insert_row("devdb", &table, &values).await.expect("insert_row failed");

    let mut row_map: HashMap<String, serde_json::Value> = HashMap::new();
    row_map.insert("id".to_string(), json!(1));
    row_map.insert("name".to_string(), json!("Books"));

    driver.update_cell_text("devdb", &table, &row_map, "name", Some("Books & Media")).await.expect("update_cell_text failed");

    let after_update = driver.fetch_table_rows("devdb", &table, 10, 0, &[], &[]).await.expect("fetch after update failed");
    let name_idx = after_update.columns.iter().position(|c| c == "name").unwrap();
    assert!(after_update.rows.iter().any(|r| r[name_idx] == "\"Books & Media\""), "{:?}", after_update.rows);

    let deleted = driver.delete_rows("devdb", &table, &[row_map]).await.expect("delete_rows failed");
    assert_eq!(deleted, 1);

    let after_delete = driver.fetch_table_rows("devdb", &table, 10, 0, &[], &[]).await.expect("fetch after delete failed");
    assert_eq!(after_delete.row_count, 0);

    drop_table(&driver, &table).await;
}

#[tokio::test]
async fn execute_ddl_create_table_needs_engine_clause_handled_automatically() {
    let driver = dev_driver().await;
    let table = unique_table_name("engine_clause");
    create_categories_table(&driver, &table).await;

    let ddl = driver.get_table_ddl("devdb", &table).await.expect("get_table_ddl failed");
    assert!(ddl.contains("MergeTree"), "{ddl}");

    drop_table(&driver, &table).await;
}

#[tokio::test]
async fn execute_ddl_add_and_drop_column_round_trip() {
    let driver = dev_driver().await;
    let table = unique_table_name("add_drop_col");
    create_categories_table(&driver, &table).await;

    let add = vec![DdlStatement::AddColumn {
        table: table.clone(),
        column: NewColumn { name: "notes".to_string(), data_type: "String".to_string(), is_nullable: true, default: None },
    }];
    let batch = driver.execute_ddl("devdb", &add).await.expect("add column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("devdb", &table).await.expect("get_table_columns failed");
    assert!(columns.iter().any(|c| c.name == "notes"));

    let drop = vec![DdlStatement::DropColumn { table: table.clone(), column: "notes".to_string() }];
    let batch = driver.execute_ddl("devdb", &drop).await.expect("drop column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("devdb", &table).await.expect("get_table_columns failed");
    assert!(!columns.iter().any(|c| c.name == "notes"));

    drop_table(&driver, &table).await;
}

/// Exercises `render_alter_column`'s ClickHouse branch — a rename
/// together with a retype becomes two separate statements (`MODIFY
/// COLUMN` under the current name, then `RENAME COLUMN ... TO ...`),
/// since ClickHouse's `MODIFY COLUMN` cannot rename in the same call
/// (confirmed live, same as StarRocks).
#[tokio::test]
async fn execute_ddl_alter_column_rename_and_retype_together() {
    let driver = dev_driver().await;
    let table = unique_table_name("alter_col");
    create_categories_table(&driver, &table).await;

    let add = vec![DdlStatement::AddColumn {
        table: table.clone(),
        column: NewColumn { name: "temp_col".to_string(), data_type: "String".to_string(), is_nullable: true, default: None },
    }];
    driver.execute_ddl("devdb", &add).await.expect("add column execute_ddl failed");

    let alter = vec![DdlStatement::AlterColumn {
        table: table.clone(),
        edit: ColumnEdit {
            current_name: "temp_col".to_string(),
            column: NewColumn { name: "renamed_col".to_string(), data_type: "String".to_string(), is_nullable: true, default: None },
        },
    }];
    let batch = driver.execute_ddl("devdb", &alter).await.expect("alter column execute_ddl failed");
    assert!(batch.results.iter().all(|r| r.success), "{:?}", batch.results);

    let columns = driver.get_table_columns("devdb", &table).await.expect("get_table_columns failed");
    assert!(columns.iter().any(|c| c.name == "renamed_col"), "{columns:?}");
    assert!(!columns.iter().any(|c| c.name == "temp_col"));

    drop_table(&driver, &table).await;
}

#[tokio::test]
async fn begin_transaction_is_unsupported() {
    let driver = dev_driver().await;
    let result = driver.begin_transaction("test-tab").await;
    assert!(result.is_err());
}
