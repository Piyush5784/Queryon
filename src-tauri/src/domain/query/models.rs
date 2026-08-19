use serde::Serialize;
use specta::Type;

/// See `TableRowsResult`'s doc comment: each cell is a JSON-encoded
/// string, not a structured value, working around specta's inability
/// to export `serde_json::Value` without infinite recursion.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum QueryResult {
    #[serde(rename_all = "camelCase")]
    Rows {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
        row_count: u32,
        truncated: bool,
        duration_ms: u32,
    },
    #[serde(rename_all = "camelCase")]
    Affected {
        row_count: u32,
        duration_ms: u32,
    },
}
