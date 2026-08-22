use sqlx::mysql::MySqlPool;
use sqlx::AssertSqlSafe;

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

/// MySQL requires an auto_increment column to be declared a key in the
/// same `CREATE TABLE` — it cannot be added as a separate `ALTER TABLE
/// ADD CONSTRAINT` afterward (confirmed live: attempting that fails with
/// "Incorrect table definition; there can be only one auto column and it
/// must be defined as a key"). Declaring `primary key` inline in the
/// column definition itself, rather than as a separate statement, is a
/// shape MySQL genuinely allows and is what this pseudo-type resolves
/// to. `is_nullable`/`default` are ignored, same reasoning as Postgres's
/// `serial` — an auto-increment key is implicitly NOT NULL and its
/// "default" is the auto-increment sequence itself.
fn render_column_def(column: &NewColumn) -> String {
    if column.data_type == AUTO_INCREMENT_TYPE {
        return format!("{} int not null auto_increment primary key", quote_ident(&column.name));
    }
    let nullability = if column.is_nullable { "" } else { " not null" };
    let default = column
        .default
        .as_ref()
        .map(|d| format!(" default {d}"))
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
) -> Result<String, AppError> {
    if columns.is_empty() {
        return Err(AppError::new("A table needs at least one column."));
    }
    let column_defs = columns
        .iter()
        .map(render_column_def)
        .collect::<Vec<_>>()
        .join(", ");
    Ok(format!(
        "create table {} ({});",
        quote_ident(table),
        column_defs
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

fn render_add_column(_schema: &str, table: &str, column: &NewColumn) -> String {
    format!(
        "alter table {} add column {};",
        quote_ident(table),
        render_column_def(column)
    )
}

fn render_drop_column(_schema: &str, table: &str, column: &str) -> String {
    format!(
        "alter table {} drop column {};",
        quote_ident(table),
        quote_ident(column)
    )
}

/// MySQL has no separate clauses for type/nullability/default the way
/// Postgres does — `CHANGE COLUMN old_name new_name full_definition`
/// restates the entire column in one go, renaming it in the same
/// statement if the name changed (or repeating the same name if not).
fn render_alter_column(_schema: &str, table: &str, edit: &ColumnEdit) -> String {
    let nullability = if edit.column.is_nullable {
        ""
    } else {
        " not null"
    };
    let default = edit
        .column
        .default
        .as_ref()
        .map(|d| format!(" default {d}"))
        .unwrap_or_default();
    format!(
        "alter table {} change column {} {} {}{}{};",
        quote_ident(table),
        quote_ident(&edit.current_name),
        quote_ident(&edit.column.name),
        edit.column.data_type,
        nullability,
        default
    )
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
) -> Result<String, AppError> {
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

/// MySQL's `ALTER TABLE ... DROP CONSTRAINT` needs to know *which kind*
/// of constraint it is dropping (`DROP FOREIGN KEY name` / `DROP CHECK
/// name` / `DROP INDEX name` for primary key/unique, which MySQL
/// implements as indexes) — unlike Postgres's uniform `DROP CONSTRAINT
/// name` that works for every kind. `DdlStatement::DropConstraint` only
/// carries a name, so the caller resolves the kind via
/// `list_constraints` first and passes it through this hint struct.
pub struct DdlDropConstraintKindHint {
    pub name: String,
    pub kind: ConstraintKind,
}

pub fn render(
    schema: &str,
    statement: &DdlStatement,
    drop_constraint_kind: Option<&DdlDropConstraintKindHint>,
) -> Result<String, AppError> {
    match statement {
        DdlStatement::CreateTable { table, columns } => render_create_table(schema, table, columns),
        DdlStatement::RenameTable { table, new_name } => {
            Ok(render_rename_table(schema, table, new_name))
        }
        DdlStatement::DropTable { table } => Ok(render_drop_table(schema, table)),
        DdlStatement::AddColumn { table, column } => Ok(render_add_column(schema, table, column)),
        DdlStatement::DropColumn { table, column } => Ok(render_drop_column(schema, table, column)),
        DdlStatement::AlterColumn { table, edit } => Ok(render_alter_column(schema, table, edit)),
        DdlStatement::AddIndex { table, index } => Ok(render_add_index(schema, table, index)),
        DdlStatement::DropIndex { table, index } => Ok(render_drop_index(schema, table, index)),
        DdlStatement::AddConstraint { table, constraint } => {
            render_add_constraint(schema, table, constraint)
        }
        DdlStatement::DropConstraint { table, constraint } => {
            let hint = drop_constraint_kind.ok_or_else(|| {
                AppError::new(format!(
                    "Could not determine constraint '{constraint}' kind — it may no longer exist."
                ))
            })?;
            Ok(render_drop_constraint(schema, table, hint))
        }
    }
}

pub async fn render_all(
    pool: &MySqlPool,
    schema: &str,
    statements: &[DdlStatement],
) -> Result<Vec<DdlPreview>, AppError> {
    let mut results = Vec::with_capacity(statements.len());
    for statement in statements {
        let hint = resolve_drop_constraint_hint(pool, schema, statement).await?;
        let sql = render(schema, statement, hint.as_ref())?;
        results.push(DdlPreview { sql });
    }
    Ok(results)
}

/// MySQL DDL auto-commits per statement — there is no way to roll back
/// an earlier statement once a later one in the same batch fails, unlike
/// Postgres's transactional DDL. Execution here stops at the first
/// failure rather than continuing, so partial application is at least
/// predictable (everything up to and including the failing statement),
/// but earlier successful statements are NOT undone.
pub async fn execute_all(
    pool: &MySqlPool,
    schema: &str,
    statements: &[DdlStatement],
) -> Result<DdlBatchResult, AppError> {
    let mut results = Vec::with_capacity(statements.len());
    for statement in statements {
        let hint = match resolve_drop_constraint_hint(pool, schema, statement).await {
            Ok(hint) => hint,
            Err(e) => {
                results.push(DdlExecutionResult {
                    sql: String::new(),
                    success: false,
                    error: Some(e.to_string()),
                });
                break;
            }
        };
        let sql = match render(schema, statement, hint.as_ref()) {
            Ok(sql) => sql,
            Err(e) => {
                results.push(DdlExecutionResult {
                    sql: String::new(),
                    success: false,
                    error: Some(e.to_string()),
                });
                break;
            }
        };

        match sqlx::query(AssertSqlSafe(sql.clone())).execute(pool).await {
            Ok(_) => results.push(DdlExecutionResult {
                sql,
                success: true,
                error: None,
            }),
            Err(e) => {
                results.push(DdlExecutionResult {
                    sql,
                    success: false,
                    error: Some(clean_mysql_error(&e)),
                });
                break;
            }
        }
    }
    Ok(DdlBatchResult {
        results,
        rolled_back: false,
    })
}

async fn resolve_drop_constraint_hint(
    pool: &MySqlPool,
    schema: &str,
    statement: &DdlStatement,
) -> Result<Option<DdlDropConstraintKindHint>, AppError> {
    let DdlStatement::DropConstraint { table, constraint } = statement else {
        return Ok(None);
    };
    let constraints = super::metadata::list_constraints(pool, schema, table).await?;
    let matching = constraints.into_iter().find(|c| &c.name == constraint);
    Ok(matching.map(|c| DdlDropConstraintKindHint {
        name: c.name,
        kind: c.kind,
    }))
}
