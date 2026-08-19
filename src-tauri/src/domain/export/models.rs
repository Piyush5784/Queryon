use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ExportFormat {
    Csv,
    Json,
    Sql,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TableExportRequest {
    pub connection_id: String,
    pub schema: String,
    pub table: String,
    pub directory: String,
    pub file_name: String,
    pub format: ExportFormat,
    pub pretty_print: bool,
    pub chunk_size: u32,
    pub delete_on_abort: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RowsExportRequest {
    pub columns: Vec<String>,
    /// Each cell already JSON-encoded as a string, same convention as
    /// every other row payload crossing the IPC boundary (see
    /// `TableRowsResult`'s doc comment).
    pub rows: Vec<Vec<String>>,
    pub directory: String,
    pub file_name: String,
    pub format: ExportFormat,
    pub pretty_print: bool,
    pub delete_on_abort: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ExportEvent {
    #[serde(rename_all = "camelCase")]
    Progress { job_id: String, rows_written: u64, total_rows: Option<u64> },
    #[serde(rename_all = "camelCase")]
    Done { job_id: String, rows_written: u64, path: String },
    #[serde(rename_all = "camelCase")]
    Cancelled { job_id: String },
    #[serde(rename_all = "camelCase")]
    Error { job_id: String, message: String },
}

impl ExportEvent {
    pub fn job_id(&self) -> &str {
        match self {
            ExportEvent::Progress { job_id, .. }
            | ExportEvent::Done { job_id, .. }
            | ExportEvent::Cancelled { job_id, .. }
            | ExportEvent::Error { job_id, .. } => job_id,
        }
    }
}
