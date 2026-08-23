use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::Value as JsonValue;
use sqlx::pool::PoolConnection;
use sqlx::sqlite::SqlitePool;
use sqlx::Sqlite;

use crate::domain::driver::DatabaseDriver;
use crate::domain::query::RawQueryResult;
use crate::domain::schema::{
    ColumnInfo, ConstraintInfo, DdlBatchResult, DdlPreview, DdlStatement, IndexInfo, TableRef,
};
use crate::domain::table::{TableFilter, TableRowsResult, TableSort};
use crate::error::AppError;

use super::{ddl, executor, metadata};

const MAX_PAGE_SIZE: i64 = 10_000;

/// SQLite has no schema layer, so every `DatabaseDriver` method here
/// ignores the `schema` parameter entirely (accepted only because the
/// trait is shared across every engine) rather than validating it —
/// unlike Postgres/MySQL, an unexpected `schema` value can't select the
/// wrong data or fail, since it's never used to build a query.
pub struct SqliteDriver {
    pool: SqlitePool,
    /// One dedicated connection per tab in manual-commit mode. SQLite
    /// only ever allows one writer at a time per file regardless of how
    /// many connections are open (see `pool::build_pool`'s
    /// `max_connections(1)`), so this mostly exists for API parity with
    /// Postgres/MySQL's manual-transaction support rather than to enable
    /// real concurrency — a reserved connection here still shares the
    /// same single-writer lock as everything else touching the file.
    reserved: Mutex<HashMap<String, PoolConnection<Sqlite>>>,
}

impl SqliteDriver {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool, reserved: Mutex::new(HashMap::new()) }
    }
}

fn json_to_insert_text(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::Null => None,
        JsonValue::String(s) => Some(s.clone()),
        JsonValue::Number(n) => Some(n.to_string()),
        JsonValue::Bool(b) => Some(if *b { "1".to_string() } else { "0".to_string() }),
        other => Some(other.to_string()),
    }
}

fn validate_identifier(ident: &str) -> Result<(), AppError> {
    let mut chars = ident.chars();
    let first_ok = chars.next().map(|c| c.is_ascii_alphabetic() || c == '_').unwrap_or(false);
    let rest_ok = chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
    if ident.is_empty() || !first_ok || !rest_ok {
        return Err(AppError::new(format!("Invalid identifier: {ident}")));
    }
    Ok(())
}

fn validate_ddl_statement(statement: &DdlStatement) -> Result<(), AppError> {
    match statement {
        DdlStatement::CreateTable { table, columns } => {
            validate_identifier(table)?;
            for column in columns {
                validate_identifier(&column.name)?;
            }
            Ok(())
        }
        DdlStatement::RenameTable { table, new_name } => {
            validate_identifier(table)?;
            validate_identifier(new_name)
        }
        DdlStatement::DropTable { table } => validate_identifier(table),
        DdlStatement::AddColumn { table, column } => {
            validate_identifier(table)?;
            validate_identifier(&column.name)
        }
        DdlStatement::DropColumn { table, column } => {
            validate_identifier(table)?;
            validate_identifier(column)
        }
        DdlStatement::AlterColumn { table, edit } => {
            validate_identifier(table)?;
            validate_identifier(&edit.current_name)?;
            validate_identifier(&edit.column.name)
        }
        DdlStatement::AddIndex { table, index } => {
            validate_identifier(table)?;
            validate_identifier(&index.name)?;
            for col in &index.columns {
                validate_identifier(col)?;
            }
            Ok(())
        }
        DdlStatement::DropIndex { table, index } => {
            validate_identifier(table)?;
            validate_identifier(index)
        }
        DdlStatement::AddConstraint { table, constraint } => {
            validate_identifier(table)?;
            validate_identifier(&constraint.name)?;
            for col in &constraint.columns {
                validate_identifier(col)?;
            }
            if let Some(ref_table) = &constraint.referenced_table {
                validate_identifier(ref_table)?;
            }
            for col in &constraint.referenced_columns {
                validate_identifier(col)?;
            }
            Ok(())
        }
        DdlStatement::DropConstraint { table, constraint } => {
            validate_identifier(table)?;
            validate_identifier(constraint)
        }
    }
}

