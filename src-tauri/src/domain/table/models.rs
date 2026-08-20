use serde::{Deserialize, Serialize};
use specta::Type;

/// Each cell is a JSON-encoded string (e.g. `"\"hello\""`, `"42"`,
/// `"null"`) rather than a structured value. specta cannot export
/// `serde_json::Value` without recursing infinitely on its own
/// Array/Object variants (a known limitation, see
/// infrastructure/postgres/executor.rs's `row_value_to_json`), so
/// cells are encoded to strings on the Rust side and `JSON.parse`d
/// uniformly on the frontend instead.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TableRowsResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub row_count: u32,
    pub has_more: bool,
    pub duration_ms: u32,
}

#[derive(Debug, Clone, Copy, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterOrEquals,
    LessThan,
    LessOrEquals,
    /// Raw SQL `LIKE` pattern — the caller supplies its own `%`/`_` wildcards.
    Like,
    /// Raw SQL case-insensitive `ILIKE` pattern (Postgres) / `LIKE` on a
    /// case-insensitive collation (MySQL) — same wildcard convention as `Like`.
    Ilike,
    NotLike,
    /// `value` is a comma-separated list; matches if the column equals any of them.
    In,
    IsNull,
    IsNotNull,
}

/// A single column filter applied as a WHERE clause in `fetch_table_rows`.
/// `value` is ignored for `IsNull`/`IsNotNull`.
#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TableFilter {
    pub column: String,
    pub operator: FilterOperator,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SortDirection {
    Asc,
    Desc,
}

/// One column/direction pair in the ordered list `fetch_table_rows`
/// results are sorted by (applied left to right, like a SQL multi-column
/// `ORDER BY`).
#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TableSort {
    pub column: String,
    pub direction: SortDirection,
}
