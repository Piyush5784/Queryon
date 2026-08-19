use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TableRef {
    pub schema: String,
    pub name: String,
    pub kind: String,
    pub estimated_rows: f64,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub default: Option<String>,
    pub is_primary_key: bool,
    pub ordinal_position: i32,
}
