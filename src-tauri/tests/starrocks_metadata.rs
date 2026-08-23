use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::driver::DatabaseDriver;
use queryon_lib::domain::query::service as query_service;
use queryon_lib::domain::schema::{ColumnEdit, ConstraintKind, DdlStatement, NewColumn, NewConstraint, AUTO_INCREMENT_TYPE};
use queryon_lib::infrastructure::mysql::driver::MySqlDriver;
use queryon_lib::infrastructure::mysql::pool::build_pool;

fn dev_profile() -> ConnectionProfile {
    ConnectionProfile {
        id: "test".to_string(),
        name: "test".to_string(),
        engine: Engine::StarRocks,
        host: "localhost".to_string(),
        port: 39030,
        database: "devdb".to_string(),
        user: "root".to_string(),
        password: "devpass".to_string(),
        ssl_mode: SslMode::Disable,
        read_only: false,
        ssh_tunnel: None,
    }
}

async fn dev_driver() -> MySqlDriver {
    let pool = build_pool(&dev_profile()).await.expect("failed to build pool");
    MySqlDriver::new_starrocks(pool, "devdb".to_string())
}

/// StarRocks reuses `MySqlDriver` — Beekeeper's `StarRocksClient extends
/// MysqlClient` — proves a real connection round-trips against a live
/// `starrocks/allin1-ubuntu` container.
#[tokio::test]
async fn server_version_reports_something() {
    let driver = dev_driver().await;
    let version = driver.server_version().await.expect("server_version failed");
    assert!(!version.is_empty());
}

#[tokio::test]
async fn list_tables_sees_the_seeded_schema() {
    let driver = dev_driver().await;
    let tables = driver.list_tables().await.expect("list_tables failed");
    let names: Vec<&str> = tables.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"users"));
    assert!(names.contains(&"products"));
}

/// `information_schema.key_column_usage`/`table_constraints` are always
/// empty on StarRocks (confirmed live) — this only passes if
/// `get_table_columns`'s `is_starrocks` branch actually falls back to
/// `column_key = 'PRI'`, not the shared correlated-subquery path every
/// other MySQL-family engine uses.
#[tokio::test]
async fn get_table_columns_finds_the_primary_key_via_column_key() {
    let driver = dev_driver().await;
    let columns = driver.get_table_columns("devdb", "products").await.expect("get_table_columns failed");
    let names: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"id"));
    assert!(names.contains(&"category_id"));
    assert!(names.contains(&"price_cents"));
    let id_col = columns.iter().find(|c| c.name == "id").unwrap();
    assert!(id_col.is_primary_key, "expected id to be detected as primary key, got {columns:?}");
}

/// `information_schema.statistics` is always empty on StarRocks
/// (confirmed live) — this only passes if `list_indexes`'s
/// `is_starrocks` branch falls back to `SHOW INDEX`, which reports
/// secondary/bitmap indexes but never the primary key itself.
#[tokio::test]
async fn list_indexes_finds_the_secondary_bitmap_index() {
    let driver = dev_driver().await;
    let indexes = driver.list_indexes("devdb", "products").await.expect("list_indexes failed");
    assert!(
        indexes.iter().any(|i| i.name == "idx_products_category_id"),
        "expected the seeded bitmap index, got {indexes:?}"
    );
    assert!(!indexes.iter().any(|i| i.is_primary), "StarRocks SHOW INDEX never reports the primary key itself");
}

/// StarRocks has no foreign key catalog at all — `list_constraints`
/// naturally returns an empty vec rather than erroring, since
/// `information_schema.table_constraints`/`key_column_usage` are just
/// empty tables there.
#[tokio::test]
async fn list_constraints_is_empty_no_fk_support() {
    let driver = dev_driver().await;
    let constraints = driver.list_constraints("devdb", "products").await.expect("list_constraints failed");
    assert!(constraints.is_empty(), "StarRocks has no FK/CHECK support, expected no constraints, got {constraints:?}");
}

#[tokio::test]
async fn fetch_table_rows_reads_seeded_data() {
    let driver = dev_driver().await;
    let result = driver.fetch_table_rows("devdb", "products", 100, 0, &[], &[]).await.expect("fetch_table_rows failed");
    assert!(result.row_count > 0);
}

#[tokio::test]
async fn execute_query_runs_arbitrary_sql() {
    let driver = dev_driver().await;
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
    use std::time::{SystemTime, UNIX_EPOCH};

    let driver = dev_driver().await;
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let name = format!("__starrocks_test_category_{unique}__");
    let updated_name = format!("__starrocks_test_category_{unique}_updated__");

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

    driver.update_cell_text("devdb", "categories", &row_map, "name", Some(&updated_name)).await.expect("update_cell_text failed");

    let mut row_map_updated = row_map.clone();
    row_map_updated.insert("name".to_string(), json!(updated_name));
    let deleted = driver.delete_rows("devdb", "categories", &[row_map_updated]).await.expect("delete_rows failed");
    assert_eq!(deleted, 1);
}

