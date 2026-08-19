use std::collections::HashMap;

use serde_json::Value as JsonValue;

use crate::domain::query::RawQueryResult;
use crate::domain::schema::{ColumnInfo, TableRef};
use crate::domain::table::TableRowsResult;
use crate::error::AppError;

/// One implementation per supported database engine (Postgres, MySQL, ...).
/// `commands/` and `domain/*/service.rs` talk to a connection only through
/// this trait, so adding an engine never requires an engine match statement
/// outside the connect step (`domain/connection/service.rs`).
#[async_trait::async_trait]
pub trait DatabaseDriver: Send + Sync {
    async fn server_version(&self) -> Result<String, AppError>;

    async fn list_tables(&self) -> Result<Vec<TableRef>, AppError>;

    async fn get_table_columns(&self, schema: &str, table: &str) -> Result<Vec<ColumnInfo>, AppError>;

    async fn fetch_table_rows(
        &self,
        schema: &str,
        table: &str,
        limit: i64,
        offset: i64,
    ) -> Result<TableRowsResult, AppError>;

    async fn update_json_cell(
        &self,
        schema: &str,
        table: &str,
        row: &HashMap<String, JsonValue>,
        column: &str,
        value: &JsonValue,
    ) -> Result<(), AppError>;

    async fn update_cell_text(
        &self,
        schema: &str,
        table: &str,
        row: &HashMap<String, JsonValue>,
        column: &str,
        new_value: Option<&str>,
    ) -> Result<(), AppError>;

    async fn delete_rows(
        &self,
        schema: &str,
        table: &str,
        rows: &[HashMap<String, JsonValue>],
    ) -> Result<u64, AppError>;

    async fn execute_query(&self, sql: &str, max_rows: usize) -> Result<RawQueryResult, AppError>;
}
