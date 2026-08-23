use sqlx::sqlite::SqlitePool;
use sqlx::{AssertSqlSafe, Row, SqliteConnection};

use crate::domain::schema::{
    ColumnEdit, ColumnInfo, ConstraintKind, DdlBatchResult, DdlExecutionResult, DdlPreview,
    DdlStatement, ForeignKeyAction, IndexInfo, NewColumn, NewConstraint, NewIndex,
    AUTO_INCREMENT_TYPE,
};
use crate::error::AppError;

use super::metadata::quote_ident;

fn render_column_def(column: &NewColumn) -> String {
    if column.data_type == AUTO_INCREMENT_TYPE {
        return format!("{} integer primary key autoincrement", quote_ident(&column.name));
    }
    let nullability = if column.is_nullable { "" } else { " not null" };
    let default = column.default.as_ref().map(|d| format!(" default ({d})")).unwrap_or_default();
    format!("{} {}{}{}", quote_ident(&column.name), column.data_type, nullability, default)
}

fn render_create_table(table: &str, columns: &[NewColumn]) -> Result<String, AppError> {
    if columns.is_empty() {
        return Err(AppError::new("A table needs at least one column."));
    }
    let column_defs = columns.iter().map(render_column_def).collect::<Vec<_>>().join(", ");
    Ok(format!("create table {} ({});", quote_ident(table), column_defs))
}

fn render_rename_table(table: &str, new_name: &str) -> String {
    format!("alter table {} rename to {};", quote_ident(table), quote_ident(new_name))
}

fn render_drop_table(table: &str) -> String {
    format!("drop table {};", quote_ident(table))
}

fn render_add_column(table: &str, column: &NewColumn) -> String {
    format!("alter table {} add column {};", quote_ident(table), render_column_def(column))
}

fn render_add_index(table: &str, index: &NewIndex) -> String {
    let unique = if index.is_unique { "unique " } else { "" };
    let cols = index.columns.iter().map(|c| quote_ident(c)).collect::<Vec<_>>().join(", ");
    format!("create {unique}index {} on {} ({});", quote_ident(&index.name), quote_ident(table), cols)
}

/// SQLite's `DROP INDEX` is standalone like Postgres's, not scoped
/// through `ALTER TABLE` like MySQL's — it doesn't even take the owning
/// table name.
fn render_drop_index(index: &str) -> String {
    format!("drop index {};", quote_ident(index))
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

async fn columns_of(conn: &mut SqliteConnection, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
    let sql = format!("PRAGMA table_xinfo({})", quote_ident(table));
    let rows = sqlx::query(AssertSqlSafe(sql))
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| AppError::new(format!("Failed to load columns: {}", crate::error::describe_sqlite_error(&e))))?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let pk: i64 = row.get("pk");
            let cid: i64 = row.get("cid");
            let notnull: i64 = row.get("notnull");
            ColumnInfo {
                name: row.get("name"),
                data_type: row.get("type"),
                is_nullable: notnull == 0,
                default: row.get("dflt_value"),
                is_primary_key: pk > 0,
                ordinal_position: cid as i32,
            }
        })
        .collect())
}

async fn indexes_of(conn: &mut SqliteConnection, table: &str) -> Result<Vec<IndexInfo>, AppError> {
    let sql = format!("PRAGMA index_list({})", quote_ident(table));
    let rows = sqlx::query(AssertSqlSafe(sql))
        .fetch_all(&mut *conn)
        .await
        .map_err(|e| AppError::new(format!("Failed to list indexes: {}", crate::error::describe_sqlite_error(&e))))?;

    let mut indexes = Vec::with_capacity(rows.len());
    for row in rows {
        let name: String = row.get("name");
        let unique: i64 = row.get("unique");
        let origin: String = row.get("origin");

        let xinfo_sql = format!("PRAGMA index_xinfo({})", quote_ident(&name));
        let xinfo_rows = sqlx::query(AssertSqlSafe(xinfo_sql))
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| AppError::new(format!("Failed to load index columns: {}", crate::error::describe_sqlite_error(&e))))?;
        let columns: Vec<String> = xinfo_rows
            .into_iter()
            .filter(|r| { let key: i64 = r.get("key"); key != 0 })
            .map(|r| r.get::<String, _>("name"))
            .collect();

        indexes.push(IndexInfo { name, columns, is_unique: unique != 0, is_primary: origin == "pk" });
    }
    Ok(indexes)
}

