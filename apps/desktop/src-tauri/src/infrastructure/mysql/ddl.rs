use sqlx::mysql::MySqlPool;
use sqlx::{AssertSqlSafe, Row};

use crate::domain::schema::{
    ColumnEdit, ConstraintKind, DdlBatchResult, DdlExecutionResult, DdlPreview, DdlStatement,
    ForeignKeyAction, NewColumn, NewConstraint, NewIndex, AUTO_INCREMENT_TYPE,
};
use crate::error::AppError;

fn quote_ident(ident: &str) -> String {
    format!("`{}`", ident.replace('`', "``"))
}

fn clean_mysql_error(err: &sqlx::Error) -> String {
    crate::error::describe_mysql_error(err)
}

fn starrocks_default(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with('\'') || trimmed.contains('(') {
        return value.to_string();
    }
    format!("'{}'", trimmed.replace('\'', "''"))
}

fn render_column_def(column: &NewColumn, is_starrocks: bool) -> String {
    if column.data_type == AUTO_INCREMENT_TYPE {
        return if is_starrocks {
            format!("{} bigint not null auto_increment", quote_ident(&column.name))
        } else {
            format!("{} int not null auto_increment primary key", quote_ident(&column.name))
        };
    }
    let nullability = if column.is_nullable { "" } else { " not null" };
    let default = column
        .default
        .as_ref()
        .map(|d| {
            let quoted = if is_starrocks { starrocks_default(d) } else { d.clone() };
            format!(" default {quoted}")
        })
        .unwrap_or_default();
    format!(
        "{} {}{}{}",
        quote_ident(&column.name),
        column.data_type,
        nullability,
        default
    )
}

fn render_create_table(
    _schema: &str,
    table: &str,
    columns: &[NewColumn],
    is_starrocks: bool,
) -> Result<String, AppError> {
    if columns.is_empty() {
        return Err(AppError::new("A table needs at least one column."));
    }
    let column_defs = columns
        .iter()
        .map(|c| render_column_def(c, is_starrocks))
        .collect::<Vec<_>>()
        .join(", ");

    if !is_starrocks {
        return Ok(format!("create table {} ({});", quote_ident(table), column_defs));
    }

    let pk_column = columns
        .iter()
        .find(|c| c.data_type == AUTO_INCREMENT_TYPE)
        .ok_or_else(|| {
            AppError::new(
                "StarRocks requires a primary key column — add an auto-increment column to this table.",
            )
        })?;

    Ok(format!(
        "create table {} ({}) primary key ({}) distributed by hash ({});",
        quote_ident(table),
        column_defs,
        quote_ident(&pk_column.name),
        quote_ident(&pk_column.name)
    ))
}

fn render_rename_table(_schema: &str, table: &str, new_name: &str) -> String {
    format!(
        "rename table {} to {};",
        quote_ident(table),
        quote_ident(new_name)
    )
}

fn render_drop_table(_schema: &str, table: &str) -> String {
    format!("drop table {};", quote_ident(table))
}

fn render_add_column(_schema: &str, table: &str, column: &NewColumn, is_starrocks: bool) -> String {
    format!(
        "alter table {} add column {};",
        quote_ident(table),
        render_column_def(column, is_starrocks)
    )
}

fn render_drop_column(_schema: &str, table: &str, column: &str) -> String {
    format!(
        "alter table {} drop column {};",
        quote_ident(table),
        quote_ident(column)
    )
}

fn render_alter_column(_schema: &str, table: &str, edit: &ColumnEdit, is_starrocks: bool) -> Vec<String> {
    let nullability = if edit.column.is_nullable { "" } else { " not null" };
    let default = edit
        .column
        .default
        .as_ref()
        .map(|d| {
            let quoted = if is_starrocks { starrocks_default(d) } else { d.clone() };
            format!(" default {quoted}")
        })
        .unwrap_or_default();

    if is_starrocks {
        let mut statements = vec![format!(
            "alter table {} modify column {} {}{}{};",
            quote_ident(table),
            quote_ident(&edit.current_name),
            edit.column.data_type,
            nullability,
            default
        )];
        if edit.column.name != edit.current_name {
            statements.push(format!(
                "alter table {} rename column {} to {};",
                quote_ident(table),
                quote_ident(&edit.current_name),
                quote_ident(&edit.column.name)
            ));
        }
        return statements;
    }

    vec![format!(
        "alter table {} change column {} {} {}{}{};",
        quote_ident(table),
        quote_ident(&edit.current_name),
        quote_ident(&edit.column.name),
        edit.column.data_type,
        nullability,
        default
    )]
}

/// MySQL indexes are created/dropped via `ALTER TABLE ... ADD/DROP
/// INDEX`, unlike Postgres's standalone `CREATE INDEX`/`DROP INDEX` —
/// `DROP INDEX` in particular needs the owning table name, which
/// Postgres's `drop index schema.name` does not.
fn render_add_index(_schema: &str, table: &str, index: &NewIndex) -> String {
    let unique = if index.is_unique { "unique " } else { "" };
    let cols = index
        .columns
        .iter()
        .map(|c| quote_ident(c))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "alter table {} add {unique}index {} ({});",
        quote_ident(table),
        quote_ident(&index.name),
        cols
    )
}