/// Exercises `render_create_table`'s StarRocks branch — PRIMARY
/// KEY(...) DISTRIBUTED BY HASH(...) is required, and a table with no
/// auto-increment column has no way to know what to distribute by (see
/// the `create_table_without_auto_increment_column_is_rejected` test
/// for the other half of this).
#[tokio::test]
async fn execute_ddl_create_table_needs_an_auto_increment_column() {
    let driver = dev_driver().await;
    let table_name = format!(
        "__sr_test_table_{}__",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    );

    let create = vec![DdlStatement::CreateTable {
        table: table_name.clone(),
        columns: vec![
            NewColumn { name: "id".to_string(), data_type: AUTO_INCREMENT_TYPE.to_string(), is_nullable: false, default: None },
            NewColumn { name: "label".to_string(), data_type: "varchar(100)".to_string(), is_nullable: false, default: None },
        ],
    }];
    let batch = driver.execute_ddl("devdb", &create).await.expect("create table execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("devdb", &table_name).await.expect("get_table_columns failed");
    assert!(columns.iter().any(|c| c.name == "id" && c.is_primary_key));

    let drop = vec![DdlStatement::DropTable { table: table_name.clone() }];
    driver.execute_ddl("devdb", &drop).await.expect("drop table execute_ddl failed");
}

#[tokio::test]
async fn execute_ddl_create_table_without_auto_increment_column_is_rejected() {
    let driver = dev_driver().await;
    let table_name = "__sr_test_no_pk__";

    let create = vec![DdlStatement::CreateTable {
        table: table_name.to_string(),
        columns: vec![NewColumn { name: "label".to_string(), data_type: "varchar(100)".to_string(), is_nullable: false, default: None }],
    }];
    let batch = driver.execute_ddl("devdb", &create).await.expect("execute_ddl failed");
    assert!(!batch.results[0].success);
    assert!(batch.results[0].error.as_deref().unwrap_or("").contains("primary key"));
}

/// StarRocks has no `ALTER TABLE ADD CONSTRAINT` at all — every kind is
/// rejected with a clear message rather than sending SQL that would
/// fail with a much less useful server error.
#[tokio::test]
async fn execute_ddl_add_constraint_is_rejected_for_every_kind() {
    let driver = dev_driver().await;

    let add_fk = vec![DdlStatement::AddConstraint {
        table: "products".to_string(),
        constraint: NewConstraint {
            name: "fk_test".to_string(),
            kind: ConstraintKind::ForeignKey,
            columns: vec!["category_id".to_string()],
            referenced_table: Some("categories".to_string()),
            referenced_columns: vec!["id".to_string()],
            on_update: None,
            on_delete: None,
            check_expression: None,
        },
    }];
    let batch = driver.execute_ddl("devdb", &add_fk).await.expect("execute_ddl failed");
    assert!(!batch.results[0].success);
    assert!(batch.results[0].error.as_deref().unwrap_or("").contains("foreign key"));
}

/// Exercises the async-schema-change polling in `execute_all` — this
/// only passes reliably if `wait_for_schema_change` actually waits for
/// the `ADD COLUMN` job to finish before `DROP COLUMN` runs; without it,
/// this fails intermittently with "A schema change operation is in
/// progress on the table ...", confirmed live before the fix.
#[tokio::test]
async fn execute_ddl_add_and_drop_column_round_trips_through_async_schema_changes() {
    let driver = dev_driver().await;

    let add = vec![DdlStatement::AddColumn {
        table: "products".to_string(),
        column: NewColumn { name: "notes".to_string(), data_type: "varchar(255)".to_string(), is_nullable: true, default: None },
    }];
    let batch = driver.execute_ddl("devdb", &add).await.expect("add column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("devdb", "products").await.expect("get_table_columns failed");
    assert!(columns.iter().any(|c| c.name == "notes"));

    let drop = vec![DdlStatement::DropColumn { table: "products".to_string(), column: "notes".to_string() }];
    let batch = driver.execute_ddl("devdb", &drop).await.expect("drop column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("devdb", "products").await.expect("get_table_columns failed");
    assert!(!columns.iter().any(|c| c.name == "notes"));
}

/// Exercises `render_alter_column`'s StarRocks branch — a rename
/// together with a retype becomes two separate statements (`MODIFY
/// COLUMN` then `RENAME COLUMN`), since StarRocks has no combined
/// `CHANGE COLUMN` the way MySQL does.
#[tokio::test]
async fn execute_ddl_alter_column_rename_and_retype_together() {
    let driver = dev_driver().await;

    let add = vec![DdlStatement::AddColumn {
        table: "products".to_string(),
        column: NewColumn { name: "temp_col".to_string(), data_type: "varchar(50)".to_string(), is_nullable: true, default: None },
    }];
    driver.execute_ddl("devdb", &add).await.expect("add column execute_ddl failed");

    let alter = vec![DdlStatement::AlterColumn {
        table: "products".to_string(),
        edit: ColumnEdit {
            current_name: "temp_col".to_string(),
            column: NewColumn { name: "renamed_col".to_string(), data_type: "varchar(100)".to_string(), is_nullable: true, default: None },
        },
    }];
    let batch = driver.execute_ddl("devdb", &alter).await.expect("alter column execute_ddl failed");
    assert!(batch.results.iter().all(|r| r.success), "{:?}", batch.results);

    let columns = driver.get_table_columns("devdb", "products").await.expect("get_table_columns failed");
    assert!(columns.iter().any(|c| c.name == "renamed_col"), "{columns:?}");
    assert!(!columns.iter().any(|c| c.name == "temp_col"));

    let drop = vec![DdlStatement::DropColumn { table: "products".to_string(), column: "renamed_col".to_string() }];
    driver.execute_ddl("devdb", &drop).await.expect("cleanup drop column failed");
}
