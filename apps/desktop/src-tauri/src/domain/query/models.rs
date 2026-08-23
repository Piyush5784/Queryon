use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use specta::Type;

pub fn encode_cell(value: JsonValue) -> String {
    serde_json::to_string(&value).unwrap_or_else(|_| "null".to_string())
}

pub enum RawQueryResult {
    Rows {
        columns: Vec<String>,
        rows: Vec<Vec<JsonValue>>,
        total_row_count: Option<usize>,
    },
    Affected {
        row_count: u64,
    },
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum QueryResult {
    #[serde(rename_all = "camelCase")]
    Rows {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
        total_row_count: Option<u32>,
        duration_ms: u32,
    },
    #[serde(rename_all = "camelCase")]
    Affected {
        row_count: u32,
        duration_ms: u32,
    },
}

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
