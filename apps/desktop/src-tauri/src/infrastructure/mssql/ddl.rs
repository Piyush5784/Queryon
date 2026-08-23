use crate::domain::schema::{
    ColumnEdit, ConstraintKind, DdlBatchResult, DdlExecutionResult, DdlPreview, DdlStatement,
    ForeignKeyAction, NewColumn, NewConstraint, NewIndex, AUTO_INCREMENT_TYPE,
};
use crate::error::AppError;

use super::metadata::quote_ident;
use super::pool::MssqlClient;


fn render_column_def(column: &NewColumn) -> String {
    if column.data_type == AUTO_INCREMENT_TYPE {
        return format!("{} int identity(1,1) primary key", quote_ident(&column.name));
    }
    let nullability = if column.is_nullable { "" } else { " not null" };
    let default = column.default.as_ref().map(|d| format!(" default {d}")).unwrap_or_default();
    format!("{} {}{}{}", quote_ident(&column.name), column.data_type, nullability, default)
}

fn render_create_table(schema: &str, table: &str, columns: &[NewColumn]) -> Result<String, AppError> {
    if columns.is_empty() {
        return Err(AppError::new("A table needs at least one column."));
    }
    let column_defs = columns.iter().map(render_column_def).collect::<Vec<_>>().join(", ");
    Ok(format!("create table {}.{} ({});", quote_ident(schema), quote_ident(table), column_defs))
}

fn render_rename_table(schema: &str, table: &str, new_name: &str) -> String {
 
    format!("EXEC sp_rename '{schema}.{table}', '{new_name}';")
}

fn render_drop_table(schema: &str, table: &str) -> String {
    format!("drop table {}.{};", quote_ident(schema), quote_ident(table))
}

fn render_add_column(schema: &str, table: &str, column: &NewColumn) -> String {
    format!(
        "alter table {}.{} add {};",
        quote_ident(schema),
        quote_ident(table),
        render_column_def(column)
    )
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
fn render_drop_index(schema: &str, table: &str, index: &str) -> String {
    format!("drop index {} on {}.{};", quote_ident(index), quote_ident(schema), quote_ident(table))
}

fn render_foreign_key_action(action: ForeignKeyAction) -> &'static str {
    match action {
        ForeignKeyAction::NoAction => "no action",
        ForeignKeyAction::Restrict => "no action",
        ForeignKeyAction::Cascade => "cascade",
        ForeignKeyAction::SetNull => "set null",
        ForeignKeyAction::SetDefault => "set default",
    }
}

