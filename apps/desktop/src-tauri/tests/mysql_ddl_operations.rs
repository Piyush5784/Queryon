use queryon_lib::domain::connection::{ConnectionProfile, Engine, SslMode};
use queryon_lib::domain::query::service as query_service;
use queryon_lib::domain::query::QueryResult;
use queryon_lib::domain::schema::service as schema_service;
use queryon_lib::domain::schema::{
    ColumnEdit, ConstraintKind, DdlStatement, ForeignKeyAction, NewColumn, NewConstraint, NewIndex,
};
use queryon_lib::error::AppError;
use queryon_lib::infrastructure::mysql::driver::MySqlDriver;
use queryon_lib::infrastructure::mysql::pool::build_pool;

async fn exec(driver: &MySqlDriver, sql: &str) -> Result<QueryResult, AppError> {
    query_service::execute_query(driver, sql).await
}

fn dev_profile() -> ConnectionProfile {
    ConnectionProfile {
        id: "test".to_string(),
        name: "test".to_string(),
        engine: Engine::MySql,
        host: "localhost".to_string(),
        port: 33066,
        database: "devdb".to_string(),
        user: "devuser".to_string(),
        password: "devpass".to_string(),
        ssl_mode: SslMode::Disable,
        read_only: false,
        ssh_tunnel: None,
    }
}

async fn dev_driver() -> MySqlDriver {
    let pool = build_pool(&dev_profile())
        .await
        .expect("failed to build pool");
    MySqlDriver::new(pool, "devdb".to_string())
}

/// Runs setup/teardown SQL (CREATE/DROP TABLE) directly through the
/// existing raw-query path — every DDL test creates its own scratch
/// table and drops it at the end, never touching the seeded
/// `users`/`orders`/etc. tables other suites rely on.
async fn raw_exec(driver: &MySqlDriver, sql: &str) {
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
            data_type: "varchar(255)".to_string(),
            is_nullable: false,
            default: None,
        },
    }];

    let previews = schema_service::render_ddl(&driver, "devdb", &statements)
        .await
        .expect("render_ddl failed");

    assert_eq!(previews.len(), 1);
    assert!(previews[0].sql.contains("add column"));
    assert!(previews[0].sql.contains("`email`"));
    assert!(previews[0].sql.contains("not null"));
}

