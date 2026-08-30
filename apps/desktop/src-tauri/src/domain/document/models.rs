use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseRef {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CollectionRef {
    pub name: String,
    pub estimated_count: f64,
    pub storage_size_bytes: f64,
    pub avg_document_size_bytes: f64,
    pub index_count: u32,
    pub total_index_size_bytes: f64,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSummary {
    pub id: String,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DocumentPage {
    pub documents: Vec<DocumentSummary>,
    pub has_more: bool,
    pub duration_ms: u32,
}
