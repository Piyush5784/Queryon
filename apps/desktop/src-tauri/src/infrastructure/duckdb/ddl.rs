use duckdb::Connection;

use crate::domain::schema::{
    ColumnEdit, DdlBatchResult, DdlExecutionResult, DdlPreview, DdlStatement, NewColumn,
    NewConstraint, AUTO_INCREMENT_TYPE,
};
use crate::error::AppError;

fn clean(e: duckdb::Error) -> AppError {
    AppError::new(e.to_string())
}

fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

fn render_column_def(column: &NewColumn, sequence_name: Option<&str>) -> String {
    let data_type = if column.data_type == AUTO_INCREMENT_TYPE { "BIGINT" } else { &column.data_type };
    let mut def = format!("{} {}", quote_ident(&column.name), data_type);
    if !column.is_nullable {
        def.push_str(" not null");
    }
    if let Some(seq) = sequence_name {
        def.push_str(&format!(" default nextval('{seq}')"));
    } else if let Some(default) = &column.default {
        def.push_str(&format!(" default {default}"));
    }
    def
}

fn sequence_name(table: &str, column: &str) -> String {
    format!("{table}_{column}_seq")
}

fn render_create_table(table: &str, columns: &[NewColumn]) -> Vec<String> {
    let mut statements = Vec::new();
    let mut pk_columns: Vec<String> = Vec::new();
    let mut column_defs = Vec::new();

    for column in columns {
        let seq = if column.data_type == AUTO_INCREMENT_TYPE {
            let seq = sequence_name(table, &column.name);
            statements.push(format!("create sequence {seq}"));
            pk_columns.push(quote_ident(&column.name));
            Some(seq)
        } else {
            None
        };
        column_defs.push(render_column_def(column, seq.as_deref()));
    }

    let pk_clause = if pk_columns.is_empty() { String::new() } else { format!(", primary key ({})", pk_columns.join(", ")) };

    statements.push(format!(
        "create table {} ({}{})",
        quote_ident(table),
        column_defs.join(", "),
        pk_clause
    ));
    statements
}

fn render_alter_column(table: &str, edit: &ColumnEdit) -> Vec<String> {
    let mut statements = Vec::new();
    let t = quote_ident(table);
    let current = quote_ident(&edit.current_name);

    statements.push(format!("alter table {t} alter column {current} type {}", edit.column.data_type));

    if let Some(default) = &edit.column.default {
        statements.push(format!("alter table {t} alter column {current} set default {default}"));
    } else {
        statements.push(format!("alter table {t} alter column {current} drop default"));
    }

    if edit.column.is_nullable {
        statements.push(format!("alter table {t} alter column {current} drop not null"));
    } else {
        statements.push(format!("alter table {t} alter column {current} set not null"));
    }

    if edit.current_name != edit.column.name {
        statements.push(format!("alter table {t} rename column {current} to {}", quote_ident(&edit.column.name)));
    }
    statements
}

fn render_add_constraint(_constraint: &NewConstraint) -> Result<String, AppError> {
    Err(AppError::new(
        "DuckDB has no ALTER TABLE ADD CONSTRAINT — constraints (primary key, unique, check, foreign key) can only be set when the table is created.",
    ))
}

fn render_drop_constraint(_constraint: &str) -> Result<String, AppError> {
    Err(AppError::new(
        "DuckDB has no ALTER TABLE DROP CONSTRAINT — constraints can only be removed by recreating the table.",
    ))
}

pub fn render(statement: &DdlStatement) -> Result<Vec<String>, AppError> {
    match statement {
        DdlStatement::CreateTable { table, columns } => Ok(render_create_table(table, columns)),
        DdlStatement::RenameTable { table, new_name } => {
            Ok(vec![format!("alter table {} rename to {}", quote_ident(table), quote_ident(new_name))])
        }
        DdlStatement::DropTable { table } => Ok(vec![format!("drop table {}", quote_ident(table))]),
        DdlStatement::AddColumn { table, column } => {
            Ok(vec![format!("alter table {} add column {}", quote_ident(table), render_column_def(column, None))])
        }
        DdlStatement::DropColumn { table, column } => {
            Ok(vec![format!("alter table {} drop column {}", quote_ident(table), quote_ident(column))])
        }
        DdlStatement::AlterColumn { table, edit } => Ok(render_alter_column(table, edit)),
        DdlStatement::AddIndex { table, index } => {
            let unique = if index.is_unique { "unique " } else { "" };
            let columns = index.columns.iter().map(|c| quote_ident(c)).collect::<Vec<_>>().join(", ");
            Ok(vec![format!("create {unique}index {} on {} ({columns})", quote_ident(&index.name), quote_ident(table))])
        }
        DdlStatement::DropIndex { table: _, index } => Ok(vec![format!("drop index {}", quote_ident(index))]),
        DdlStatement::AddConstraint { constraint, .. } => Ok(vec![render_add_constraint(constraint)?]),
        DdlStatement::DropConstraint { constraint, .. } => Ok(vec![render_drop_constraint(constraint)?]),
    }
}

pub fn render_all(statements: &[DdlStatement]) -> Result<Vec<DdlPreview>, AppError> {
    let mut previews = Vec::new();
    for statement in statements {
        for sql in render(statement)? {
            previews.push(DdlPreview { sql });
        }
    }
    Ok(previews)
}

pub fn execute_all(conn: &Connection, statements: &[DdlStatement]) -> Result<DdlBatchResult, AppError> {
    let mut results = Vec::new();

    for statement in statements {
        let sub_statements = match render(statement) {
            Ok(stmts) => stmts,
            Err(e) => {
                results.push(DdlExecutionResult { sql: String::new(), success: false, error: Some(e.to_string()) });
                break;
            }
        };

        let mut failed = false;
        for sql in sub_statements {
            match conn.execute(&sql, []).map_err(clean) {
                Ok(_) => results.push(DdlExecutionResult { sql, success: true, error: None }),
                Err(e) => {
                    results.push(DdlExecutionResult { sql, success: false, error: Some(e.to_string()) });
                    failed = true;
                    break;
                }
            }
        }
        if failed {
            break;
        }
    }

    Ok(DdlBatchResult { results, rolled_back: false })
}
