use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::driver::DatabaseDriver;
use queryon_lib::domain::query::service as query_service;
use queryon_lib::domain::schema::{DdlStatement, NewColumn};
use queryon_lib::infrastructure::libsql::driver::LibSqlDriver;
use queryon_lib::infrastructure::libsql::pool::build_connection;

const LIBSQL_URL: &str = "http://localhost:58082";

static SCHEMA_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

struct SeededSchema {
    driver: LibSqlDriver,
    products: String,
    tables: Vec<String>,
}

impl SeededSchema {
    async fn cleanup(self) {
        let conn = match build_connection(&test_profile()).await {
            Ok(conn) => conn,
            Err(_) => return,
        };
        for table in self.tables.iter() {
            for attempt in 0..5 {
                match conn.execute_batch(&format!("drop table if exists {table};")).await {
                    Ok(_) => break,
                    Err(_) if attempt < 4 => {
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                    }
                    Err(e) => eprintln!("cleanup: failed to drop {table}: {e}"),
                }
            }
        }
    }
}

fn test_profile() -> ConnectionProfile {
    ConnectionProfile {
        id: "test".to_string(),
        name: "test".to_string(),
        engine: Engine::LibSql,
        host: LIBSQL_URL.to_string(),
        port: 0,
        database: String::new(),
        user: String::new(),
        password: String::new(),
        ssl_mode: SslMode::Disable,
        read_only: false,
        ssh_tunnel: None,
    }
}

async fn seeded_driver(test_name: &str) -> SeededSchema {
    let suffix = unique_suffix();
    let categories = format!("categories_{test_name}_{suffix}");
    let products = format!("products_{test_name}_{suffix}");
    let orders = format!("orders_{test_name}_{suffix}");
    let order_items = format!("order_items_{test_name}_{suffix}");

    let conn = build_connection(&test_profile()).await.expect("failed to connect to libsql server");

    let _schema_guard = SCHEMA_LOCK.lock().unwrap();

    conn.execute_batch(&format!(
        "
        create table {categories} (
            id integer primary key autoincrement,
            name text not null unique
        );
        create table {products} (
            id integer primary key autoincrement,
            category_id integer references {categories}(id) on delete set null,
            sku text not null unique,
            name text not null,
            price_cents integer not null check (price_cents >= 0),
            in_stock integer not null default 1
        );
        create index idx_{products}_category_id on {products}(category_id);
        create table {orders} (
            id integer primary key autoincrement,
            user_id integer not null,
            status text not null default 'pending',
            total_cents integer not null default 0,
            notes text
        );
        create table {order_items} (
            id integer primary key autoincrement,
            order_id integer not null references {orders}(id) on delete cascade,
            product_id integer not null references {products}(id) on delete restrict,
            quantity integer not null check (quantity > 0),
            unit_price_cents integer not null
        );
        insert into {categories} (name) values ('Electronics'), ('Books'), ('Home & Kitchen');
        insert into {products} (category_id, sku, name, price_cents, in_stock) values
            (1, 'ELEC-001-{suffix}', 'Wireless Mouse', 2499, 1),
            (1, 'ELEC-002-{suffix}', 'Mechanical Keyboard', 8999, 1),
            (2, 'BOOK-001-{suffix}', 'The Pragmatic Programmer', 3999, 1),
            (3, 'HOME-001-{suffix}', 'French Press', 1899, 0);
        insert into {orders} (user_id, status, total_cents, notes) values
            (1, 'paid', 11498, null),
            (1, 'shipped', 3999, 'Gift wrap requested');
        insert into {order_items} (order_id, product_id, quantity, unit_price_cents) values
            (1, 1, 1, 2499),
            (1, 2, 1, 8999);
        "
    ))
    .await
    .expect("failed to seed libsql schema");

    let tables = vec![order_items, orders, products.clone(), categories];
    let driver = LibSqlDriver::new(conn);
    SeededSchema { driver, products, tables }
}

#[tokio::test]
async fn server_version_reports_a_sqlite_compatible_build() {
    let schema = seeded_driver("server_version").await;
    let driver = &schema.driver;
    let version = driver.server_version().await.expect("server_version failed");
    assert!(!version.is_empty());
    schema.cleanup().await;
}

#[tokio::test]
async fn list_tables_sees_the_seeded_schema() {
    let schema = seeded_driver("list_tables").await;
    let driver = &schema.driver;
    let products = &schema.products;
    let tables = driver.list_tables().await.expect("list_tables failed");
    let names: Vec<&str> = tables.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&products.as_str()));
    schema.cleanup().await;
}

