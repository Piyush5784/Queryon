use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::Value as JsonValue;
use tiberius::Query;

use crate::domain::driver::DatabaseDriver;
use crate::domain::query::RawQueryResult;
use crate::domain::schema::{
    ColumnInfo, ConstraintInfo, DdlBatchResult, DdlPreview, DdlStatement, IndexInfo, TableRef,
};
use crate::domain::table::{TableFilter, TableRowsResult, TableSort};
use crate::error::AppError;

use super::pool::{MssqlClient, MssqlPool};
use super::{ddl, executor, metadata};

const MAX_PAGE_SIZE: i64 = 10_000;
const DEFAULT_SCHEMA: &str = "dbo";

pub struct MssqlDriver {
    pool: MssqlPool,
    /// One dedicated connection per tab in manual-commit mode — same
    /// reasoning as MySQL/Postgres's `reserved` field. `deadpool`'s pooled
    /// object type (`deadpool::managed::Object<MssqlManager>`) is what's
    /// actually stored; it derefs to `MssqlClient`.
    reserved: Mutex<HashMap<String, deadpool::managed::Object<super::pool::MssqlManager>>>,
    /// The SQL Server `@@SPID` of whichever connection `tab_id` currently
    /// has a query running on — same purpose as MySQL's `running_ids`,
    /// see its doc comment. `cancel_query` runs `KILL <spid>` on a
    /// separate connection, since the one running the query is busy.
    running_spids: Mutex<HashMap<String, i16>>,
}

impl MssqlDriver {
    pub fn new(pool: MssqlPool) -> Self {
        Self { pool, reserved: Mutex::new(HashMap::new()), running_spids: Mutex::new(HashMap::new()) }
    }

