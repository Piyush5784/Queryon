use serde::Serialize;
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRowsResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<JsonValue>>,
    pub row_count: usize,
    pub has_more: bool,
}