fn render_drop_index(_schema: &str, table: &str, index: &str) -> String {
    format!(
        "alter table {} drop index {};",
        quote_ident(table),
        quote_ident(index)
    )
}

fn render_foreign_key_action(action: ForeignKeyAction) -> &'static str {
    match action {
        ForeignKeyAction::NoAction => "no action",
        ForeignKeyAction::Restrict => "restrict",
        ForeignKeyAction::Cascade => "cascade",
        ForeignKeyAction::SetNull => "set null",
        ForeignKeyAction::SetDefault => "set default",
    }
}

fn render_add_constraint(
    _schema: &str,
    table: &str,
    constraint: &NewConstraint,
    is_starrocks: bool,
) -> Result<String, AppError> {
    if is_starrocks {
        return Err(AppError::new(match constraint.kind {
            ConstraintKind::PrimaryKey | ConstraintKind::Unique => {
                "StarRocks has no ALTER TABLE ADD CONSTRAINT — PRIMARY KEY/UNIQUE KEY can only be set when the table is created."
            }
            ConstraintKind::ForeignKey => "StarRocks does not support foreign key constraints.",
            ConstraintKind::Check => "StarRocks does not support check constraints.",
        }));
    }

    let cols = constraint
        .columns
        .iter()
        .map(|c| quote_ident(c))
        .collect::<Vec<_>>()
        .join(", ");
    let clause = match constraint.kind {
        ConstraintKind::PrimaryKey => format!("primary key ({cols})"),
        ConstraintKind::Unique => format!("unique ({cols})"),
        ConstraintKind::ForeignKey => {
            let ref_table = constraint
                .referenced_table
                .as_deref()
                .ok_or_else(|| AppError::new("A foreign key needs a referenced table."))?;
            let ref_cols = constraint
                .referenced_columns
                .iter()
                .map(|c| quote_ident(c))
                .collect::<Vec<_>>()
                .join(", ");
            let on_update = constraint
                .on_update
                .map(|a| format!(" on update {}", render_foreign_key_action(a)))
                .unwrap_or_default();
            let on_delete = constraint
                .on_delete
                .map(|a| format!(" on delete {}", render_foreign_key_action(a)))
                .unwrap_or_default();
            format!(
                "foreign key ({cols}) references {} ({ref_cols}){on_update}{on_delete}",
                quote_ident(ref_table)
            )
        }
        ConstraintKind::Check => {
            let expr = constraint
                .check_expression
                .as_deref()
                .ok_or_else(|| AppError::new("A check constraint needs an expression."))?;
            format!("check ({expr})")
        }
    };
    Ok(format!(
        "alter table {} add constraint {} {};",
        quote_ident(table),
        quote_ident(&constraint.name),
        clause
    ))
}

fn render_drop_constraint(
    _schema: &str,
    table: &str,
    constraint: &DdlDropConstraintKindHint,
) -> String {
    match constraint.kind {
        ConstraintKind::ForeignKey => {
            format!(
                "alter table {} drop foreign key {};",
                quote_ident(table),
                quote_ident(&constraint.name)
            )
        }
        ConstraintKind::Check => {
            format!(
                "alter table {} drop check {};",
                quote_ident(table),
                quote_ident(&constraint.name)
            )
        }
        _ => {
            format!(
                "alter table {} drop index {};",
                quote_ident(table),
                quote_ident(&constraint.name)
            )
        }
    }
}

pub struct DdlDropConstraintKindHint {
    pub name: String,
    pub kind: ConstraintKind,
}

pub fn render(
    schema: &str,
    statement: &DdlStatement,
    drop_constraint_kind: Option<&DdlDropConstraintKindHint>,
    is_starrocks: bool,
) -> Result<Vec<String>, AppError> {
    match statement {
        DdlStatement::CreateTable { table, columns } => {
            Ok(vec![render_create_table(schema, table, columns, is_starrocks)?])
        }
        DdlStatement::RenameTable { table, new_name } => {
            Ok(vec![render_rename_table(schema, table, new_name)])
        }
        DdlStatement::DropTable { table } => Ok(vec![render_drop_table(schema, table)]),
        DdlStatement::AddColumn { table, column } => {
            Ok(vec![render_add_column(schema, table, column, is_starrocks)])
        }
        DdlStatement::DropColumn { table, column } => Ok(vec![render_drop_column(schema, table, column)]),
        DdlStatement::AlterColumn { table, edit } => Ok(render_alter_column(schema, table, edit, is_starrocks)),
        DdlStatement::AddIndex { table, index } => Ok(vec![render_add_index(schema, table, index)]),
        DdlStatement::DropIndex { table, index } => Ok(vec![render_drop_index(schema, table, index)]),
        DdlStatement::AddConstraint { table, constraint } => {
            Ok(vec![render_add_constraint(schema, table, constraint, is_starrocks)?])
        }
        DdlStatement::DropConstraint { table, constraint } => {
            let hint = drop_constraint_kind.ok_or_else(|| {
                AppError::new(format!(
                    "Could not determine constraint '{constraint}' kind — it may no longer exist."
                ))
            })?;
            Ok(vec![render_drop_constraint(schema, table, hint)])
        }
    }
}