    async fn execute_query_trackable(
        &self,
        tab_id: &str,
        client: &mut MssqlClient,
        sql: &str,
        offset: u64,
        limit: u64,
    ) -> Result<RawQueryResult, AppError> {
        let spid_row = client
            .simple_query("SELECT @@SPID as spid")
            .await
            .map_err(|e| AppError::new(e.to_string()))?
            .into_row()
            .await
            .map_err(|e| AppError::new(e.to_string()))?;
        let spid: i16 = spid_row.and_then(|r| r.get("spid")).unwrap_or(0);
        self.running_spids.lock().unwrap().insert(tab_id.to_string(), spid);

        let result = executor::execute_query(client, sql, offset, limit).await;

        self.running_spids.lock().unwrap().remove(tab_id);
        result
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
impl DatabaseDriver for MssqlDriver {
    async fn server_version(&self) -> Result<String, AppError> {
        let mut client = self.pool.get().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        let row = Query::new("SELECT @@VERSION as version")
            .query(&mut *client)
            .await
            .map_err(|e| AppError::new(format!("Connected, but failed to query server: {e}")))?
            .into_row()
            .await
            .map_err(|e| AppError::new(format!("Connected, but failed to query server: {e}")))?
            .ok_or_else(|| AppError::new("Connected, but the server returned no version."))?;
        row.get::<&str, _>("version")
            .map(|s| s.lines().next().unwrap_or(s).to_string())
            .ok_or_else(|| AppError::new("Connected, but failed to read server version."))
    }

    async fn list_tables(&self) -> Result<Vec<TableRef>, AppError> {
        let mut client = self.pool.get().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        metadata::list_tables(&mut client).await
    }

    async fn get_table_columns(&self, schema: &str, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
        validate_identifier(table)?;
        let schema = if schema.is_empty() { DEFAULT_SCHEMA } else { schema };
        let mut client = self.pool.get().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        metadata::get_table_columns(&mut client, schema, table).await
    }

    async fn list_indexes(&self, schema: &str, table: &str) -> Result<Vec<IndexInfo>, AppError> {
        validate_identifier(table)?;
        let schema = if schema.is_empty() { DEFAULT_SCHEMA } else { schema };
        let mut client = self.pool.get().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        metadata::list_indexes(&mut client, schema, table).await
    }

    async fn list_constraints(&self, schema: &str, table: &str) -> Result<Vec<ConstraintInfo>, AppError> {
        validate_identifier(table)?;
        let schema = if schema.is_empty() { DEFAULT_SCHEMA } else { schema };
        let mut client = self.pool.get().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        metadata::list_constraints(&mut client, schema, table).await
    }

    async fn get_table_ddl(&self, schema: &str, table: &str) -> Result<String, AppError> {
        validate_identifier(table)?;
        let schema = if schema.is_empty() { DEFAULT_SCHEMA } else { schema };
        let mut client = self.pool.get().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        metadata::get_table_ddl(&mut client, schema, table).await
    }

    async fn render_ddl(&self, schema: &str, statements: &[DdlStatement]) -> Result<Vec<DdlPreview>, AppError> {
        for statement in statements {
            validate_ddl_statement(statement)?;
        }
        let schema = if schema.is_empty() { DEFAULT_SCHEMA } else { schema };
        let mut client = self.pool.get().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        ddl::render_all(&mut client, schema, statements).await
    }

    async fn execute_ddl(&self, schema: &str, statements: &[DdlStatement]) -> Result<DdlBatchResult, AppError> {
        for statement in statements {
            validate_ddl_statement(statement)?;
        }
        let schema = if schema.is_empty() { DEFAULT_SCHEMA } else { schema };
        let mut client = self.pool.get().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        ddl::execute_all(&mut client, schema, statements).await
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
        validate_identifier(table)?;
        for filter in filters {
            validate_identifier(&filter.column)?;
        }
        for sort in sort {
            validate_identifier(&sort.column)?;
        }
        let schema = if schema.is_empty() { DEFAULT_SCHEMA } else { schema };
        let limit = limit.clamp(1, MAX_PAGE_SIZE);
        let offset = offset.max(0);

        let mut client = self.pool.get().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        executor::fetch_rows(&mut client, schema, table, limit, offset, filters, sort).await
    }

    async fn count_table_rows(&self, schema: &str, table: &str, filters: &[TableFilter]) -> Result<u64, AppError> {
        validate_identifier(table)?;
        for filter in filters {
            validate_identifier(&filter.column)?;
        }
        let schema = if schema.is_empty() { DEFAULT_SCHEMA } else { schema };
        let mut client = self.pool.get().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        executor::count_rows(&mut client, schema, table, filters).await
    }

    async fn update_json_cell(
        &self,
        schema: &str,
        table: &str,
        row: &HashMap<String, JsonValue>,
        column: &str,
        value: &JsonValue,
    ) -> Result<(), AppError> {
        validate_identifier(table)?;
        validate_identifier(column)?;
        let schema = if schema.is_empty() { DEFAULT_SCHEMA } else { schema };
        let mut client = self.pool.get().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        let columns = metadata::get_table_columns(&mut client, schema, table).await?;
        let pk_values = pk_values_from_row(&columns, row)?;
        executor::update_json_cell(&mut client, schema, table, &pk_values, column, value).await
    }

    async fn update_cell_text(
        &self,
        schema: &str,
        table: &str,
        row: &HashMap<String, JsonValue>,
        column: &str,
        new_value: Option<&str>,
    ) -> Result<(), AppError> {
        validate_identifier(table)?;
        validate_identifier(column)?;
        let schema = if schema.is_empty() { DEFAULT_SCHEMA } else { schema };
        let mut client = self.pool.get().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        let columns = metadata::get_table_columns(&mut client, schema, table).await?;
        if !columns.iter().any(|c| c.name == column) {
            return Err(AppError::new(format!("Unknown column '{column}'.")));
        }
        let pk_values = pk_values_from_row(&columns, row)?;
        executor::update_cell_text(&mut client, schema, table, &pk_values, column, new_value).await
    }

    async fn delete_rows(&self, schema: &str, table: &str, rows: &[HashMap<String, JsonValue>]) -> Result<u64, AppError> {
        validate_identifier(table)?;
        if rows.is_empty() {
            return Ok(0);
        }
        let schema = if schema.is_empty() { DEFAULT_SCHEMA } else { schema };
        let mut client = self.pool.get().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        let columns = metadata::get_table_columns(&mut client, schema, table).await?;
        let mut rows_pk_values = Vec::with_capacity(rows.len());
        for row in rows {
            rows_pk_values.push(pk_values_from_row(&columns, row)?);
        }
        executor::delete_rows(&mut client, schema, table, &rows_pk_values).await
    }

    async fn insert_row(&self, schema: &str, table: &str, values: &HashMap<String, JsonValue>) -> Result<(), AppError> {
        validate_identifier(table)?;
        let schema = if schema.is_empty() { DEFAULT_SCHEMA } else { schema };
        let mut insert_values = Vec::with_capacity(values.len());
        for (column, value) in values {
            let Some(text) = json_to_insert_text(value) else { continue };
            validate_identifier(column)?;
            insert_values.push((column.clone(), text));
        }
        let mut client = self.pool.get().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        executor::insert_row(&mut client, schema, table, &insert_values).await
    }

    async fn execute_query(&self, sql: &str, offset: u64, limit: u64) -> Result<RawQueryResult, AppError> {
        if sql.trim().is_empty() {
            return Err(AppError::new("Cannot execute an empty query."));
        }
        let mut client = self.pool.get().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        executor::execute_query(&mut client, sql, offset, limit).await
    }

    async fn execute_query_for_tab(&self, tab_id: &str, sql: &str, offset: u64, limit: u64) -> Result<RawQueryResult, AppError> {
        if sql.trim().is_empty() {
            return Err(AppError::new("Cannot execute an empty query."));
        }

        let reserved = self.reserved.lock().unwrap().remove(tab_id);
        match reserved {
            Some(mut client) => {
                let result = self.execute_query_trackable(tab_id, &mut client, sql, offset, limit).await;
                self.reserved.lock().unwrap().insert(tab_id.to_string(), client);
                result
            }
            None => {
                let mut client = self.pool.get().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
                self.execute_query_trackable(tab_id, &mut client, sql, offset, limit).await
            }
        }
    }

    // `begin_transaction`/`commit_transaction`/`rollback_transaction`/
    // `cancel_query` all run through `Client::simple_query`, never
    // `Query::execute` — see `mssql::ddl::execute_all`'s doc comment for
    // why: `Query::execute` runs each call as its own isolated
    // `sp_executesql` RPC batch, which drops transaction state between
    // calls on the same connection even though nothing else touched it.
    async fn begin_transaction(&self, tab_id: &str) -> Result<(), AppError> {
        if self.reserved.lock().unwrap().contains_key(tab_id) {
            return Err(AppError::new("This tab already has an open transaction."));
        }
        let mut client = self.pool.get().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        client.simple_query("BEGIN TRANSACTION").await.map_err(|e| AppError::new(e.to_string()))?;
        self.reserved.lock().unwrap().insert(tab_id.to_string(), client);
        Ok(())
    }

    async fn commit_transaction(&self, tab_id: &str) -> Result<(), AppError> {
        let mut client = self.reserved.lock().unwrap().remove(tab_id).ok_or_else(|| AppError::new("This tab has no open transaction to commit."))?;
        client.simple_query("COMMIT TRANSACTION").await.map_err(|e| AppError::new(e.to_string()))?;
        Ok(())
    }

    async fn rollback_transaction(&self, tab_id: &str) -> Result<(), AppError> {
        let mut client = self.reserved.lock().unwrap().remove(tab_id).ok_or_else(|| AppError::new("This tab has no open transaction to roll back."))?;
        client.simple_query("ROLLBACK TRANSACTION").await.map_err(|e| AppError::new(e.to_string()))?;
        Ok(())
    }

    fn has_active_transaction(&self, tab_id: &str) -> bool {
        self.reserved.lock().unwrap().contains_key(tab_id)
    }

    async fn cancel_query(&self, tab_id: &str) -> Result<(), AppError> {
        let spid = self
            .running_spids
            .lock()
            .unwrap()
            .get(tab_id)
            .copied()
            .ok_or_else(|| AppError::new("No query is currently running on this tab."))?;

        let mut client = self.pool.get().await.map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
        client.simple_query(format!("KILL {spid}")).await.map_err(|e| AppError::new(e.to_string()))?;
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
    }

    #[test]
    fn rejects_sql_injection_attempts() {
        assert!(validate_identifier("users; drop table users;--").is_err());
        assert!(validate_identifier("users] OR [1]=[1").is_err());
        assert!(validate_identifier("users.other").is_err());
    }
}
