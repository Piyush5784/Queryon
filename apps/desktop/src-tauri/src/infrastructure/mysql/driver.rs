use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::Value as JsonValue;
use sqlx::mysql::MySqlPool;
use sqlx::pool::PoolConnection;
use sqlx::{MySql, Row};

use crate::domain::driver::DatabaseDriver;
use crate::domain::query::RawQueryResult;
use crate::domain::schema::{
    ColumnInfo, ConstraintInfo, DdlBatchResult, DdlPreview, DdlStatement, IndexInfo, TableRef,
};
use crate::domain::table::{TableFilter, TableRowsResult, TableSort};
use crate::error::AppError;

use super::{ddl, executor, metadata};

const MAX_PAGE_SIZE: i64 = 10_000;

pub struct MySqlDriver {
    pool: MySqlPool,
    database: String,
    
    is_mariadb: bool,
    is_starrocks: bool,
    reserved: Mutex<HashMap<String, PoolConnection<MySql>>>,
    running_ids: Mutex<HashMap<String, u32>>,
}

impl MySqlDriver {
    pub fn new(pool: MySqlPool, database: String) -> Self {
        Self {
            pool,
            database,
            is_mariadb: false,
            is_starrocks: false,
            reserved: Mutex::new(HashMap::new()),
            running_ids: Mutex::new(HashMap::new()),
        }
    }

    pub fn new_mariadb(pool: MySqlPool, database: String) -> Self {
        Self {
            pool,
            database,
            is_mariadb: true,
            is_starrocks: false,
            reserved: Mutex::new(HashMap::new()),
            running_ids: Mutex::new(HashMap::new()),
        }
    }

    pub fn new_starrocks(pool: MySqlPool, database: String) -> Self {
        Self {
            pool,
            database,
            is_mariadb: false,
            is_starrocks: true,
            reserved: Mutex::new(HashMap::new()),
            running_ids: Mutex::new(HashMap::new()),
        }
    }

    async fn execute_query_trackable(
        &self,
        tab_id: &str,
        conn: &mut PoolConnection<MySql>,
        sql: &str,
        offset: u64,
        limit: u64,
    ) -> Result<RawQueryResult, AppError> {
        let id_row = sqlx::query("SELECT connection_id() AS id")
            .fetch_one(&mut **conn)
            .await
            .map_err(|e| AppError::new(crate::error::describe_mysql_error(&e)))?;
        let connection_id: u32 = id_row
            .try_get::<u64, _>("id")
            .map(|id| id as u32)
            .map_err(|e| AppError::new(format!("Could not read connection id: {e}")))?;
        self.running_ids.lock().unwrap().insert(tab_id.to_string(), connection_id);

        let result = executor::execute_query(&mut **conn, sql, offset, limit).await;

        self.running_ids.lock().unwrap().remove(tab_id);
        result
    }
}

fn json_to_insert_text(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::Null => None,
        JsonValue::String(s) => Some(s.clone()),
        JsonValue::Number(n) => Some(n.to_string()),
        JsonValue::Bool(b) => Some(b.to_string()),
        other => Some(other.to_string()),
    }
}