fn rebuild_tail(table: &str, temp_table: &str, indexes: &[IndexInfo], skip_index: Option<&str>) -> Vec<String> {
    let drop_old = format!("drop table {};", quote_ident(table));
    let rename = format!("alter table {} rename to {};", quote_ident(temp_table), quote_ident(table));

    let reindex: Vec<String> = indexes
        .iter()
        .filter(|i| !i.is_primary && !i.name.starts_with("sqlite_autoindex_") && Some(i.name.as_str()) != skip_index)
        .map(|i| {
            let unique = if i.is_unique { "unique " } else { "" };
            let cols = i.columns.iter().map(|c| quote_ident(c)).collect::<Vec<_>>().join(", ");
            format!("create {unique}index {} on {} ({});", quote_ident(&i.name), quote_ident(table), cols)
        })
        .collect();

    let mut tail = vec![drop_old, rename];
    tail.extend(reindex);
    tail.push("pragma foreign_keys=on;".to_string());
    tail
}

fn column_def_text(c: &ColumnInfo) -> String {
    let nullability = if c.is_nullable { "" } else { " not null" };
    let default = c.default.as_ref().map(|d| format!(" default ({d})")).unwrap_or_default();
    format!("{} {}{}{}", quote_ident(&c.name), c.data_type, nullability, default)
}

async fn render_drop_column(conn: &mut SqliteConnection, table: &str, column: &str) -> Result<String, AppError> {
    let columns = columns_of(conn, table).await?;
    let indexes = indexes_of(conn, table).await?;
    let is_constrained = indexes.iter().any(|i| (i.is_primary || i.is_unique) && i.columns.iter().any(|c| c == column));

    if !is_constrained {
        return Ok(format!("alter table {} drop column {};", quote_ident(table), quote_ident(column)));
    }

    let kept: Vec<NewColumn> = columns
        .iter()
        .filter(|c| c.name != column)
        .map(|c| NewColumn { name: c.name.clone(), data_type: c.data_type.clone(), is_nullable: c.is_nullable, default: c.default.clone() })
        .collect();
    if kept.is_empty() {
        return Err(AppError::new("Cannot drop the only remaining column."));
    }
    render_rebuild(conn, table, &kept, None).await
}

async fn render_alter_column(conn: &mut SqliteConnection, table: &str, edit: &ColumnEdit) -> Result<String, AppError> {
    let columns = columns_of(conn, table).await?;
    let new_columns: Vec<NewColumn> = columns
        .iter()
        .map(|c| {
            if c.name == edit.current_name {
                edit.column.clone()
            } else {
                NewColumn { name: c.name.clone(), data_type: c.data_type.clone(), is_nullable: c.is_nullable, default: c.default.clone() }
            }
        })
        .collect();
    render_rebuild(conn, table, &new_columns, Some(&edit.current_name)).await
}

async fn render_rebuild(
    conn: &mut SqliteConnection,
    table: &str,
    new_columns: &[NewColumn],
    renamed_from: Option<&str>,
) -> Result<String, AppError> {
    let old_columns = columns_of(conn, table).await?;
    let indexes = indexes_of(conn, table).await?;
    let temp_table = format!("{table}_bks_rebuild");

    let create = render_create_table(&temp_table, new_columns)?;

    let mut insert_list = Vec::with_capacity(new_columns.len());
    let mut select_list = Vec::with_capacity(new_columns.len());
    for new_col in new_columns {
        let source_name = if renamed_from.is_some() && !old_columns.iter().any(|c| c.name == new_col.name) {
            renamed_from.unwrap().to_string()
        } else {
            new_col.name.clone()
        };
        insert_list.push(quote_ident(&new_col.name));
        select_list.push(quote_ident(&source_name));
    }

    let copy = format!(
        "insert into {} ({}) select {} from {};",
        quote_ident(&temp_table),
        insert_list.join(", "),
        select_list.join(", "),
        quote_ident(table)
    );

    let mut statements = vec!["pragma foreign_keys=off;".to_string(), create, copy];
    statements.extend(rebuild_tail(table, &temp_table, &indexes, None));
    Ok(statements.join("\n"))
}

