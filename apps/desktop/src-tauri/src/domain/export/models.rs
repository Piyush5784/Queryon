use serde::{Deserialize, Serialize};
use specta::Type;

use crate::domain::table::{TableFilter, TableSort};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ExportFormat {
    Csv,
    Json,
    Sql,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TableExportRequest {
    pub connection_id: String,
    pub schema: String,
    pub table: String,
    /// Empty means "whole table" — every row, filters ignored. Non-empty
    /// means "filtered view" — the same filters currently applied in the
    /// grid, re-run against the database rather than limited to whatever
    /// page is loaded. Matches the two scopes Beekeeper Studio's table
    /// export menu offers ("Export whole table" / "Export filtered
    /// view") — see `TableTable.vue`'s `exportTable`/`exportFiltered`.
    pub filters: Vec<TableFilter>,
    pub sort: Vec<TableSort>,
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

/// Exports the full result of a query tab's SQL, re-run in chunks
/// straight from the database — never limited to whatever page happens
/// to be loaded in the results grid. Mirrors how Beekeeper Studio's
/// "Export to File" always re-runs the query rather than exporting the
/// rendered grid (see `TabQueryEditor.vue`'s `submitQueryToFile`).
#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct QueryExportRequest {
    pub connection_id: String,
    pub tab_id: String,
    pub sql: String,
    pub directory: String,
    pub file_name: String,
    pub format: ExportFormat,
    pub pretty_print: bool,
    pub chunk_size: u32,
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