fn pk_values_from_row(columns: &[ColumnInfo], row: &HashMap<String, JsonValue>) -> Result<Vec<(String, JsonValue)>, AppError> {
    let pk_columns: Vec<&str> = columns.iter().filter(|c| c.is_primary_key).map(|c| c.name.as_str()).collect();
    if pk_columns.is_empty() {
        return Err(AppError::new("This table has no primary key — cannot safely target a single row to update."));
    }
    let mut pk_values = Vec::with_capacity(pk_columns.len());
    for pk_col in &pk_columns {
        let value = row.get(*pk_col).ok_or_else(|| AppError::new(format!("Missing primary key value for '{pk_col}'.")))?;
        pk_values.push((pk_col.to_string(), value.clone()));
    }
    Ok(pk_values)
}

#[async_trait::async_trait]
impl DatabaseDriver for SqliteDriver {
    async fn server_version(&self) -> Result<String, AppError> {
        let row = sqlx::query("select sqlite_version() as version")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::new(format!("Connected, but failed to query server: {}", crate::error::describe_sqlite_error(&e))))?;
        sqlx::Row::try_get::<String, _>(&row, "version")
            .map_err(|e| AppError::new(format!("Connected, but failed to read server version: {e}")))
    }

    async fn list_tables(&self) -> Result<Vec<TableRef>, AppError> {
        metadata::list_tables(&self.pool).await
    }

    async fn get_table_columns(&self, _schema: &str, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
        validate_identifier(table)?;
        metadata::get_table_columns(&self.pool, table).await
    }

    async fn list_indexes(&self, _schema: &str, table: &str) -> Result<Vec<IndexInfo>, AppError> {
        validate_identifier(table)?;
        metadata::list_indexes(&self.pool, table).await
    }

    async fn list_constraints(&self, _schema: &str, table: &str) -> Result<Vec<ConstraintInfo>, AppError> {
        validate_identifier(table)?;
        metadata::list_constraints(&self.pool, table).await
    }

    async fn get_table_ddl(&self, _schema: &str, table: &str) -> Result<String, AppError> {
        validate_identifier(table)?;
        metadata::get_table_ddl(&self.pool, table).await
    }

    async fn render_ddl(&self, _schema: &str, statements: &[DdlStatement]) -> Result<Vec<DdlPreview>, AppError> {
        for statement in statements {
            validate_ddl_statement(statement)?;
        }
        ddl::render_all(&self.pool, statements).await
    }

    async fn execute_ddl(&self, _schema: &str, statements: &[DdlStatement]) -> Result<DdlBatchResult, AppError> {
        for statement in statements {
            validate_ddl_statement(statement)?;
        }
        ddl::execute_all(&self.pool, statements).await
    }

    async fn fetch_table_rows(
        &self,
        _schema: &str,
        table: &str,
        limit: i64,
        offset: i64,
        filters: &[TableFilter],
        sort: &[TableSort],
    ) -> Result<TableRowsResult, AppError> {
        validate_identifier(table)?;
        for filter in filters {
            validate_identifier(&filter.column)?;
        }
        for sort in sort {
            validate_identifier(&sort.column)?;
        }
        let limit = limit.clamp(1, MAX_PAGE_SIZE);
        let offset = offset.max(0);
        executor::fetch_rows(&self.pool, table, limit, offset, filters, sort).await
    }

    async fn count_table_rows(&self, _schema: &str, table: &str, filters: &[TableFilter]) -> Result<u64, AppError> {
        validate_identifier(table)?;
        for filter in filters {
            validate_identifier(&filter.column)?;
        }
        executor::count_rows(&self.pool, table, filters).await
    }

    async fn update_json_cell(
        &self,
        _schema: &str,
        table: &str,
        row: &HashMap<String, JsonValue>,
        column: &str,
        value: &JsonValue,
    ) -> Result<(), AppError> {
        validate_identifier(table)?;
        validate_identifier(column)?;
        let columns = metadata::get_table_columns(&self.pool, table).await?;
        let pk_values = pk_values_from_row(&columns, row)?;
        executor::update_json_cell(&self.pool, table, &pk_values, column, value).await
    }

    async fn update_cell_text(
        &self,
        _schema: &str,
        table: &str,
        row: &HashMap<String, JsonValue>,
        column: &str,
        new_value: Option<&str>,
    ) -> Result<(), AppError> {
        validate_identifier(table)?;
        validate_identifier(column)?;
        let columns = metadata::get_table_columns(&self.pool, table).await?;
        if !columns.iter().any(|c| c.name == column) {
            return Err(AppError::new(format!("Unknown column '{column}'.")));
        }
        let pk_values = pk_values_from_row(&columns, row)?;
        executor::update_cell_text(&self.pool, table, &pk_values, column, new_value).await
    }

    async fn delete_rows(&self, _schema: &str, table: &str, rows: &[HashMap<String, JsonValue>]) -> Result<u64, AppError> {
        validate_identifier(table)?;
        if rows.is_empty() {
            return Ok(0);
        }
        let columns = metadata::get_table_columns(&self.pool, table).await?;
        let mut rows_pk_values = Vec::with_capacity(rows.len());
        for row in rows {
            rows_pk_values.push(pk_values_from_row(&columns, row)?);
        }
        executor::delete_rows(&self.pool, table, &rows_pk_values).await
    }

    async fn insert_row(&self, _schema: &str, table: &str, values: &HashMap<String, JsonValue>) -> Result<(), AppError> {
        validate_identifier(table)?;
        let mut insert_values = Vec::with_capacity(values.len());
        for (column, value) in values {
            let Some(text) = json_to_insert_text(value) else { continue };
            validate_identifier(column)?;
            insert_values.push((column.clone(), text));
        }
        executor::insert_row(&self.pool, table, &insert_values).await
    }

    async fn execute_query(&self, sql: &str, offset: u64, limit: u64) -> Result<RawQueryResult, AppError> {
        if sql.trim().is_empty() {
            return Err(AppError::new("Cannot execute an empty query."));
        }
        let mut conn = self.pool.acquire().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        executor::execute_query(&mut conn, sql, offset, limit).await
    }

    async fn execute_query_for_tab(&self, tab_id: &str, sql: &str, offset: u64, limit: u64) -> Result<RawQueryResult, AppError> {
        if sql.trim().is_empty() {
            return Err(AppError::new("Cannot execute an empty query."));
        }
        let reserved = self.reserved.lock().unwrap().remove(tab_id);
        match reserved {
            Some(mut conn) => {
                let result = executor::execute_query(&mut conn, sql, offset, limit).await;
                self.reserved.lock().unwrap().insert(tab_id.to_string(), conn);
                result
            }
            None => {
                let mut conn = self.pool.acquire().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
                executor::execute_query(&mut conn, sql, offset, limit).await
            }
        }
    }

    async fn begin_transaction(&self, tab_id: &str) -> Result<(), AppError> {
        if self.reserved.lock().unwrap().contains_key(tab_id) {
            return Err(AppError::new("This tab already has an open transaction."));
        }
        let mut conn = self.pool.acquire().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        sqlx::raw_sql("BEGIN")
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::new(crate::error::describe_sqlite_error(&e)))?;
        self.reserved.lock().unwrap().insert(tab_id.to_string(), conn);
        Ok(())
    }

    async fn commit_transaction(&self, tab_id: &str) -> Result<(), AppError> {
        let mut conn = self.reserved.lock().unwrap().remove(tab_id).ok_or_else(|| AppError::new("This tab has no open transaction to commit."))?;
        sqlx::raw_sql("COMMIT")
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::new(crate::error::describe_sqlite_error(&e)))?;
        Ok(())
    }

    async fn rollback_transaction(&self, tab_id: &str) -> Result<(), AppError> {
        let mut conn = self.reserved.lock().unwrap().remove(tab_id).ok_or_else(|| AppError::new("This tab has no open transaction to roll back."))?;
        sqlx::raw_sql("ROLLBACK")
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::new(crate::error::describe_sqlite_error(&e)))?;
        Ok(())
    }

    fn has_active_transaction(&self, tab_id: &str) -> bool {
        self.reserved.lock().unwrap().contains_key(tab_id)
    }

    /// SQLite has no server process to send a kill signal to — cancelling
    /// a running query means calling `sqlite3_interrupt()` on the exact
    /// connection handle running it, which sqlx does not expose through
    /// its pooled connection API. Every query this driver runs is also
    /// against a local embedded file with no network round trip, so the
    /// class of "query hung for minutes, need to interrupt it" that
    /// justifies MySQL/Postgres's cancel support essentially doesn't
    /// arise here — this returns a clear, honest error instead of
    /// pretending to cancel something it can't reach.
    async fn cancel_query(&self, _tab_id: &str) -> Result<(), AppError> {
        Err(AppError::new("Cancelling a running query is not supported for SQLite."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plain_identifiers() {
        assert!(validate_identifier("users").is_ok());
        assert!(validate_identifier("_private").is_ok());
        assert!(validate_identifier("order_items").is_ok());
    }

    #[test]
    fn rejects_sql_injection_attempts() {
        assert!(validate_identifier("users; drop table users;--").is_err());
        assert!(validate_identifier("users\" OR \"1\"=\"1").is_err());
        assert!(validate_identifier("users.other").is_err());
    }
}
