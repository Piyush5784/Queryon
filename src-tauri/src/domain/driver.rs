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

    /// Renders each statement's SQL without running it — the preview
    /// step the frontend shows before "run" (see Phase-3 doc).
    /// Non-destructive (never writes), but engines may need a read
    /// round-trip to render correctly — e.g. MySQL's `DROP CONSTRAINT`
    /// needs to look up the constraint's kind first (see
    /// `infrastructure/mysql/ddl.rs`) — so this is async.
    async fn render_ddl(
        &self,
        schema: &str,
        statements: &[DdlStatement],
    ) -> Result<Vec<DdlPreview>, AppError>;

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

    /// Runs `sql` against the connection reserved for `tab_id` if one
    /// exists (see `begin_transaction`), otherwise behaves exactly like
    /// `execute_query` — grabs any pooled connection for the one call.
    /// The SQL editor always calls this rather than `execute_query`
    /// directly; whether a tab has a reserved session is invisible to
    /// the caller past this one branch.
    ///
    /// Only ever fetches `limit` rows starting at `offset` — for a
    /// SELECT-shaped `sql` this wraps it as `SELECT * FROM (sql) AS q
    /// LIMIT/OFFSET` and separately counts the total with `SELECT
    /// COUNT(*) FROM (sql) AS q`, so paging to a later `offset` re-runs
    /// the query against the database rather than reading from anything
    /// cached in memory. See `RawQueryResult::Rows::total_row_count`
    /// for what happens when `sql` can't be wrapped this way.
    async fn execute_query_for_tab(
        &self,
        tab_id: &str,
        sql: &str,
        offset: u64,
        limit: u64,
    ) -> Result<RawQueryResult, AppError>;

    /// Same paging behavior as `execute_query_for_tab`, without a tab's
    /// reserved connection — always grabs a pooled connection for the
    /// one call.
    async fn execute_query(&self, sql: &str, offset: u64, limit: u64) -> Result<RawQueryResult, AppError>;

    /// Cancels the query currently running on `tab_id`'s connection, if
    /// any — a query's connection records its own backend
    /// identifier/PID when it starts (see `execute_query_for_tab`), and
    /// this looks that up and sends the engine's cancel signal on a
    /// separate connection, since the one running the query is busy and
    /// can't cancel itself. Errors if no query is currently running on
    /// this tab, or if the cancel signal can't be delivered (e.g. the
    /// query already finished on its own).
    async fn cancel_query(&self, tab_id: &str) -> Result<(), AppError>;

    /// Reserves one dedicated connection from the pool for `tab_id` and
    /// runs `BEGIN` on it. Every later `execute_query_for_tab` call for
    /// this same `tab_id` reuses that connection until
    /// `commit_transaction`/`rollback_transaction` releases it back to
    /// the pool — this is what makes manual-commit mode's "run several
    /// statements, then decide" workflow possible: a SQL transaction is
    /// scoped to one connection, and a pooled `execute_query` call
    /// grabbing a different connection each time can never honor that.
    /// Errors if `tab_id` already has a reserved connection.
    async fn begin_transaction(&self, tab_id: &str) -> Result<(), AppError>;

    /// Runs `COMMIT` on `tab_id`'s reserved connection and releases it
    /// back to the pool. Errors if `tab_id` has no reserved connection.
    async fn commit_transaction(&self, tab_id: &str) -> Result<(), AppError>;

    /// Runs `ROLLBACK` on `tab_id`'s reserved connection and releases it
    /// back to the pool. Errors if `tab_id` has no reserved connection.
    async fn rollback_transaction(&self, tab_id: &str) -> Result<(), AppError>;

    /// Whether `tab_id` currently has a reserved connection with an open
    /// transaction. Used to restore UI state (e.g. after a frontend
    /// reload) rather than trusting frontend-only state.
    fn has_active_transaction(&self, tab_id: &str) -> bool;
}
