use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::query::service as query_service;
use queryon_lib::domain::query::QueryResult;
use queryon_lib::domain::schema::service as schema_service;
use queryon_lib::domain::schema::{
    ColumnEdit, ConstraintKind, DdlStatement, ForeignKeyAction, NewColumn, NewConstraint, NewIndex,
};
use queryon_lib::error::AppError;
use queryon_lib::infrastructure::postgres::driver::PostgresDriver;
use queryon_lib::infrastructure::postgres::pool::build_pool;

async fn exec(driver: &PostgresDriver, sql: &str) -> Result<QueryResult, AppError> {
    query_service::execute_query(driver, sql).await
}

fn dev_profile() -> ConnectionProfile {
    ConnectionProfile {
        id: "test".to_string(),
        name: "test".to_string(),
        engine: Engine::Postgres,
        host: "localhost".to_string(),
        port: 55434,
        database: "devdb".to_string(),
        user: "devuser".to_string(),
        password: "devpass".to_string(),
        ssl_mode: SslMode::Disable,
        read_only: false,
        ssh_tunnel: None,
    }
}

async fn dev_driver() -> PostgresDriver {
    let pool = build_pool(&dev_profile()).expect("failed to build pool");
    PostgresDriver::new(pool)
}

/// Runs setup/teardown SQL (CREATE/DROP TABLE) directly through the
/// existing raw-query path — every DDL test creates its own scratch
/// table and drops it at the end, never touching the seeded
/// `users`/`orders`/etc. tables other suites rely on.
async fn raw_exec(driver: &PostgresDriver, sql: &str) {
    exec(driver, sql)
        .await
        .expect("scratch table setup/teardown SQL failed");
}

#[tokio::test]
async fn render_ddl_add_column_produces_expected_sql() {
    let driver = dev_driver().await;
    let statements = vec![DdlStatement::AddColumn {
        table: "ddl_render_test".to_string(),
        column: NewColumn {
            name: "email".to_string(),
            data_type: "text".to_string(),
            is_nullable: false,
            default: None,
        },
    }];

    let previews = schema_service::render_ddl(&driver, "public", &statements)
        .await
        .expect("render_ddl failed");

    assert_eq!(previews.len(), 1);
    assert!(previews[0].sql.contains("add column"));
    assert!(previews[0].sql.contains("\"email\""));
    assert!(previews[0].sql.contains("not null"));
}