pub async fn render_all(
    pool: &MySqlPool,
    schema: &str,
    statements: &[DdlStatement],
    is_starrocks: bool,
) -> Result<Vec<DdlPreview>, AppError> {
    let mut results = Vec::with_capacity(statements.len());
    for statement in statements {
        let hint = resolve_drop_constraint_hint(pool, schema, statement, is_starrocks).await?;
        let sqls = render(schema, statement, hint.as_ref(), is_starrocks)?;
        for sql in sqls {
            results.push(DdlPreview { sql });
        }
    }
    Ok(results)
}

pub async fn execute_all(
    pool: &MySqlPool,
    schema: &str,
    statements: &[DdlStatement],
    is_starrocks: bool,
) -> Result<DdlBatchResult, AppError> {
    let mut results = Vec::with_capacity(statements.len());
    for statement in statements {
        let hint = match resolve_drop_constraint_hint(pool, schema, statement, is_starrocks).await {
            Ok(hint) => hint,
            Err(e) => {
                results.push(DdlExecutionResult {
                    sql: String::new(),
                    success: false,
                    error: Some(e.to_string()),
                });
                return Ok(DdlBatchResult { results, rolled_back: false });
            }
        };
        let sqls = match render(schema, statement, hint.as_ref(), is_starrocks) {
            Ok(sqls) => sqls,
            Err(e) => {
                results.push(DdlExecutionResult {
                    sql: String::new(),
                    success: false,
                    error: Some(e.to_string()),
                });
                return Ok(DdlBatchResult { results, rolled_back: false });
            }
        };

        let table_name = ddl_statement_table(statement);

        for sql in sqls {
            match sqlx::raw_sql(AssertSqlSafe(sql.clone())).execute(pool).await {
                Ok(_) => {
                    results.push(DdlExecutionResult { sql, success: true, error: None });
                    if is_starrocks {
                        if let Some(table) = table_name {
                            if let Err(e) = wait_for_schema_change(pool, schema, table).await {
                                results.push(DdlExecutionResult {
                                    sql: String::new(),
                                    success: false,
                                    error: Some(e.to_string()),
                                });
                                return Ok(DdlBatchResult { results, rolled_back: false });
                            }
                        }
                    }
                }
                Err(e) => {
                    results.push(DdlExecutionResult {
                        sql,
                        success: false,
                        error: Some(clean_mysql_error(&e)),
                    });
                    return Ok(DdlBatchResult { results, rolled_back: false });
                }
            }
        }
    }
    Ok(DdlBatchResult {
        results,
        rolled_back: false,
    })
}

fn ddl_statement_table(statement: &DdlStatement) -> Option<&str> {
    match statement {
        DdlStatement::CreateTable { table, .. }
        | DdlStatement::RenameTable { table, .. }
        | DdlStatement::DropTable { table }
        | DdlStatement::AddColumn { table, .. }
        | DdlStatement::DropColumn { table, .. }
        | DdlStatement::AlterColumn { table, .. }
        | DdlStatement::AddIndex { table, .. }
        | DdlStatement::DropIndex { table, .. }
        | DdlStatement::AddConstraint { table, .. }
        | DdlStatement::DropConstraint { table, .. } => Some(table),
    }
}

async fn wait_for_schema_change(pool: &MySqlPool, database: &str, table: &str) -> Result<(), AppError> {
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    for _ in 0..30 {
        let row = sqlx::raw_sql(AssertSqlSafe(format!(
            "show alter table column from `{}` where TableName = '{}' order by CreateTime desc limit 1",
            database.replace('`', "``"),
            table.replace('\'', "''")
        )))
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to check schema change status: {e}")))?;

        let Some(row) = row else { return Ok(()) };
        let state: String = row.try_get("State").unwrap_or_default();
        if state == "FINISHED" || state == "CANCELLED" {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    Err(AppError::new(format!(
        "Timed out waiting for a schema change on '{table}' to finish."
    )))
}

async fn resolve_drop_constraint_hint(
    pool: &MySqlPool,
    schema: &str,
    statement: &DdlStatement,
    is_starrocks: bool,
) -> Result<Option<DdlDropConstraintKindHint>, AppError> {
    let DdlStatement::DropConstraint { table, constraint } = statement else {
        return Ok(None);
    };
    let constraints = super::metadata::list_constraints(pool, schema, table, is_starrocks).await?;
    let matching = constraints.into_iter().find(|c| &c.name == constraint);
    Ok(matching.map(|c| DdlDropConstraintKindHint {
        name: c.name,
        kind: c.kind,
    }))
}