#[tokio::test]
async fn get_table_columns_matches_the_seeded_schema() {
    let schema = seeded_driver("get_table_columns").await;
    let driver = &schema.driver;
    let products = &schema.products;
    let columns = driver.get_table_columns("main", &products).await.expect("get_table_columns failed");
    let names: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();
    assert!(names.contains(&"id"));
    assert!(names.contains(&"category_id"));
    assert!(names.contains(&"price_cents"));
    let id_col = columns.iter().find(|c| c.name == "id").unwrap();
    assert!(id_col.is_primary_key);
    schema.cleanup().await;
}

#[tokio::test]
async fn list_indexes_finds_the_implicit_unique_index() {
    let schema = seeded_driver("list_indexes").await;
    let driver = &schema.driver;
    let products = &schema.products;
    let indexes = driver.list_indexes("main", &products).await.expect("list_indexes failed");
    assert!(
        indexes.iter().any(|i| i.is_unique && !i.is_primary),
        "expected the implicit sqlite_autoindex_ from the `sku text unique` column constraint, got {indexes:?}"
    );
    schema.cleanup().await;
}

#[tokio::test]
async fn list_constraints_finds_the_primary_key_and_foreign_key() {
    let schema = seeded_driver("list_constraints").await;
    let driver = &schema.driver;
    let suffix = unique_suffix();
    let order_items = format!("order_items_list_constraints_{suffix}");
    let orders = format!("orders_list_constraints_{suffix}");
    let products = format!("products_list_constraints_{suffix}");

    driver
        .execute_ddl(
            "main",
            &[DdlStatement::CreateTable {
                table: orders.clone(),
                columns: vec![NewColumn {
                    name: "id".to_string(),
                    data_type: queryon_lib::domain::schema::AUTO_INCREMENT_TYPE.to_string(),
                    is_nullable: false,
                    default: None,
                }],
            }],
        )
        .await
        .expect("create orders failed");
    driver
        .execute_ddl(
            "main",
            &[DdlStatement::CreateTable {
                table: products.clone(),
                columns: vec![NewColumn {
                    name: "id".to_string(),
                    data_type: queryon_lib::domain::schema::AUTO_INCREMENT_TYPE.to_string(),
                    is_nullable: false,
                    default: None,
                }],
            }],
        )
        .await
        .expect("create products failed");

    let conn = build_connection(&test_profile()).await.expect("failed to connect");

    conn.execute_batch(&format!(
        "create table {order_items} (
            id integer primary key autoincrement,
            order_id integer not null references {orders}(id) on delete cascade,
            product_id integer not null references {products}(id) on delete restrict
        );"
    ))
    .await
    .expect("failed to create order_items");

    let constraints = driver.list_constraints("main", &order_items).await.expect("list_constraints failed");
    assert!(constraints.iter().any(|c| matches!(c.kind, queryon_lib::domain::schema::ConstraintKind::PrimaryKey)));
    let fk = constraints
        .iter()
        .find(|c| matches!(c.kind, queryon_lib::domain::schema::ConstraintKind::ForeignKey) && c.referenced_table.as_deref() == Some(orders.as_str()));
    assert!(fk.is_some(), "expected a foreign key to {orders}, got {constraints:?}");

    conn.execute_batch(&format!(
        "drop table {order_items}; drop table {orders}; drop table {products};"
    ))
    .await
    .expect("failed to clean up list_constraints tables");
    schema.cleanup().await;
}

#[tokio::test]
async fn get_table_ddl_returns_the_stored_create_table_text() {
    let schema = seeded_driver("get_table_ddl").await;
    let driver = &schema.driver;
    let products = &schema.products;
    let ddl = driver.get_table_ddl("main", &products).await.expect("get_table_ddl failed");
    assert!(ddl.to_lowercase().contains("create table"));
    assert!(ddl.contains(products.as_str()));
    schema.cleanup().await;
}

#[tokio::test]
async fn fetch_table_rows_reads_seeded_data() {
    let schema = seeded_driver("fetch_rows").await;
    let driver = &schema.driver;
    let products = &schema.products;
    let result = driver
        .fetch_table_rows("main", &products, 10, 0, &[], &[])
        .await
        .expect("fetch_table_rows failed");
    assert_eq!(result.row_count, 4);
    schema.cleanup().await;
}

#[tokio::test]
async fn count_table_rows_matches_the_seeded_row_count() {
    let schema = seeded_driver("count_rows").await;
    let driver = &schema.driver;
    let products = &schema.products;
    let count = driver.count_table_rows("main", &products, &[]).await.expect("count_table_rows failed");
    assert_eq!(count, 4);
    schema.cleanup().await;
}