#[tokio::test]
async fn execute_ddl_add_and_drop_column_round_trips() {
    let driver = dev_driver().await;
    raw_exec(&driver, "drop table if exists ddl_col_test;").await;
    raw_exec(
        &driver,
        "create table ddl_col_test (id int primary key auto_increment);",
    )
    .await;

    let add = vec![DdlStatement::AddColumn {
        table: "ddl_col_test".to_string(),
        column: NewColumn {
            name: "email".to_string(),
            data_type: "varchar(255)".to_string(),
            is_nullable: true,
            default: None,
        },
    }];
    let batch = schema_service::execute_ddl(&driver, "devdb", &add)
        .await
        .expect("add column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = schema_service::get_table_columns(&driver, "devdb", "ddl_col_test")
        .await
        .expect("get_table_columns failed");
    assert!(columns.iter().any(|c| c.name == "email"));

    let drop = vec![DdlStatement::DropColumn {
        table: "ddl_col_test".to_string(),
        column: "email".to_string(),
    }];
    let batch = schema_service::execute_ddl(&driver, "devdb", &drop)
        .await
        .expect("drop column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = schema_service::get_table_columns(&driver, "devdb", "ddl_col_test")
        .await
        .expect("get_table_columns failed");
    assert!(!columns.iter().any(|c| c.name == "email"));

    raw_exec(&driver, "drop table ddl_col_test;").await;
}

/// MySQL DDL auto-commits per statement — a batch can never be rolled
/// back the way Postgres's can. This asserts that documented limitation
/// directly: the first statement's effect survives the second one's
/// failure, and `rolled_back` is always false.
#[tokio::test]
async fn execute_ddl_batch_stops_at_failure_without_rollback() {
    let driver = dev_driver().await;
    raw_exec(&driver, "drop table if exists ddl_rollback_test;").await;
    raw_exec(
        &driver,
        "create table ddl_rollback_test (id int primary key auto_increment);",
    )
    .await;

    let statements = vec![
        DdlStatement::AddColumn {
            table: "ddl_rollback_test".to_string(),
            column: NewColumn {
                name: "email".to_string(),
                data_type: "varchar(255)".to_string(),
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

    let batch = schema_service::execute_ddl(&driver, "devdb", &statements)
        .await
        .expect("execute_ddl should not itself error");

    assert!(
        !batch.rolled_back,
        "MySQL DDL can never report rolled_back: true"
    );
    assert_eq!(
        batch.results.len(),
        2,
        "execution should stop after the failing statement"
    );
    assert!(
        batch.results[0].success,
        "first statement should have run before the failure"
    );
    assert!(
        !batch.results[1].success,
        "second statement should have failed"
    );

    let columns = schema_service::get_table_columns(&driver, "devdb", "ddl_rollback_test")
        .await
        .expect("get_table_columns failed");
    assert!(
        columns.iter().any(|c| c.name == "email"),
        "MySQL cannot undo the earlier successful ADD COLUMN — it must still be applied"
    );

    raw_exec(&driver, "drop table ddl_rollback_test;").await;
}

#[tokio::test]
async fn execute_ddl_alter_column_renames_and_changes_type() {
    let driver = dev_driver().await;
    raw_exec(&driver, "drop table if exists ddl_alter_test;").await;
    raw_exec(
        &driver,
        "create table ddl_alter_test (id int primary key auto_increment, age varchar(255));",
    )
    .await;

    let statements = vec![DdlStatement::AlterColumn {
        table: "ddl_alter_test".to_string(),
        edit: ColumnEdit {
            current_name: "age".to_string(),
            column: NewColumn {
                name: "years_old".to_string(),
                data_type: "int".to_string(),
                is_nullable: false,
                default: Some("0".to_string()),
            },
        },
    }];

    let batch = schema_service::execute_ddl(&driver, "devdb", &statements)
        .await
        .expect("alter column execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let columns = schema_service::get_table_columns(&driver, "devdb", "ddl_alter_test")
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
        "create table ddl_index_test (id int primary key auto_increment, email varchar(255));",
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
    let batch = schema_service::execute_ddl(&driver, "devdb", &add)
        .await
        .expect("add index execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let indexes = schema_service::list_indexes(&driver, "devdb", "ddl_index_test")
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
    let batch = schema_service::execute_ddl(&driver, "devdb", &drop)
        .await
        .expect("drop index execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let indexes = schema_service::list_indexes(&driver, "devdb", "ddl_index_test")
        .await
        .expect("list_indexes failed");
    assert!(!indexes.iter().any(|i| i.name == "ddl_index_test_email_idx"));

    raw_exec(&driver, "drop table ddl_index_test;").await;
}

/// MySQL's DROP CONSTRAINT needs the constraint's kind resolved first
/// (FOREIGN KEY / CHECK / index-backed) to pick the right DROP syntax —
/// this exercises that resolution end to end via a foreign key.
#[tokio::test]
async fn execute_ddl_add_foreign_key_and_drop() {
    let driver = dev_driver().await;
    raw_exec(&driver, "drop table if exists ddl_fk_child;").await;
    raw_exec(&driver, "drop table if exists ddl_fk_parent;").await;
    raw_exec(
        &driver,
        "create table ddl_fk_parent (id int primary key auto_increment);",
    )
    .await;
    raw_exec(
        &driver,
        "create table ddl_fk_child (id int primary key auto_increment, parent_id int);",
    )
    .await;

    let add = vec![DdlStatement::AddConstraint {
        table: "ddl_fk_child".to_string(),
        constraint: NewConstraint {
            name: "ddl_fk_child_parent_fk".to_string(),
            kind: ConstraintKind::ForeignKey,
            columns: vec!["parent_id".to_string()],
            referenced_table: Some("ddl_fk_parent".to_string()),
            referenced_columns: vec!["id".to_string()],
            on_update: None,
            on_delete: None,
            check_expression: None,
        },
    }];
    let batch = schema_service::execute_ddl(&driver, "devdb", &add)
        .await
        .expect("add foreign key execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let constraints = schema_service::list_constraints(&driver, "devdb", "ddl_fk_child")
        .await
        .expect("list_constraints failed");
    assert!(constraints
        .iter()
        .any(|c| c.name == "ddl_fk_child_parent_fk"));

    let drop = vec![DdlStatement::DropConstraint {
        table: "ddl_fk_child".to_string(),
        constraint: "ddl_fk_child_parent_fk".to_string(),
    }];
    let batch = schema_service::execute_ddl(&driver, "devdb", &drop)
        .await
        .expect("drop foreign key execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let constraints = schema_service::list_constraints(&driver, "devdb", "ddl_fk_child")
        .await
        .expect("list_constraints failed");
    assert!(!constraints
        .iter()
        .any(|c| c.name == "ddl_fk_child_parent_fk"));

    raw_exec(&driver, "drop table ddl_fk_child;").await;
    raw_exec(&driver, "drop table ddl_fk_parent;").await;
}

#[tokio::test]
async fn execute_ddl_create_table_with_index_and_constraint() {
    let driver = dev_driver().await;
    raw_exec(&driver, "drop table if exists ddl_create_test;").await;

    let statements = vec![
        DdlStatement::CreateTable {
            table: "ddl_create_test".to_string(),
            columns: vec![
                // MySQL requires an auto_increment column to be a key
                // at CREATE TABLE time — it cannot be added as a
                // separate ALTER TABLE ADD CONSTRAINT afterward the way
                // Postgres's `serial` can. Since NewColumn has no way to
                // declare "primary key" inline, this test uses a plain
                // int and adds the primary key as its own statement,
                // which is a shape MySQL does allow after the fact.
                NewColumn {
                    name: "id".to_string(),
                    data_type: "int".to_string(),
                    is_nullable: false,
                    default: None,
                },
                NewColumn {
                    name: "email".to_string(),
                    data_type: "varchar(255)".to_string(),
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

    let batch = schema_service::execute_ddl(&driver, "devdb", &statements)
        .await
        .expect("create table execute_ddl failed");
    for result in &batch.results {
        assert!(result.success, "{:?}", result.error);
    }

    let columns = schema_service::get_table_columns(&driver, "devdb", "ddl_create_test")
        .await
        .expect("get_table_columns failed");
    assert!(columns.iter().any(|c| c.name == "id"));
    assert!(columns.iter().any(|c| c.name == "email"));

    let constraints = schema_service::list_constraints(&driver, "devdb", "ddl_create_test")
        .await
        .expect("list_constraints failed");
    // MySQL always names primary key constraints "PRIMARY" regardless of
    // the name requested in ADD CONSTRAINT ... PRIMARY KEY — confirmed
    // directly against a live container, not a bug in our code.
    assert!(constraints
        .iter()
        .any(|c| c.kind == ConstraintKind::PrimaryKey && c.name == "PRIMARY"));

    let indexes = schema_service::list_indexes(&driver, "devdb", "ddl_create_test")
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
        "create table ddl_rename_test (id int primary key auto_increment);",
    )
    .await;

    let statements = vec![DdlStatement::RenameTable {
        table: "ddl_rename_test".to_string(),
        new_name: "ddl_rename_test_2".to_string(),
    }];
    let batch = schema_service::execute_ddl(&driver, "devdb", &statements)
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
        "create table ddl_drop_test (id int primary key auto_increment);",
    )
    .await;

    let statements = vec![DdlStatement::DropTable {
        table: "ddl_drop_test".to_string(),
    }];
    let batch = schema_service::execute_ddl(&driver, "devdb", &statements)
        .await
        .expect("drop table execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let tables = schema_service::list_tables(&driver)
        .await
        .expect("list_tables failed");
    assert!(!tables.iter().any(|t| t.name == "ddl_drop_test"));
}

#[tokio::test]
async fn execute_ddl_add_foreign_key_with_actions_and_read_back() {
    let driver = dev_driver().await;
    raw_exec(&driver, "drop table if exists ddl_fk_action_child;").await;
    raw_exec(&driver, "drop table if exists ddl_fk_action_parent;").await;
    raw_exec(&driver, "create table ddl_fk_action_parent (id int primary key auto_increment);").await;
    raw_exec(
        &driver,
        "create table ddl_fk_action_child (id int primary key auto_increment, parent_id int);",
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

    let preview = schema_service::render_ddl(&driver, "devdb", &add)
        .await
        .expect("render_ddl failed");
    assert!(preview[0].sql.to_lowercase().contains("on update set null"));
    assert!(preview[0].sql.to_lowercase().contains("on delete cascade"));

    let batch = schema_service::execute_ddl(&driver, "devdb", &add)
        .await
        .expect("add foreign key execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    let constraints = schema_service::list_constraints(&driver, "devdb", "ddl_fk_action_child")
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
                data_type: "varchar(255)".to_string(),
                is_nullable: true,
                default: None,
            },
        ],
    }];

    let preview = schema_service::render_ddl(&driver, "devdb", &statements)
        .await
        .expect("render_ddl failed");
    assert!(preview[0].sql.contains("auto_increment"));
    assert!(preview[0].sql.contains("primary key"));

    let batch = schema_service::execute_ddl(&driver, "devdb", &statements)
        .await
        .expect("create table execute_ddl failed");
    assert!(batch.results[0].success, "{:?}", batch.results[0].error);

    raw_exec(&driver, "insert into ddl_autoinc_test (name) values ('a');").await;
    raw_exec(&driver, "insert into ddl_autoinc_test (name) values ('b');").await;

    let columns = schema_service::get_table_columns(&driver, "devdb", "ddl_autoinc_test")
        .await
        .expect("get_table_columns failed");
    let id_col = columns.iter().find(|c| c.name == "id").expect("expected id column");
    assert!(id_col.is_primary_key);
    assert!(!id_col.is_nullable);

    raw_exec(&driver, "drop table ddl_autoinc_test;").await;
}