#[tokio::test]
async fn execute_ddl_add_and_drop_column_round_trips() {
    let driver = dev_driver().await;
    raw_exec(&driver, "drop table if exists ddl_col_test;").await;
    raw_exec(
        &driver,
        "create table ddl_col_test (id serial primary key);",
    )
    .await;

    let add = vec![DdlStatement::AddColumn {
        table: "ddl_col_test".to_string(),
        column: NewColumn {
            name: "email".to_string(),
            data_type: "text".to_string(),
            is_nullable: true,
            default: None,
        },
    }];
    let batch = schema_service::execute_ddl(&driver, "public", &add)
        .await
        .expect("add column execute_ddl failed");
    assert!(!batch.rolled_back);
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = schema_service::get_table_columns(&driver, "public", "ddl_col_test")
        .await
        .expect("get_table_columns failed");
    assert!(columns.iter().any(|c| c.name == "email"));

    let drop = vec![DdlStatement::DropColumn {
        table: "ddl_col_test".to_string(),
        column: "email".to_string(),
    }];
    let batch = schema_service::execute_ddl(&driver, "public", &drop)
        .await
        .expect("drop column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = schema_service::get_table_columns(&driver, "public", "ddl_col_test")
        .await
        .expect("get_table_columns failed");
    assert!(!columns.iter().any(|c| c.name == "email"));

    raw_exec(&driver, "drop table ddl_col_test;").await;
}

#[tokio::test]
async fn execute_ddl_batch_rolls_back_on_failure() {
    let driver = dev_driver().await;
    raw_exec(&driver, "drop table if exists ddl_rollback_test;").await;
    raw_exec(
        &driver,
        "create table ddl_rollback_test (id serial primary key);",
    )
    .await;

    let statements = vec![
        DdlStatement::AddColumn {
            table: "ddl_rollback_test".to_string(),
            column: NewColumn {
                name: "email".to_string(),
                data_type: "text".to_string(),
                is_nullable: true,
                default: None,
            },
        },
        DdlStatement::AddColumn {
            table: "ddl_rollback_test".to_string(),
            column: NewColumn {
                name: "bogus".to_string(),
                data_type: "not_a_real_type".to_string(),
                is_nullable: true,
                default: None,
            },
        },
    ];

    let batch = schema_service::execute_ddl(&driver, "public", &statements)
        .await
        .expect("execute_ddl should not itself error");

    assert!(
        batch.rolled_back,
        "batch with a failing statement should be rolled back"
    );
    assert!(
        batch.results[0].success,
        "first statement should have run before the failure"
    );
    assert!(
        !batch.results[1].success,
        "second statement should have failed"
    );

    let columns = schema_service::get_table_columns(&driver, "public", "ddl_rollback_test")
        .await
        .expect("get_table_columns failed");
    assert!(
        !columns.iter().any(|c| c.name == "email"),
        "rollback should have undone the earlier successful ADD COLUMN"
    );

    raw_exec(&driver, "drop table ddl_rollback_test;").await;
}

#[tokio::test]
async fn execute_ddl_alter_column_renames_and_changes_type() {
    let driver = dev_driver().await;
    raw_exec(&driver, "drop table if exists ddl_alter_test;").await;
    raw_exec(
        &driver,
        "create table ddl_alter_test (id serial primary key, age text);",
    )
    .await;

    let statements = vec![DdlStatement::AlterColumn {
        table: "ddl_alter_test".to_string(),
        edit: ColumnEdit {
            current_name: "age".to_string(),
            column: NewColumn {
                name: "years_old".to_string(),
                data_type: "int4".to_string(),
                is_nullable: false,
                default: Some("0".to_string()),
            },
        },
    }];

    // Column starts out empty of data, so the text->int4 cast is safe.
    let batch = schema_service::execute_ddl(&driver, "public", &statements)
        .await
        .expect("alter column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = schema_service::get_table_columns(&driver, "public", "ddl_alter_test")
        .await
        .expect("get_table_columns failed");
    assert!(
        !columns.iter().any(|c| c.name == "age"),
        "old column name should be gone"
    );
    let renamed = columns
        .iter()
        .find(|c| c.name == "years_old")
        .expect("expected the renamed column");
    // Postgres reports int4 back under its canonical name "integer".
    assert_eq!(renamed.data_type, "integer");
    assert!(!renamed.is_nullable);
    assert!(renamed.default.is_some());

    raw_exec(&driver, "drop table ddl_alter_test;").await;
}

#[tokio::test]
async fn execute_ddl_add_index_and_drop_index() {
    let driver = dev_driver().await;
    raw_exec(&driver, "drop table if exists ddl_index_test;").await;
    raw_exec(
        &driver,
        "create table ddl_index_test (id serial primary key, email text);",
    )
    .await;

    let add = vec![DdlStatement::AddIndex {
        table: "ddl_index_test".to_string(),
        index: NewIndex {
            name: "ddl_index_test_email_idx".to_string(),
            columns: vec!["email".to_string()],
            is_unique: true,
        },
    }];
    let batch = schema_service::execute_ddl(&driver, "public", &add)
        .await
        .expect("add index execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let indexes = schema_service::list_indexes(&driver, "public", "ddl_index_test")
        .await
        .expect("list_indexes failed");
    let created = indexes
        .iter()
        .find(|i| i.name == "ddl_index_test_email_idx")
        .expect("expected the new index to be listed");
    assert!(created.is_unique);

    let drop = vec![DdlStatement::DropIndex {
        table: "ddl_index_test".to_string(),
        index: "ddl_index_test_email_idx".to_string(),
    }];
    let batch = schema_service::execute_ddl(&driver, "public", &drop)
        .await
        .expect("drop index execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let indexes = schema_service::list_indexes(&driver, "public", "ddl_index_test")
        .await
        .expect("list_indexes failed");
    assert!(!indexes.iter().any(|i| i.name == "ddl_index_test_email_idx"));

    raw_exec(&driver, "drop table ddl_index_test;").await;
}

#[tokio::test]
async fn execute_ddl_add_check_constraint_and_drop() {
    let driver = dev_driver().await;
    raw_exec(&driver, "drop table if exists ddl_constraint_test;").await;
    raw_exec(
        &driver,
        "create table ddl_constraint_test (id serial primary key, price numeric);",
    )
    .await;

    let add = vec![DdlStatement::AddConstraint {
        table: "ddl_constraint_test".to_string(),
        constraint: NewConstraint {
            name: "ddl_constraint_test_price_check".to_string(),
            kind: ConstraintKind::Check,
            columns: vec![],
            referenced_table: None,
            referenced_columns: vec![],
            on_update: None,
            on_delete: None,
            check_expression: Some("price >= 0".to_string()),
        },
    }];
    let batch = schema_service::execute_ddl(&driver, "public", &add)
        .await
        .expect("add constraint execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let constraints = schema_service::list_constraints(&driver, "public", "ddl_constraint_test")
        .await
        .expect("list_constraints failed");
    assert!(constraints
        .iter()
        .any(|c| c.name == "ddl_constraint_test_price_check"));

    let drop = vec![DdlStatement::DropConstraint {
        table: "ddl_constraint_test".to_string(),
        constraint: "ddl_constraint_test_price_check".to_string(),
    }];
    let batch = schema_service::execute_ddl(&driver, "public", &drop)
        .await
        .expect("drop constraint execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let constraints = schema_service::list_constraints(&driver, "public", "ddl_constraint_test")
        .await
        .expect("list_constraints failed");
    assert!(!constraints
        .iter()
        .any(|c| c.name == "ddl_constraint_test_price_check"));

    raw_exec(&driver, "drop table ddl_constraint_test;").await;
}

#[tokio::test]
async fn render_ddl_rejects_invalid_identifier() {
    let driver = dev_driver().await;
    let statements = vec![DdlStatement::DropColumn {
        table: "users; drop table users;".to_string(),
        column: "email".to_string(),
    }];

    let result = schema_service::render_ddl(&driver, "public", &statements).await;
    assert!(
        result.is_err(),
        "an invalid identifier should be rejected before rendering"
    );
}

#[tokio::test]
async fn execute_ddl_create_table_with_index_and_constraint() {
    let driver = dev_driver().await;
    raw_exec(&driver, "drop table if exists ddl_create_test;").await;

    let statements = vec![
        DdlStatement::CreateTable {
            table: "ddl_create_test".to_string(),
            columns: vec![
                NewColumn {
                    name: "id".to_string(),
                    data_type: "serial".to_string(),
                    is_nullable: false,
                    default: None,
                },
                NewColumn {
                    name: "email".to_string(),
                    data_type: "text".to_string(),
                    is_nullable: false,
                    default: None,
                },
            ],
        },
        DdlStatement::AddConstraint {
            table: "ddl_create_test".to_string(),
            constraint: NewConstraint {
                name: "ddl_create_test_pkey".to_string(),
                kind: ConstraintKind::PrimaryKey,
                columns: vec!["id".to_string()],
                referenced_table: None,
                referenced_columns: vec![],
                on_update: None,
                on_delete: None,
                check_expression: None,
            },
        },
        DdlStatement::AddIndex {
            table: "ddl_create_test".to_string(),
            index: NewIndex {
                name: "ddl_create_test_email_idx".to_string(),
                columns: vec!["email".to_string()],
                is_unique: true,
            },
        },
    ];

    let batch = schema_service::execute_ddl(&driver, "public", &statements)
        .await
        .expect("create table execute_ddl failed");
    assert!(!batch.rolled_back);
    for result in &batch.results {
        assert!(result.success, "{:?}", result.error);
    }

    let columns = schema_service::get_table_columns(&driver, "public", "ddl_create_test")
        .await
        .expect("get_table_columns failed");
    assert!(columns.iter().any(|c| c.name == "id"));
    assert!(columns.iter().any(|c| c.name == "email"));

    let constraints = schema_service::list_constraints(&driver, "public", "ddl_create_test")
        .await
        .expect("list_constraints failed");
    assert!(constraints.iter().any(|c| c.name == "ddl_create_test_pkey"));

    let indexes = schema_service::list_indexes(&driver, "public", "ddl_create_test")
        .await
        .expect("list_indexes failed");
    assert!(indexes
        .iter()
        .any(|i| i.name == "ddl_create_test_email_idx"));

    raw_exec(&driver, "drop table ddl_create_test;").await;
}

#[tokio::test]
async fn execute_ddl_rename_table() {
    let driver = dev_driver().await;
    raw_exec(&driver, "drop table if exists ddl_rename_test;").await;
    raw_exec(&driver, "drop table if exists ddl_rename_test_2;").await;
    raw_exec(
        &driver,
        "create table ddl_rename_test (id serial primary key);",
    )
    .await;

    let statements = vec![DdlStatement::RenameTable {
        table: "ddl_rename_test".to_string(),
        new_name: "ddl_rename_test_2".to_string(),
    }];
    let batch = schema_service::execute_ddl(&driver, "public", &statements)
        .await
        .expect("rename table execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let tables = schema_service::list_tables(&driver)
        .await
        .expect("list_tables failed");
    assert!(!tables.iter().any(|t| t.name == "ddl_rename_test"));
    assert!(tables.iter().any(|t| t.name == "ddl_rename_test_2"));

    raw_exec(&driver, "drop table ddl_rename_test_2;").await;
}

#[tokio::test]
async fn execute_ddl_drop_table() {
    let driver = dev_driver().await;
    raw_exec(&driver, "drop table if exists ddl_drop_test;").await;
    raw_exec(
        &driver,
        "create table ddl_drop_test (id serial primary key);",
    )
    .await;

    let statements = vec![DdlStatement::DropTable {
        table: "ddl_drop_test".to_string(),
    }];
    let batch = schema_service::execute_ddl(&driver, "public", &statements)
        .await
        .expect("drop table execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let tables = schema_service::list_tables(&driver)
        .await
        .expect("list_tables failed");
    assert!(!tables.iter().any(|t| t.name == "ddl_drop_test"));
}

#[test]
fn rename_table_deserializes_camel_case_new_name() {
    let json = r#"{"op":"renameTable","table":"t","newName":"n2"}"#;
    let statement: DdlStatement =
        serde_json::from_str(json).expect("should deserialize camelCase newName");
    match statement {
        DdlStatement::RenameTable { table, new_name } => {
            assert_eq!(table, "t");
            assert_eq!(new_name, "n2");
        }
        _ => panic!("wrong variant"),
    }
}

#[tokio::test]
async fn execute_ddl_add_foreign_key_with_actions_and_read_back() {
    let driver = dev_driver().await;
    raw_exec(&driver, "drop table if exists ddl_fk_action_child;").await;
    raw_exec(&driver, "drop table if exists ddl_fk_action_parent;").await;
    raw_exec(&driver, "create table ddl_fk_action_parent (id serial primary key);").await;
    raw_exec(
        &driver,
        "create table ddl_fk_action_child (id serial primary key, parent_id int);",
    )
    .await;

    let add = vec![DdlStatement::AddConstraint {
        table: "ddl_fk_action_child".to_string(),
        constraint: NewConstraint {
            name: "ddl_fk_action_child_parent_fk".to_string(),
            kind: ConstraintKind::ForeignKey,
            columns: vec!["parent_id".to_string()],
            referenced_table: Some("ddl_fk_action_parent".to_string()),
            referenced_columns: vec!["id".to_string()],
            on_update: Some(ForeignKeyAction::SetNull),
            on_delete: Some(ForeignKeyAction::Cascade),
            check_expression: None,
        },
    }];

    let preview = schema_service::render_ddl(&driver, "public", &add)
        .await
        .expect("render_ddl failed");
    assert!(preview[0].sql.to_lowercase().contains("on update set null"));
    assert!(preview[0].sql.to_lowercase().contains("on delete cascade"));

    let batch = schema_service::execute_ddl(&driver, "public", &add)
        .await
        .expect("add foreign key execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let constraints = schema_service::list_constraints(&driver, "public", "ddl_fk_action_child")
        .await
        .expect("list_constraints failed");
    let fk = constraints
        .iter()
        .find(|c| c.name == "ddl_fk_action_child_parent_fk")
        .expect("expected the new foreign key");
    assert_eq!(fk.on_update, Some(ForeignKeyAction::SetNull));
    assert_eq!(fk.on_delete, Some(ForeignKeyAction::Cascade));

    raw_exec(&driver, "drop table ddl_fk_action_child;").await;
    raw_exec(&driver, "drop table ddl_fk_action_parent;").await;
}

#[tokio::test]
async fn execute_ddl_create_table_with_auto_increment_column() {
    let driver = dev_driver().await;
    raw_exec(&driver, "drop table if exists ddl_autoinc_test;").await;

    let statements = vec![DdlStatement::CreateTable {
        table: "ddl_autoinc_test".to_string(),
        columns: vec![
            NewColumn {
                name: "id".to_string(),
                data_type: "auto-increment".to_string(),
                is_nullable: true,
                default: None,
            },
            NewColumn {
                name: "name".to_string(),
                data_type: "text".to_string(),
                is_nullable: true,
                default: None,
            },
        ],
    }];

    let preview = schema_service::render_ddl(&driver, "public", &statements)
        .await
        .expect("render_ddl failed");
    assert!(preview[0].sql.contains("serial"));

    let batch = schema_service::execute_ddl(&driver, "public", &statements)
        .await
        .expect("create table execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    // Insert twice without specifying id — the auto-increment sequence
    // should assign values on its own.
    raw_exec(&driver, "insert into ddl_autoinc_test (name) values ('a');").await;
    raw_exec(&driver, "insert into ddl_autoinc_test (name) values ('b');").await;

    let columns = schema_service::get_table_columns(&driver, "public", "ddl_autoinc_test")
        .await
        .expect("get_table_columns failed");
    let id_col = columns.iter().find(|c| c.name == "id").expect("expected id column");
    assert!(!id_col.is_nullable, "serial columns are implicitly not null");

    raw_exec(&driver, "drop table ddl_autoinc_test;").await;
}
