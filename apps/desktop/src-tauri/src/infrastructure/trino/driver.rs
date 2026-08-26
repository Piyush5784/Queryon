use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::Value as JsonValue;
use trino_rust_client::client::Client;
use trino_rust_client::transaction::TransactionId;

use crate::domain::driver::DatabaseDriver;
use crate::domain::query::RawQueryResult;
use crate::domain::schema::{
    ColumnInfo, ConstraintInfo, DdlBatchResult, DdlPreview, DdlStatement, IndexInfo, TableRef,
};
use crate::domain::table::{TableFilter, TableRowsResult, TableSort};
use crate::error::AppError;

use super::{ddl, executor, metadata};

const MAX_PAGE_SIZE: i64 = 10_000;
const DEFAULT_SCHEMA: &str = "default";

pub struct TrinoDriver {
    client: Client,
    query_lock: tokio::sync::Mutex<()>,
    transactions: Mutex<HashMap<String, TransactionId>>,
}

impl TrinoDriver {
    pub fn new(client: Client) -> Self {
        Self { client, query_lock: tokio::sync::Mutex::new(()), transactions: Mutex::new(HashMap::new()) }
    }

    fn effective_schema(schema: &str) -> &str {
        if schema.is_empty() { DEFAULT_SCHEMA } else { schema }
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
            validate_identifier(&index.name)
        }
        DdlStatement::DropIndex { table, index } => {
            validate_identifier(table)?;
            validate_identifier(index)
        }
        DdlStatement::AddConstraint { table, constraint } => {
            validate_identifier(table)?;
            validate_identifier(&constraint.name)
        }
        DdlStatement::DropConstraint { table, constraint } => {
            validate_identifier(table)?;
            validate_identifier(constraint)
        }
    }
}

#[async_trait::async_trait]
impl DatabaseDriver for TrinoDriver {
    async fn server_version(&self) -> Result<String, AppError> {
        let dataset = self
            .client
            .get_all::<trino_rust_client::Row>("select version()")
            .await
            .map_err(|e| AppError::new(format!("Connected, but failed to query server: {}", crate::error::describe_trino_error(&e))))?;
        let row = dataset
            .into_vec()
            .into_iter()
            .next()
            .ok_or_else(|| AppError::new("Connected, but the server returned no version."))?;
        let values = row.into_json();
        match values.first() {
            Some(JsonValue::String(s)) => Ok(format!("Trino {s}")),
            Some(other) => Ok(format!("Trino {other}")),
            None => Err(AppError::new("Connected, but failed to read server version.")),
        }
    }

    async fn list_tables(&self) -> Result<Vec<TableRef>, AppError> {
        metadata::list_tables(&self.client, DEFAULT_SCHEMA).await
    }

    async fn get_table_columns(&self, schema: &str, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
        validate_identifier(table)?;
        metadata::get_table_columns(&self.client, Self::effective_schema(schema), table).await
    }

    async fn list_indexes(&self, schema: &str, table: &str) -> Result<Vec<IndexInfo>, AppError> {
        validate_identifier(table)?;
        metadata::list_indexes(&self.client, Self::effective_schema(schema), table).await
    }

    async fn list_constraints(&self, schema: &str, table: &str) -> Result<Vec<ConstraintInfo>, AppError> {
        validate_identifier(table)?;
        metadata::list_constraints(&self.client, Self::effective_schema(schema), table).await
    }

    async fn get_table_ddl(&self, schema: &str, table: &str) -> Result<String, AppError> {
        validate_identifier(table)?;
        metadata::get_table_ddl(&self.client, Self::effective_schema(schema), table).await
    }

    async fn render_ddl(&self, schema: &str, statements: &[DdlStatement]) -> Result<Vec<DdlPreview>, AppError> {
        for statement in statements {
            validate_ddl_statement(statement)?;
        }
        ddl::render_all(Self::effective_schema(schema), statements).await
    }

    async fn execute_ddl(&self, schema: &str, statements: &[DdlStatement]) -> Result<DdlBatchResult, AppError> {
        for statement in statements {
            validate_ddl_statement(statement)?;
        }
        ddl::execute_all(&self.client, Self::effective_schema(schema), statements).await
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
        let limit = limit.clamp(1, MAX_PAGE_SIZE);
        let offset = offset.max(0);
        executor::fetch_rows(&self.client, Self::effective_schema(schema), table, limit, offset, filters, sort).await
    }

    async fn count_table_rows(&self, schema: &str, table: &str, filters: &[TableFilter]) -> Result<u64, AppError> {
        validate_identifier(table)?;
        for filter in filters {
            validate_identifier(&filter.column)?;
        }
        executor::count_rows(&self.client, Self::effective_schema(schema), table, filters).await
    }