#[tokio::test]
async fn insert_update_delete_round_trip() {
    use serde_json::json;
    use std::collections::HashMap;

    let schema = seeded_driver("round_trip").await;
    let driver = &schema.driver;
    let products = &schema.products;

    let mut values: HashMap<String, serde_json::Value> = HashMap::new();
    values.insert("sku".to_string(), json!(format!("TEST-{}", unique_suffix())));
    values.insert("name".to_string(), json!("Doohickey"));
    values.insert("category_id".to_string(), json!(1));
    values.insert("price_cents".to_string(), json!(499));
    driver.insert_row("main", &products, &values).await.expect("insert_row failed");

    let after_insert = driver.count_table_rows("main", &products, &[]).await.expect("count after insert failed");
    assert_eq!(after_insert, 5);

    let mut rows = driver.fetch_table_rows("main", &products, 10, 0, &[], &[]).await.expect("fetch products failed");
    let name_idx = rows.columns.iter().position(|c| c == "name").unwrap();
    let row = rows.rows.drain(..).find(|r| r[name_idx] == "\"Doohickey\"").expect("expected the row just inserted");
    let row_map: HashMap<String, serde_json::Value> = rows
        .columns
        .iter()
        .zip(row.iter())
        .map(|(c, v)| (c.clone(), serde_json::from_str(v).unwrap()))
        .collect();

    driver.update_cell_text("main", &products, &row_map, "name", Some("Updated Doohickey")).await.expect("update_cell_text failed");

    let deleted = driver.delete_rows("main", &products, &[row_map]).await.expect("delete_rows failed");
    assert_eq!(deleted, 1);

    let after_delete = driver.count_table_rows("main", &products, &[]).await.expect("count after delete failed");
    assert_eq!(after_delete, 4);
    schema.cleanup().await;
}

#[tokio::test]
async fn execute_query_runs_arbitrary_sql() {
    let schema = seeded_driver("execute_query").await;
    let driver = &schema.driver;
    let products = &schema.products;
    let result = query_service::execute_query(driver, &format!("select count(*) from {products}"))
        .await
        .expect("execute_query failed");
    match result {
        queryon_lib::domain::query::QueryResult::Rows { rows, .. } => {
            assert_eq!(rows.len(), 1);
        }
        queryon_lib::domain::query::QueryResult::Affected { .. } => panic!("expected a Rows result"),
    }
    schema.cleanup().await;
}

#[tokio::test]
async fn execute_ddl_add_and_drop_column_round_trips() {
    let schema = seeded_driver("add_drop_column").await;
    let driver = &schema.driver;
    let products = &schema.products;

    let add = vec![DdlStatement::AddColumn {
        table: products.clone(),
        column: NewColumn { name: "notes".to_string(), data_type: "text".to_string(), is_nullable: true, default: None },
    }];
    let batch = driver.execute_ddl("main", &add).await.expect("add column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("main", &products).await.expect("get_table_columns failed");
    assert!(columns.iter().any(|c| c.name == "notes"));

    let drop = vec![DdlStatement::DropColumn { table: products.clone(), column: "notes".to_string() }];
    let batch = driver.execute_ddl("main", &drop).await.expect("drop column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = driver.get_table_columns("main", &products).await.expect("get_table_columns failed");
    assert!(!columns.iter().any(|c| c.name == "notes"));
    schema.cleanup().await;
}

#[tokio::test]
async fn execute_ddl_alter_column_rebuilds_the_table_and_preserves_data_and_indexes() {
    let schema = seeded_driver("alter_column").await;
    let driver = &schema.driver;
    let products = &schema.products;

    let before = driver.fetch_table_rows("main", &products, 10, 0, &[], &[]).await.expect("fetch before failed");
    assert_eq!(before.row_count, 4);

    let alter = vec![DdlStatement::AlterColumn {
        table: products.clone(),
        edit: queryon_lib::domain::schema::ColumnEdit {
            current_name: "price_cents".to_string(),
            column: NewColumn { name: "price_cents".to_string(), data_type: "text".to_string(), is_nullable: true, default: None },
        },
    }];
    let batch = driver.execute_ddl("main", &alter).await.expect("alter column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let after = driver.fetch_table_rows("main", &products, 10, 0, &[], &[]).await.expect("fetch after failed");
    assert_eq!(after.row_count, 4, "data should survive the rebuild");

    let columns = driver.get_table_columns("main", &products).await.expect("get_table_columns failed");
    let price_col = columns.iter().find(|c| c.name == "price_cents").unwrap();
    assert_eq!(price_col.data_type.to_lowercase(), "text");

    let indexes = driver.list_indexes("main", &products).await.expect("list_indexes failed");
    assert!(
        indexes.iter().any(|i| i.name == format!("idx_{products}_category_id")),
        "expected idx_{products}_category_id to survive the rebuild, got {indexes:?}"
    );
    schema.cleanup().await;
}
