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

    async fn get_table_columns(&self, schema: &str, table: &str) -> Result<Vec<ColumnInfo>, AppError>;

    async fn list_indexes(&self, schema: &str, table: &str) -> Result<Vec<IndexInfo>, AppError>;

    async fn list_constraints(&self, schema: &str, table: &str) -> Result<Vec<ConstraintInfo>, AppError>;

    async fn get_table_ddl(&self, schema: &str, table: &str) -> Result<String, AppError>;

    /// Renders each statement's SQL without running it — the preview
    /// step the frontend shows before "run" (see Phase-3 doc).
    /// Non-destructive (never writes), but engines may need a read
    /// round-trip to render correctly — e.g. MySQL's `DROP CONSTRAINT`
    /// needs to look up the constraint's kind first (see
    /// `infrastructure/mysql/ddl.rs`) — so this is async.
    async fn render_ddl(&self, schema: &str, statements: &[DdlStatement]) -> Result<Vec<DdlPreview>, AppError>;

    /// Executes a batch of statements in the order given. Postgres runs
    /// the batch inside one transaction — any failure rolls back
    /// everything (`DdlBatchResult::rolled_back` is `true`). MySQL
    /// cannot offer this: DDL there auto-commits per statement, so
    /// execution stops at the first failure but earlier successful
    /// statements stay applied (`rolled_back` is always `false`). See
    /// each result's `success`/`error` for per-statement outcome.
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

    async fn execute_query(&self, sql: &str, max_rows: usize) -> Result<RawQueryResult, AppError>;
}
