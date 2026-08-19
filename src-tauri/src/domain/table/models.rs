use serde::Serialize;
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
}
