use crate::domain::schema::{ColumnEdit, ConstraintKind, DdlPreview, DdlStatement, NewColumn, NewConstraint, AUTO_INCREMENT_TYPE};
use crate::error::AppError;

use super::pool::ClickHouseClient;

fn render_data_type(data_type: &str) -> String {
    if data_type == AUTO_INCREMENT_TYPE {
        "UInt64".to_string()
    } else {
        data_type.to_string()
    }
}

fn render_column_def(column: &NewColumn) -> String {
    let mut type_name = render_data_type(&column.data_type);
    if column.is_nullable && !type_name.starts_with("Nullable(") {
        type_name = format!("Nullable({type_name})");
    }

    let mut def = format!("`{}` {}", column.name, type_name);
    if let Some(default) = &column.default {
        def.push_str(&format!(" DEFAULT {default}"));
    }
    def
}

fn render_create_table(table: &str, columns: &[NewColumn]) -> String {
    let column_defs: Vec<String> = columns.iter().map(render_column_def).collect();
    let order_by = columns.first().map(|c| format!("`{}`", c.name)).unwrap_or_else(|| "tuple()".to_string());

    format!(
        "create table `{table}` (\n  {}\n) engine = MergeTree() order by {order_by}",
        column_defs.join(",\n  ")
    )
}

fn render_alter_column(table: &str, edit: &ColumnEdit) -> Vec<String> {
    let mut retyped_column = edit.column.clone();
    retyped_column.name = edit.current_name.clone();

    let mut statements = vec![format!(
        "alter table `{table}` modify column {}",
        render_column_def(&retyped_column)
    )];

    if edit.current_name != edit.column.name {
        statements.push(format!(
            "alter table `{table}` rename column `{}` to `{}`",
            edit.current_name, edit.column.name
        ));
    }
    statements
}

fn render_add_constraint(table: &str, constraint: &NewConstraint) -> Result<String, AppError> {
    match constraint.kind {
        ConstraintKind::Check => {
            let expr = constraint
                .check_expression
                .as_deref()
                .ok_or_else(|| AppError::new("A CHECK constraint needs an expression."))?;
            Ok(format!(
                "alter table `{table}` add constraint `{}` check {expr}",
                constraint.name
            ))
        }
        ConstraintKind::PrimaryKey => Err(AppError::new(
            "ClickHouse's primary key is set by the table's ORDER BY/ENGINE clause at creation time — it cannot be added with ALTER TABLE.",
        )),
        ConstraintKind::Unique => Err(AppError::new(
            "ClickHouse has no UNIQUE constraint — it is not enforced by any table engine here.",
        )),
        ConstraintKind::ForeignKey => Err(AppError::new(
            "ClickHouse does not enforce foreign key constraints — ALTER TABLE ADD CONSTRAINT ... FOREIGN KEY parses but is silently dropped, never stored or checked.",
        )),
    }
}

pub fn render(statement: &DdlStatement) -> Result<Vec<String>, AppError> {
    match statement {
        DdlStatement::CreateTable { table, columns } => Ok(vec![render_create_table(table, columns)]),
        DdlStatement::RenameTable { table, new_name } => {
            Ok(vec![format!("rename table `{table}` to `{new_name}`")])
        }
        DdlStatement::DropTable { table } => Ok(vec![format!("drop table `{table}`")]),
        DdlStatement::AddColumn { table, column } => {
            Ok(vec![format!("alter table `{table}` add column {}", render_column_def(column))])
        }
        DdlStatement::DropColumn { table, column } => {
            Ok(vec![format!("alter table `{table}` drop column `{column}`")])
        }
        DdlStatement::AlterColumn { table, edit } => Ok(render_alter_column(table, edit)),
        DdlStatement::AddIndex { table, index } => {
            let expr = index.columns.iter().map(|c| format!("`{c}`")).collect::<Vec<_>>().join(", ");
            Ok(vec![format!(
                "alter table `{table}` add index `{}` ({expr}) type minmax granularity 4",
                index.name
            )])
        }
        DdlStatement::DropIndex { table, index } => {
            Ok(vec![format!("alter table `{table}` drop index `{index}`")])
        }
        DdlStatement::AddConstraint { table, constraint } => Ok(vec![render_add_constraint(table, constraint)?]),
        DdlStatement::DropConstraint { table, constraint } => {
            Ok(vec![format!("alter table `{table}` drop constraint `{constraint}`")])
        }
    }
}

pub async fn render_all(statements: &[DdlStatement]) -> Result<Vec<DdlPreview>, AppError> {
    let mut previews = Vec::with_capacity(statements.len());
    for statement in statements {
        for sql in render(statement)? {
            previews.push(DdlPreview { sql });
        }
    }
    Ok(previews)
}

pub async fn execute_all(
    client: &ClickHouseClient,
    statements: &[DdlStatement],
) -> Result<crate::domain::schema::DdlBatchResult, AppError> {
    let mut results = Vec::new();

    for statement in statements {
        let sub_statements = match render(statement) {
            Ok(stmts) => stmts,
            Err(e) => {
                results.push(crate::domain::schema::DdlExecutionResult {
                    sql: String::new(),
                    success: false,
                    error: Some(e.to_string()),
                });
                break;
            }
        };

        let mut failed = false;
        for sql in sub_statements {
            match client.execute(&sql).await {
                Ok(()) => {
                    results.push(crate::domain::schema::DdlExecutionResult {
                        sql,
                        success: true,
                        error: None,
                    });
                }
                Err(e) => {
                    results.push(crate::domain::schema::DdlExecutionResult {
                        sql,
                        success: false,
                        error: Some(e.to_string()),
                    });
                    failed = true;
                    break;
                }
            }
        }
        if failed {
            break;
        }
    }

    Ok(crate::domain::schema::DdlBatchResult { results, rolled_back: false })
}