fn validate_identifier(ident: &str) -> Result<(), AppError> {
    let mut chars = ident.chars();
    let first_ok = chars
        .next()
        .map(|c| c.is_ascii_alphabetic() || c == '_')
        .unwrap_or(false);
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

fn pk_values_from_row(
    columns: &[ColumnInfo],
    row: &HashMap<String, JsonValue>,
) -> Result<Vec<(String, JsonValue)>, AppError> {
    let pk_columns: Vec<&str> = columns
        .iter()
        .filter(|c| c.is_primary_key)
        .map(|c| c.name.as_str())
        .collect();

    if pk_columns.is_empty() {
        return Err(AppError::new(
            "This table has no primary key — cannot safely target a single row to update.",
        ));
    }

    let mut pk_values = Vec::with_capacity(pk_columns.len());
    for pk_col in &pk_columns {
        let value = row
            .get(*pk_col)
            .ok_or_else(|| AppError::new(format!("Missing primary key value for '{pk_col}'.")))?;
        pk_values.push((pk_col.to_string(), value.clone()));
    }
    Ok(pk_values)
}

#[async_trait::async_trait]
impl DatabaseDriver for MySqlDriver {
    async fn server_version(&self) -> Result<String, AppError> {
        let row = sqlx::query("select version()")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::new(format!("Connected, but failed to query server: {e}")))?;
        row.try_get::<String, _>(0).map_err(|e| {
            AppError::new(format!("Connected, but failed to read server version: {e}"))
        })
    }

    async fn list_tables(&self) -> Result<Vec<TableRef>, AppError> {
        metadata::list_tables(&self.pool, &self.database).await
    }

    async fn get_table_columns(
        &self,
        schema: &str,
        table: &str,
    ) -> Result<Vec<ColumnInfo>, AppError> {
        metadata::get_table_columns(&self.pool, schema, table, self.is_mariadb, self.is_starrocks).await
    }

    async fn list_indexes(&self, schema: &str, table: &str) -> Result<Vec<IndexInfo>, AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;
        metadata::list_indexes(&self.pool, schema, table, self.is_starrocks).await
    }

    async fn list_constraints(
        &self,
        schema: &str,
        table: &str,
    ) -> Result<Vec<ConstraintInfo>, AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;
        metadata::list_constraints(&self.pool, schema, table, self.is_starrocks).await
    }

    async fn render_ddl(
        &self,
        schema: &str,
        statements: &[DdlStatement],
    ) -> Result<Vec<DdlPreview>, AppError> {
        validate_identifier(schema)?;
        for statement in statements {
            validate_ddl_statement(statement)?;
        }
        ddl::render_all(&self.pool, schema, statements, self.is_starrocks).await
    }

    async fn execute_ddl(
        &self,
        schema: &str,
        statements: &[DdlStatement],
    ) -> Result<DdlBatchResult, AppError> {
        validate_identifier(schema)?;
        for statement in statements {
            validate_ddl_statement(statement)?;
        }
        ddl::execute_all(&self.pool, schema, statements, self.is_starrocks).await
    }

    async fn get_table_ddl(&self, schema: &str, table: &str) -> Result<String, AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;
        metadata::get_table_ddl(&self.pool, table).await
    }

    async fn fetch_table_rows(
        &self,
        schema: &str,
        table: &str,
        limit: i64,
        offset: i64,
        filters: &[TableFilter],
        sort: &[TableSort],
    ) -> Result<TableRowsResult, AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;
        for filter in filters {
            validate_identifier(&filter.column)?;
        }
        for sort in sort {
            validate_identifier(&sort.column)?;
        }

        let limit = limit.clamp(1, MAX_PAGE_SIZE);
        let offset = offset.max(0);

        executor::fetch_rows(&self.pool, schema, table, limit, offset, filters, sort).await
    }

    async fn count_table_rows(
        &self,
        schema: &str,
        table: &str,
        filters: &[TableFilter],
    ) -> Result<u64, AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;
        for filter in filters {
            validate_identifier(&filter.column)?;
        }

        executor::count_rows(&self.pool, schema, table, filters).await
    }

    async fn update_json_cell(
        &self,
        schema: &str,
        table: &str,
        row: &HashMap<String, JsonValue>,
        column: &str,
        value: &JsonValue,
    ) -> Result<(), AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;
        validate_identifier(column)?;

        let columns = metadata::get_table_columns(&self.pool, schema, table, self.is_mariadb, self.is_starrocks).await?;
        let pk_values = pk_values_from_row(&columns, row)?;

        executor::update_json_cell(&self.pool, schema, table, &pk_values, column, value, self.is_starrocks).await
    }

    async fn update_cell_text(
        &self,
        schema: &str,
        table: &str,
        row: &HashMap<String, JsonValue>,
        column: &str,
        new_value: Option<&str>,
    ) -> Result<(), AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;
        validate_identifier(column)?;

        let columns = metadata::get_table_columns(&self.pool, schema, table, self.is_mariadb, self.is_starrocks).await?;
        if !columns.iter().any(|c| c.name == column) {
            return Err(AppError::new(format!("Unknown column '{column}'.")));
        }

        let pk_values = pk_values_from_row(&columns, row)?;

        executor::update_cell_text(&self.pool, schema, table, &pk_values, column, new_value, self.is_starrocks).await
    }

    async fn delete_rows(
        &self,
        schema: &str,
        table: &str,
        rows: &[HashMap<String, JsonValue>],
    ) -> Result<u64, AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;

        if rows.is_empty() {
            return Ok(0);
        }

        let columns = metadata::get_table_columns(&self.pool, schema, table, self.is_mariadb, self.is_starrocks).await?;

        let mut rows_pk_values = Vec::with_capacity(rows.len());
        for row in rows {
            rows_pk_values.push(pk_values_from_row(&columns, row)?);
        }

        executor::delete_rows(&self.pool, schema, table, &rows_pk_values, self.is_starrocks).await
    }

    async fn insert_row(
        &self,
        schema: &str,
        table: &str,
        values: &HashMap<String, JsonValue>,
    ) -> Result<(), AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;

        let mut insert_values = Vec::with_capacity(values.len());
        for (column, value) in values {
            let Some(text) = json_to_insert_text(value) else {
                continue;
            };
            validate_identifier(column)?;
            insert_values.push((column.clone(), text));
        }

        executor::insert_row(&self.pool, schema, table, &insert_values, self.is_starrocks).await
    }

    async fn execute_query(&self, sql: &str, offset: u64, limit: u64) -> Result<RawQueryResult, AppError> {
        if sql.trim().is_empty() {
            return Err(AppError::new("Cannot execute an empty query."));
        }
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        executor::execute_query(&mut conn, sql, offset, limit).await
    }

    async fn execute_query_for_tab(
        &self,
        tab_id: &str,
        sql: &str,
        offset: u64,
        limit: u64,
    ) -> Result<RawQueryResult, AppError> {
        if sql.trim().is_empty() {
            return Err(AppError::new("Cannot execute an empty query."));
        }

        // Std Mutex guards can't cross an .await point, so the reserved
        // connection is taken out of the map, used, then put back.
        let reserved = self.reserved.lock().unwrap().remove(tab_id);
        match reserved {
            Some(mut conn) => {
                let result = self.execute_query_trackable(tab_id, &mut conn, sql, offset, limit).await;
                self.reserved.lock().unwrap().insert(tab_id.to_string(), conn);
                result
            }
            None => {
                let mut conn = self
                    .pool
                    .acquire()
                    .await
                    .map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
                self.execute_query_trackable(tab_id, &mut conn, sql, offset, limit).await
            }
        }
    }

    async fn begin_transaction(&self, tab_id: &str) -> Result<(), AppError> {
        if self.reserved.lock().unwrap().contains_key(tab_id) {
            return Err(AppError::new("This tab already has an open transaction."));
        }
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        // MySQL rejects START TRANSACTION/COMMIT/ROLLBACK as prepared
        // statements ("This command is not supported in the prepared
        // statement protocol yet") — confirmed live. sqlx::query(...)
        // always prepares; sqlx::raw_sql runs on the plain text protocol
        // instead, which is what these three commands need.
        sqlx::raw_sql("START TRANSACTION")
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::new(crate::error::describe_mysql_error(&e)))?;
        self.reserved.lock().unwrap().insert(tab_id.to_string(), conn);
        Ok(())
    }

    async fn commit_transaction(&self, tab_id: &str) -> Result<(), AppError> {
        let mut conn = self.reserved.lock().unwrap().remove(tab_id).ok_or_else(|| {
            AppError::new("This tab has no open transaction to commit.")
        })?;
        sqlx::raw_sql("COMMIT")
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::new(crate::error::describe_mysql_error(&e)))?;
        Ok(())
    }

    async fn rollback_transaction(&self, tab_id: &str) -> Result<(), AppError> {
        let mut conn = self.reserved.lock().unwrap().remove(tab_id).ok_or_else(|| {
            AppError::new("This tab has no open transaction to roll back.")
        })?;
        sqlx::raw_sql("ROLLBACK")
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::new(crate::error::describe_mysql_error(&e)))?;
        Ok(())
    }

    fn has_active_transaction(&self, tab_id: &str) -> bool {
        self.reserved.lock().unwrap().contains_key(tab_id)
    }

    async fn cancel_query(&self, tab_id: &str) -> Result<(), AppError> {
        let connection_id = self
            .running_ids
            .lock()
            .unwrap()
            .get(tab_id)
            .copied()
            .ok_or_else(|| AppError::new("No query is currently running on this tab."))?;

        // The connection running the query is busy and can't cancel
        // itself — a separate connection sends KILL QUERY instead, same
        // as Beekeeper Studio's own MySQL cancel path.
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!("KILL QUERY {connection_id}")))
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::new(crate::error::describe_mysql_error(&e)))?;
        Ok(())
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
        assert!(validate_identifier("Users2").is_ok());
    }

    #[test]
    fn rejects_empty_identifier() {
        assert!(validate_identifier("").is_err());
    }

    #[test]
    fn rejects_identifier_starting_with_digit() {
        assert!(validate_identifier("2fast").is_err());
    }

    #[test]
    fn rejects_sql_injection_attempts() {
        assert!(validate_identifier("users; drop table users;--").is_err());
        assert!(validate_identifier("users` OR `1`=`1").is_err());
        assert!(validate_identifier("users--").is_err());
        assert!(validate_identifier("users.other").is_err());
        assert!(validate_identifier("users OR 1=1").is_err());
        assert!(validate_identifier("users)").is_err());
    }

    #[test]
    fn rejects_whitespace_and_quotes() {
        assert!(validate_identifier("my table").is_err());
        assert!(validate_identifier("`users`").is_err());
        assert!(validate_identifier("'users'").is_err());
    }
}