fn render_add_constraint(schema: &str, table: &str, constraint: &NewConstraint) -> Result<String, AppError> {
    let cols = constraint.columns.iter().map(|c| quote_ident(c)).collect::<Vec<_>>().join(", ");
    let clause = match constraint.kind {
        ConstraintKind::PrimaryKey => format!("primary key ({cols})"),
        ConstraintKind::Unique => format!("unique ({cols})"),
        ConstraintKind::ForeignKey => {
            let ref_table = constraint.referenced_table.as_deref().ok_or_else(|| AppError::new("A foreign key needs a referenced table."))?;
            let ref_cols = constraint.referenced_columns.iter().map(|c| quote_ident(c)).collect::<Vec<_>>().join(", ");
            let on_update = constraint.on_update.map(|a| format!(" on update {}", render_foreign_key_action(a))).unwrap_or_default();
            let on_delete = constraint.on_delete.map(|a| format!(" on delete {}", render_foreign_key_action(a))).unwrap_or_default();
            format!("foreign key ({cols}) references {} ({ref_cols}){on_update}{on_delete}", quote_ident(ref_table))
        }
        ConstraintKind::Check => {
            let expr = constraint.check_expression.as_deref().ok_or_else(|| AppError::new("A check constraint needs an expression."))?;
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

async fn render_alter_column(
    client: &mut MssqlClient,
    schema: &str,
    table: &str,
    edit: &ColumnEdit,
) -> Result<String, AppError> {
    let mut statements = Vec::new();

    if let Some(default_name) = super::metadata::default_constraint_name(client, table, &edit.current_name).await? {
        statements.push(format!("alter table {}.{} drop constraint {};", quote_ident(schema), quote_ident(table), quote_ident(&default_name)));
    }

    let nullability = if edit.column.is_nullable { "null" } else { "not null" };
    statements.push(format!(
        "alter table {}.{} alter column {} {} {};",
        quote_ident(schema),
        quote_ident(table),
        quote_ident(&edit.current_name),
        edit.column.data_type,
        nullability
    ));

    if edit.column.name != edit.current_name {
        statements.push(format!(
            "EXEC sp_rename '{schema}.{table}.{}', '{}', 'COLUMN';",
            edit.current_name, edit.column.name
        ));
    }

    if let Some(default) = &edit.column.default {
        let final_name = &edit.column.name;
        statements.push(format!(
            "alter table {}.{} add constraint {} default {} for {};",
            quote_ident(schema),
            quote_ident(table),
            quote_ident(&format!("DF_{table}_{final_name}")),
            default,
            quote_ident(final_name)
        ));
    }

    Ok(statements.join("\n"))
}
async fn render_drop_column(
    client: &mut MssqlClient,
    schema: &str,
    table: &str,
    column: &str,
) -> Result<String, AppError> {
    let mut statements = Vec::new();

    if let Some(default_name) = super::metadata::default_constraint_name(client, table, column).await? {
        statements.push(format!("alter table {}.{} drop constraint {};", quote_ident(schema), quote_ident(table), quote_ident(&default_name)));
    }

    for index_ref in super::metadata::indexes_on_column(client, schema, table, column).await? {
        let statement = if index_ref.is_constraint {
            format!("alter table {}.{} drop constraint {};", quote_ident(schema), quote_ident(table), quote_ident(&index_ref.name))
        } else {
            format!("drop index {} on {}.{};", quote_ident(&index_ref.name), quote_ident(schema), quote_ident(table))
        };
        statements.push(statement);
    }

    statements.push(format!("alter table {}.{} drop column {};", quote_ident(schema), quote_ident(table), quote_ident(column)));
    Ok(statements.join("\n"))
}

async fn render_one(client: &mut MssqlClient, schema: &str, statement: &DdlStatement) -> Result<String, AppError> {
    match statement {
        DdlStatement::CreateTable { table, columns } => render_create_table(schema, table, columns),
        DdlStatement::RenameTable { table, new_name } => Ok(render_rename_table(schema, table, new_name)),
        DdlStatement::DropTable { table } => Ok(render_drop_table(schema, table)),
        DdlStatement::AddColumn { table, column } => Ok(render_add_column(schema, table, column)),
        DdlStatement::DropColumn { table, column } => render_drop_column(client, schema, table, column).await,
        DdlStatement::AlterColumn { table, edit } => render_alter_column(client, schema, table, edit).await,
        DdlStatement::AddIndex { table, index } => Ok(render_add_index(schema, table, index)),
        DdlStatement::DropIndex { table, index } => Ok(render_drop_index(schema, table, index)),
        DdlStatement::AddConstraint { table, constraint } => render_add_constraint(schema, table, constraint),
        DdlStatement::DropConstraint { table, constraint } => Ok(render_drop_constraint(schema, table, constraint)),
    }
}

pub async fn render_all(client: &mut MssqlClient, schema: &str, statements: &[DdlStatement]) -> Result<Vec<DdlPreview>, AppError> {
    let mut results = Vec::with_capacity(statements.len());
    for statement in statements {
        let sql = render_one(client, schema, statement).await?;
        results.push(DdlPreview { sql });
    }
    Ok(results)
}

pub async fn execute_all(client: &mut MssqlClient, schema: &str, statements: &[DdlStatement]) -> Result<DdlBatchResult, AppError> {
    let mut results = Vec::with_capacity(statements.len());

    client
        .simple_query("BEGIN TRANSACTION")
        .await
        .map_err(|e| AppError::new(format!("Failed to start transaction: {e}")))?;

    for statement in statements {
        let sql = match render_one(client, schema, statement).await {
            Ok(sql) => sql,
            Err(e) => {
                results.push(DdlExecutionResult { sql: String::new(), success: false, error: Some(e.to_string()) });
                let _ = client.simple_query("ROLLBACK TRANSACTION").await;
                return Ok(DdlBatchResult { results, rolled_back: true });
            }
        };

        let outcome = match client.simple_query(sql.clone()).await {
            Ok(stream) => stream.into_results().await.map(|_| ()),
            Err(e) => Err(e),
        };
        match outcome {
            Ok(()) => results.push(DdlExecutionResult { sql, success: true, error: None }),
            Err(e) => {
                results.push(DdlExecutionResult { sql, success: false, error: Some(e.to_string()) });
                let _ = client.simple_query("ROLLBACK TRANSACTION").await;
                return Ok(DdlBatchResult { results, rolled_back: true });
            }
        }
    }

    client
        .simple_query("COMMIT TRANSACTION")
        .await
        .map_err(|e| AppError::new(format!("Failed to commit: {e}")))?;
    Ok(DdlBatchResult { results, rolled_back: false })
}
