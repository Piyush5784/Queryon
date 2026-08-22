use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use specta::Type;

/// JSON-encodes a single cell value to a string. See `TableRowsResult`'s
/// doc comment for why cells cross the IPC boundary as strings rather
/// than structured `serde_json::Value`s. Shared across every engine's
/// `DatabaseDriver` impl.
pub fn encode_cell(value: JsonValue) -> String {
    serde_json::to_string(&value).unwrap_or_else(|_| "null".to_string())
}

/// The result of running arbitrary SQL, before cell values are
/// JSON-encoded to strings for the IPC boundary. Each `DatabaseDriver`
/// impl produces this from whatever wire format its engine uses.
pub enum RawQueryResult {
    Rows {
        columns: Vec<String>,
        /// Just this page's rows — the engine only ever fetches
        /// `limit` rows per call, via `LIMIT`/`OFFSET` on a wrapped
        /// subquery, never the full result set at once.
        rows: Vec<Vec<JsonValue>>,
        /// The query's total matching row count, from a separate
        /// `SELECT COUNT(*) FROM (<sql>) AS q`. `None` when the query
        /// couldn't be wrapped for counting/paging (rare — some
        /// statement shapes can't be used as a subquery) — in that
        /// case `rows` is simply everything the query returned, and
        /// there's no next page to fetch.
        total_row_count: Option<usize>,
    },
    Affected {
        row_count: u64,
    },
}

/// See `TableRowsResult`'s doc comment: each cell is a JSON-encoded
/// string, not a structured value, working around specta's inability
/// to export `serde_json::Value` without infinite recursion.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum QueryResult {
    #[serde(rename_all = "camelCase")]
    Rows {
        columns: Vec<String>,
        /// Just the first page — see `db_fetch_query_result_page` for
        /// the rest, which re-runs the query against the database with
        /// a different `OFFSET` rather than reading from anything
        /// cached in memory.
        rows: Vec<Vec<String>>,
        /// The query's total matching row count, when known (see
        /// `RawQueryResult::Rows::total_row_count`). `None` means this
        /// result can't be paginated — `rows` is everything there is.
        total_row_count: Option<u32>,
        duration_ms: u32,
    },
    #[serde(rename_all = "camelCase")]
    Affected {
        row_count: u32,
        duration_ms: u32,
    },
}

/// One page of query results, fetched fresh from the database with
/// `LIMIT`/`OFFSET` on the original query text — see
/// `db_fetch_query_result_page`.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct QueryResultPage {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub total_row_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SavedQuery {
    pub id: String,
    pub connection_id: String,
    pub title: String,
    pub sql: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum QueryHistoryStatus {
    Success,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct QueryHistoryEntry {
    pub id: String,
    pub connection_id: String,
    pub sql: String,
    pub status: QueryHistoryStatus,
    pub error_message: Option<String>,
    pub row_count: Option<u32>,
    pub duration_ms: u32,
    pub ran_at: String,
}
