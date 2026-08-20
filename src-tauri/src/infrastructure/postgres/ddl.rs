use deadpool_postgres::Client;

use crate::domain::schema::{ColumnEdit, ConstraintKind, DdlBatchResult, DdlExecutionResult, DdlPreview, DdlStatement, NewColumn, NewConstraint, NewIndex};
use crate::error::{describe_pg_error, AppError};

fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

fn render_add_column(schema: &str, table: &str, column: &NewColumn) -> String {
    let nullability = if column.is_nullable { "" } else { " not null" };
    let default = column
        .default
        .as_ref()
        .map(|d| format!(" default {d}"))
        .unwrap_or_default();
    format!(
        "alter table {}.{} add column {} {}{}{};",
        quote_ident(schema),
        quote_ident(table),
        quote_ident(&column.name),
        column.data_type,
        nullability,
        default
    )
}

fn render_drop_column(schema: &str, table: &str, column: &str) -> String {
    format!(
        "alter table {}.{} drop column {};",
        quote_ident(schema),
        quote_ident(table),
        quote_ident(column)
    )
}

/// Postgres has no single clause that changes a column's type,
/// nullability, and default together — each needs its own `ALTER
/// COLUMN ... SET/DROP ...` clause, and `RENAME COLUMN` in particular
/// cannot be combined with any other clause in the same `ALTER TABLE`
/// at all. This renders every needed clause/statement and joins them
/// with `;` into the single SQL string `render_all`/`execute_all`
/// expect one `DdlStatement` to produce.
fn render_alter_column(schema: &str, table: &str, edit: &ColumnEdit) -> String {
    let table_ref = format!("{}.{}", quote_ident(schema), quote_ident(table));
    let mut statements = Vec::new();

    if edit.column.name != edit.current_name {
        statements.push(format!(
            "alter table {table_ref} rename column {} to {};",
            quote_ident(&edit.current_name),
            quote_ident(&edit.column.name)
        ));
    }

    let target_column = quote_ident(&edit.column.name);
    statements.push(format!(
        "alter table {table_ref} alter column {target_column} type {} using {target_column}::{};",
        edit.column.data_type, edit.column.data_type
    ));
    statements.push(format!(
        "alter table {table_ref} alter column {target_column} {};",
        if edit.column.is_nullable { "drop not null" } else { "set not null" }
    ));
    statements.push(match &edit.column.default {
        Some(default) => format!("alter table {table_ref} alter column {target_column} set default {default};"),
        None => format!("alter table {table_ref} alter column {target_column} drop default;"),
    });

    statements.join("\n")
}

fn render_add_index(schema: &str, table: &str, index: &NewIndex) -> String {
    let unique = if index.is_unique { "unique " } else { "" };
    let cols = index.columns.iter().map(|c| quote_ident(c)).collect::<Vec<_>>().join(", ");
    format!(
        "create {unique}index {} on {}.{} ({});",
        quote_ident(&index.name),
        quote_ident(schema),
        quote_ident(table),
        cols
    )
}

fn render_drop_index(schema: &str, index: &str) -> String {
    format!("drop index {}.{};", quote_ident(schema), quote_ident(index))
}

fn render_add_constraint(schema: &str, table: &str, constraint: &NewConstraint) -> Result<String, AppError> {
    let cols = constraint.columns.iter().map(|c| quote_ident(c)).collect::<Vec<_>>().join(", ");
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
            format!("foreign key ({cols}) references {} ({ref_cols})", quote_ident(ref_table))
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
        "alter table {}.{} add constraint {} {};",
        quote_ident(schema),
        quote_ident(table),
        quote_ident(&constraint.name),
        clause
    ))
}

fn render_drop_constraint(schema: &str, table: &str, constraint: &str) -> String {
    format!(
        "alter table {}.{} drop constraint {};",
        quote_ident(schema),
        quote_ident(table),
        quote_ident(constraint)
    )
}

pub fn render(schema: &str, statement: &DdlStatement) -> Result<String, AppError> {
    match statement {
        DdlStatement::AddColumn { table, column } => Ok(render_add_column(schema, table, column)),
        DdlStatement::DropColumn { table, column } => Ok(render_drop_column(schema, table, column)),
        DdlStatement::AlterColumn { table, edit } => Ok(render_alter_column(schema, table, edit)),
        DdlStatement::AddIndex { table, index } => Ok(render_add_index(schema, table, index)),
        DdlStatement::DropIndex { index, .. } => Ok(render_drop_index(schema, index)),
        DdlStatement::AddConstraint { table, constraint } => render_add_constraint(schema, table, constraint),
        DdlStatement::DropConstraint { table, constraint } => Ok(render_drop_constraint(schema, table, constraint)),
    }
}

pub fn render_all(schema: &str, statements: &[DdlStatement]) -> Result<Vec<DdlPreview>, AppError> {
    statements
        .iter()
        .map(|s| render(schema, s).map(|sql| DdlPreview { sql }))
        .collect()
}

pub async fn execute_all(
    client: &mut Client,
    schema: &str,
    statements: &[DdlStatement],
) -> Result<DdlBatchResult, AppError> {
    let txn = client
        .transaction()
        .await
        .map_err(|e| AppError::new(format!("Could not start transaction: {}", describe_pg_error(&e))))?;

    let mut results = Vec::with_capacity(statements.len());
    let mut failed = false;

    for statement in statements {
        if failed {
            break;
        }

        let sql = match render(schema, statement) {
            Ok(sql) => sql,
            Err(e) => {
                results.push(DdlExecutionResult {
                    sql: String::new(),
                    success: false,
                    error: Some(e.to_string()),
                });
                failed = true;
                continue;
            }
        };

        match txn.batch_execute(&sql).await {
            Ok(()) => results.push(DdlExecutionResult { sql, success: true, error: None }),
            Err(e) => {
                results.push(DdlExecutionResult {
                    sql,
                    success: false,
                    error: Some(describe_pg_error(&e)),
                });
                failed = true;
            }
        }
    }

    if failed {
        txn.rollback()
            .await
            .map_err(|e| AppError::new(format!("Rollback failed: {}", describe_pg_error(&e))))?;
    } else {
        txn.commit()
            .await
            .map_err(|e| AppError::new(format!("Commit failed: {}", describe_pg_error(&e))))?;
    }

    Ok(DdlBatchResult { results, rolled_back: failed })
}