async fn render_add_constraint(conn: &mut SqliteConnection, table: &str, constraint: &NewConstraint) -> Result<String, AppError> {
    let old_columns = columns_of(conn, table).await?;
    let indexes = indexes_of(conn, table).await?;
    let temp_table = format!("{table}_bks_rebuild");

    let column_defs: Vec<String> = old_columns.iter().map(column_def_text).collect();

    let cols = constraint.columns.iter().map(|c| quote_ident(c)).collect::<Vec<_>>().join(", ");
    let constraint_clause = match constraint.kind {
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

    let create = format!(
        "create table {} ({}, constraint {} {});",
        quote_ident(&temp_table),
        column_defs.join(", "),
        quote_ident(&constraint.name),
        constraint_clause
    );
    let all_cols = old_columns.iter().map(|c| quote_ident(&c.name)).collect::<Vec<_>>().join(", ");
    let copy = format!("insert into {} ({}) select {} from {};", quote_ident(&temp_table), all_cols, all_cols, quote_ident(table));

    let mut statements = vec!["pragma foreign_keys=off;".to_string(), create, copy];
    statements.extend(rebuild_tail(table, &temp_table, &indexes, None));
    Ok(statements.join("\n"))
}

async fn render_drop_constraint(conn: &mut SqliteConnection, table: &str, constraint: &str) -> Result<String, AppError> {
    let row = sqlx::query("select sql from sqlite_master where name = ? and type = 'table'")
        .bind(table)
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| AppError::new(format!("Failed to get table DDL: {}", crate::error::describe_sqlite_error(&e))))?;
    let ddl: String = match row {
        Some(row) => row.try_get("sql").map_err(|e| AppError::new(format!("Unexpected response reading table DDL: {e}")))?,
        None => return Err(AppError::new(format!("Table '{table}' not found."))),
    };

    let lower = ddl.to_lowercase();
    let needle_quoted = format!("constraint \"{}\"", constraint.to_lowercase());
    let needle_bare = format!("constraint {}", constraint.to_lowercase());
    if !lower.contains(&needle_quoted) && !lower.contains(&needle_bare) {
        return Err(AppError::new(format!(
            "Constraint '{constraint}' was not declared with a table-level CONSTRAINT clause — SQLite has no way to drop an inline column constraint (e.g. a column-level UNIQUE) by name. Recreate the table instead."
        )));
    }

    let old_columns = columns_of(conn, table).await?;
    let indexes = indexes_of(conn, table).await?;
    let temp_table = format!("{table}_bks_rebuild");

    let column_defs: Vec<String> = old_columns.iter().map(column_def_text).collect();
    let create = format!("create table {} ({});", quote_ident(&temp_table), column_defs.join(", "));
    let all_cols = old_columns.iter().map(|c| quote_ident(&c.name)).collect::<Vec<_>>().join(", ");
    let copy = format!("insert into {} ({}) select {} from {};", quote_ident(&temp_table), all_cols, all_cols, quote_ident(table));

    let mut statements = vec!["pragma foreign_keys=off;".to_string(), create, copy];
    statements.extend(rebuild_tail(table, &temp_table, &indexes, Some(constraint)));
    Ok(statements.join("\n"))
}

async fn render_one(conn: &mut SqliteConnection, statement: &DdlStatement) -> Result<String, AppError> {
    match statement {
        DdlStatement::CreateTable { table, columns } => render_create_table(table, columns),
        DdlStatement::RenameTable { table, new_name } => Ok(render_rename_table(table, new_name)),
        DdlStatement::DropTable { table } => Ok(render_drop_table(table)),
        DdlStatement::AddColumn { table, column } => Ok(render_add_column(table, column)),
        DdlStatement::DropColumn { table, column } => render_drop_column(conn, table, column).await,
        DdlStatement::AlterColumn { table, edit } => render_alter_column(conn, table, edit).await,
        DdlStatement::AddIndex { table, index } => Ok(render_add_index(table, index)),
        DdlStatement::DropIndex { index, .. } => Ok(render_drop_index(index)),
        DdlStatement::AddConstraint { table, constraint } => render_add_constraint(conn, table, constraint).await,
        DdlStatement::DropConstraint { table, constraint } => render_drop_constraint(conn, table, constraint).await,
    }
}

pub async fn render_all(pool: &SqlitePool, statements: &[DdlStatement]) -> Result<Vec<DdlPreview>, AppError> {
    let mut conn = pool.acquire().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
    let mut results = Vec::with_capacity(statements.len());
    for statement in statements {
        let sql = render_one(&mut conn, statement).await?;
        results.push(DdlPreview { sql });
    }
    Ok(results)
}

pub async fn execute_all(pool: &SqlitePool, statements: &[DdlStatement]) -> Result<DdlBatchResult, AppError> {
    let mut results = Vec::with_capacity(statements.len());
    let mut conn = pool.acquire().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;

    for statement in statements {
        let sql = match render_one(&mut conn, statement).await {
            Ok(sql) => sql,
            Err(e) => {
                results.push(DdlExecutionResult { sql: String::new(), success: false, error: Some(e.to_string()) });
                break;
            }
        };

        match sqlx::raw_sql(AssertSqlSafe(sql.clone())).execute(&mut *conn).await {
            Ok(_) => results.push(DdlExecutionResult { sql, success: true, error: None }),
            Err(e) => {
                results.push(DdlExecutionResult { sql, success: false, error: Some(crate::error::describe_sqlite_error(&e)) });
                break;
            }
        }
    }

    Ok(DdlBatchResult { results, rolled_back: false })
}
