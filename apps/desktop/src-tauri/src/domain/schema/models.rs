use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ConstraintKind {
    PrimaryKey,
    ForeignKey,
    Unique,
    Check,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ForeignKeyAction {
    NoAction,
    Restrict,
    Cascade,
    SetNull,
    SetDefault,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConstraintInfo {
    pub name: String,
    pub kind: ConstraintKind,
    pub columns: Vec<String>,
    pub referenced_table: Option<String>,
    pub referenced_columns: Vec<String>,
    pub on_update: Option<ForeignKeyAction>,
    pub on_delete: Option<ForeignKeyAction>,
    pub check_expression: Option<String>,
}
pub const AUTO_INCREMENT_TYPE: &str = "auto-increment";

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct NewColumn {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct NewIndex {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct NewConstraint {
    pub name: String,
    pub kind: ConstraintKind,
    pub columns: Vec<String>,
    pub referenced_table: Option<String>,
    pub referenced_columns: Vec<String>,
    pub on_update: Option<ForeignKeyAction>,
    pub on_delete: Option<ForeignKeyAction>,
    pub check_expression: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ColumnEdit {
    pub current_name: String,
    pub column: NewColumn,
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase", tag = "op")]
pub enum DdlStatement {
    CreateTable {
        table: String,
        columns: Vec<NewColumn>,
    },
    #[serde(rename_all = "camelCase")]
    RenameTable {
        table: String,
        new_name: String,
    },
    DropTable {
        table: String,
    },
    AddColumn {
        table: String,
        column: NewColumn,
    },
    DropColumn {
        table: String,
        column: String,
    },
    AlterColumn {
        table: String,
        edit: ColumnEdit,
    },
    AddIndex {
        table: String,
        index: NewIndex,
    },
    DropIndex {
        table: String,
        index: String,
    },
    AddConstraint {
        table: String,
        constraint: NewConstraint,
    },
    DropConstraint {
        table: String,
        constraint: String,
    },
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DdlPreview {
    pub sql: String,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DdlExecutionResult {
    pub sql: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DdlBatchResult {
    pub results: Vec<DdlExecutionResult>,
    pub rolled_back: bool,
}
