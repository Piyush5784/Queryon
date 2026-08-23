use std::collections::HashMap;

use serde_json::Value as JsonValue;

use crate::domain::query::RawQueryResult;
use crate::domain::schema::{
    ColumnInfo, ConstraintInfo, DdlBatchResult, DdlPreview, DdlStatement, IndexInfo, TableRef,
};
use crate::domain::table::{TableFilter, TableRowsResult, TableSort};
use crate::error::AppError;

#[async_trait::async_trait]
pub trait DatabaseDriver: Send + Sync {
    async fn server_version(&self) -> Result<String, AppError>;

    async fn list_tables(&self) -> Result<Vec<TableRef>, AppError>;

    async fn get_table_columns(
        &self,
        schema: &str,
        table: &str,
    ) -> Result<Vec<ColumnInfo>, AppError>;

    async fn list_indexes(&self, schema: &str, table: &str) -> Result<Vec<IndexInfo>, AppError>;

    async fn list_constraints(
        &self,
        schema: &str,
        table: &str,
    ) -> Result<Vec<ConstraintInfo>, AppError>;

    async fn get_table_ddl(&self, schema: &str, table: &str) -> Result<String, AppError>;
    async fn render_ddl(
        &self,
        schema: &str,
        statements: &[DdlStatement],
    ) -> Result<Vec<DdlPreview>, AppError>;

    async fn execute_ddl(
        &self,
        schema: &str,
        statements: &[DdlStatement],
    ) -> Result<DdlBatchResult, AppError>;

    async fn fetch_table_rows(
        &self,
        schema: &str,
        table: &str,
        limit: i64,
        offset: i64,
        filters: &[TableFilter],
        sort: &[TableSort],
    ) -> Result<TableRowsResult, AppError>;

    async fn count_table_rows(
        &self,
        schema: &str,
        table: &str,
        filters: &[TableFilter],
    ) -> Result<u64, AppError>;

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

    async fn insert_row(
        &self,
        schema: &str,
        table: &str,
        values: &HashMap<String, JsonValue>,
    ) -> Result<(), AppError>;

    async fn execute_query_for_tab(
        &self,
        tab_id: &str,
        sql: &str,
        offset: u64,
        limit: u64,
    ) -> Result<RawQueryResult, AppError>;
    async fn execute_query(&self, sql: &str, offset: u64, limit: u64) -> Result<RawQueryResult, AppError>;

    async fn cancel_query(&self, tab_id: &str) -> Result<(), AppError>;

    async fn begin_transaction(&self, tab_id: &str) -> Result<(), AppError>;

    async fn commit_transaction(&self, tab_id: &str) -> Result<(), AppError>;

    async fn rollback_transaction(&self, tab_id: &str) -> Result<(), AppError>;

    fn has_active_transaction(&self, tab_id: &str) -> bool;
}