    async fn update_json_cell(
        &self,
        _schema: &str,
        _table: &str,
        _row: &HashMap<String, JsonValue>,
        _column: &str,
        _value: &JsonValue,
    ) -> Result<(), AppError> {
        Err(AppError::new(
            "Editing rows is not supported for Trino — most Trino connectors expose data as read-mostly external sources without row-level update support.",
        ))
    }

    async fn update_cell_text(
        &self,
        _schema: &str,
        _table: &str,
        _row: &HashMap<String, JsonValue>,
        _column: &str,
        _new_value: Option<&str>,
    ) -> Result<(), AppError> {
        Err(AppError::new(
            "Editing rows is not supported for Trino — most Trino connectors expose data as read-mostly external sources without row-level update support.",
        ))
    }

    async fn delete_rows(&self, _schema: &str, _table: &str, _rows: &[HashMap<String, JsonValue>]) -> Result<u64, AppError> {
        Err(AppError::new(
            "Deleting rows is not supported for Trino — most Trino connectors expose data as read-mostly external sources without row-level delete support.",
        ))
    }

    async fn insert_row(&self, schema: &str, table: &str, values: &HashMap<String, JsonValue>) -> Result<(), AppError> {
        validate_identifier(table)?;
        let mut insert_values = Vec::with_capacity(values.len());
        for (column, value) in values {
            validate_identifier(column)?;
            insert_values.push((column.clone(), value.clone()));
        }
        executor::insert_row(&self.client, Self::effective_schema(schema), table, &insert_values).await
    }

    async fn execute_query(&self, sql: &str, offset: u64, limit: u64) -> Result<RawQueryResult, AppError> {
        if sql.trim().is_empty() {
            return Err(AppError::new("Cannot execute an empty query."));
        }
        let _guard = self.query_lock.lock().await;
        executor::execute_query(&self.client, sql, offset, limit).await
    }

    async fn execute_query_for_tab(&self, tab_id: &str, sql: &str, offset: u64, limit: u64) -> Result<RawQueryResult, AppError> {
        if sql.trim().is_empty() {
            return Err(AppError::new("Cannot execute an empty query."));
        }
        let _guard = self.query_lock.lock().await;
        let tx_id = self.transactions.lock().unwrap().get(tab_id).cloned();
        if let Some(tx_id) = tx_id {
            self.client.set_transaction_id(tx_id).await;
        } else {
            self.client.set_transaction_id(TransactionId::NoTransaction).await;
        }
        let result = executor::execute_query(&self.client, sql, offset, limit).await;
        if self.transactions.lock().unwrap().contains_key(tab_id) {
            let current = self.client.transaction_id().await;
            self.transactions.lock().unwrap().insert(tab_id.to_string(), current);
        }
        result
    }

    async fn begin_transaction(&self, tab_id: &str) -> Result<(), AppError> {
        if self.transactions.lock().unwrap().contains_key(tab_id) {
            return Err(AppError::new("This tab already has an open transaction."));
        }
        let _guard = self.query_lock.lock().await;
        self.client.set_transaction_id(TransactionId::NoTransaction).await;
        self.client.begin_transaction().await.map_err(|e| AppError::new(crate::error::describe_trino_error(&e)))?;
        let tx_id = self.client.transaction_id().await;
        self.transactions.lock().unwrap().insert(tab_id.to_string(), tx_id);
        Ok(())
    }

    async fn commit_transaction(&self, tab_id: &str) -> Result<(), AppError> {
        let tx_id = self.transactions.lock().unwrap().remove(tab_id).ok_or_else(|| AppError::new("This tab has no open transaction to commit."))?;
        let _guard = self.query_lock.lock().await;
        self.client.set_transaction_id(tx_id).await;
        self.client.commit().await.map_err(|e| AppError::new(crate::error::describe_trino_error(&e)))
    }

    async fn rollback_transaction(&self, tab_id: &str) -> Result<(), AppError> {
        let tx_id = self.transactions.lock().unwrap().remove(tab_id).ok_or_else(|| AppError::new("This tab has no open transaction to roll back."))?;
        let _guard = self.query_lock.lock().await;
        self.client.set_transaction_id(tx_id).await;
        self.client.rollback().await.map_err(|e| AppError::new(crate::error::describe_trino_error(&e)))
    }

    fn has_active_transaction(&self, tab_id: &str) -> bool {
        self.transactions.lock().unwrap().contains_key(tab_id)
    }

    async fn cancel_query(&self, _tab_id: &str) -> Result<(), AppError> {
        Err(AppError::new("Cancelling a running query is not supported for Trino."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plain_identifiers() {
        assert!(validate_identifier("orders").is_ok());
        assert!(validate_identifier("_private").is_ok());
        assert!(validate_identifier("order_items").is_ok());
    }

    #[test]
    fn rejects_sql_injection_attempts() {
        assert!(validate_identifier("orders; drop table orders;--").is_err());
        assert!(validate_identifier("orders\" OR \"1\"=\"1").is_err());
        assert!(validate_identifier("orders.other").is_err());
    }
}
