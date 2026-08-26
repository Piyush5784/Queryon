use trino_rust_client::client::Client;

use crate::domain::schema::{DdlBatchResult, DdlExecutionResult, DdlPreview, DdlStatement, NewColumn};
use crate::error::AppError;

use super::metadata::quote_ident;

fn qualified_table(schema: &str, table: &str) -> String {
    format!("{}.{}", quote_ident(schema), quote_ident(table))
}

fn render_column_def(column: &NewColumn) -> String {
    format!("{} {}", quote_ident(&column.name), column.data_type)
}

fn render_create_table(schema: &str, table: &str, columns: &[NewColumn]) -> Result<String, AppError> {
    if columns.is_empty() {
        return Err(AppError::new("A table needs at least one column."));
    }
    let column_defs = columns.iter().map(render_column_def).collect::<Vec<_>>().join(", ");
    Ok(format!("create table {} ({})", qualified_table(schema, table), column_defs))
}

fn render_rename_table(schema: &str, table: &str, new_name: &str) -> String {
    format!("alter table {} rename to {}", qualified_table(schema, table), qualified_table(schema, new_name))
}

fn render_drop_table(schema: &str, table: &str) -> String {
    format!("drop table {}", qualified_table(schema, table))
}

fn render_add_column(schema: &str, table: &str, column: &NewColumn) -> String {
    format!("alter table {} add column {}", qualified_table(schema, table), render_column_def(column))
}

fn unsupported(operation: &str) -> AppError {
    AppError::new(format!(
        "Trino does not support {operation} — its catalogs are federated connectors, not a traditional RDBMS with schema-editing this granular. Recreate the table instead."
    ))
}

fn render_one(schema: &str, statement: &DdlStatement) -> Result<String, AppError> {
    match statement {
        DdlStatement::CreateTable { table, columns } => render_create_table(schema, table, columns),
        DdlStatement::RenameTable { table, new_name } => Ok(render_rename_table(schema, table, new_name)),
        DdlStatement::DropTable { table } => Ok(render_drop_table(schema, table)),
        DdlStatement::AddColumn { table, column } => Ok(render_add_column(schema, table, column)),
        DdlStatement::DropColumn { .. } => Err(unsupported("dropping a column")),
        DdlStatement::AlterColumn { .. } => Err(unsupported("altering a column's type, name, or nullability")),
        DdlStatement::AddIndex { .. } => Err(unsupported("indexes — Trino connectors have no index concept")),
        DdlStatement::DropIndex { .. } => Err(unsupported("indexes — Trino connectors have no index concept")),
        DdlStatement::AddConstraint { .. } => Err(unsupported("constraints — Trino connectors have no primary key, foreign key, unique, or check constraint concept")),
        DdlStatement::DropConstraint { .. } => Err(unsupported("constraints — Trino connectors have no primary key, foreign key, unique, or check constraint concept")),
    }
}

pub async fn render_all(schema: &str, statements: &[DdlStatement]) -> Result<Vec<DdlPreview>, AppError> {
    let mut results = Vec::with_capacity(statements.len());
    for statement in statements {
        let sql = render_one(schema, statement)?;
        results.push(DdlPreview { sql });
    }
    Ok(results)
}

pub async fn execute_all(client: &Client, schema: &str, statements: &[DdlStatement]) -> Result<DdlBatchResult, AppError> {
    let mut results = Vec::with_capacity(statements.len());

    for statement in statements {
        let sql = match render_one(schema, statement) {
            Ok(sql) => sql,
            Err(e) => {
                results.push(DdlExecutionResult { sql: String::new(), success: false, error: Some(e.to_string()) });
                break;
            }
        };

        match client.execute(&sql).await {
            Ok(_) => results.push(DdlExecutionResult { sql, success: true, error: None }),
            Err(e) => {
                results.push(DdlExecutionResult { sql, success: false, error: Some(crate::error::describe_trino_error(&e)) });
                break;
            }
        }
    }

    Ok(DdlBatchResult { results, rolled_back: false })
}
