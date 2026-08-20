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

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConstraintInfo {
    pub name: String,
    pub kind: ConstraintKind,
    pub columns: Vec<String>,
    /// Set only for `ForeignKey` — the referenced table and its columns,
    /// positionally matched to `columns`.
    pub referenced_table: Option<String>,
    pub referenced_columns: Vec<String>,
    /// Set only for `Check` — the constraint's boolean expression, as the
    /// engine reports it back (already-normalized text, not necessarily
    /// what the user originally typed).
    pub check_expression: Option<String>,
}

/// A new column's definition, as submitted from the frontend's "Add
/// column" form. `data_type` is a raw engine-dialect type string (e.g.
/// `text`, `varchar(255)`, `int8`) — not validated against a fixed enum,
/// since the set of valid types differs per engine and this app doesn't
/// maintain its own type catalog.
#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct NewColumn {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub default: Option<String>,
}

/// A new index's definition, as submitted from the frontend's "Add
/// index" form.
#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct NewIndex {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
}

/// A new constraint's definition, as submitted from the frontend's "Add
/// constraint" form. Which fields apply depends on `kind`, mirroring
/// `ConstraintInfo`'s own shape (see its doc comments for which fields
/// are set for which kind).
#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct NewConstraint {
    pub name: String,
    pub kind: ConstraintKind,
    pub columns: Vec<String>,
    pub referenced_table: Option<String>,
    pub referenced_columns: Vec<String>,
    pub check_expression: Option<String>,
}

/// An existing column's edited definition, as submitted from the
/// frontend's inline column editor. `current_name` locates the column;
/// `column` carries its full target shape (name, type, nullable,
/// default) — MySQL's `MODIFY COLUMN` must restate the whole definition
/// regardless of which fields actually changed, so this always carries
/// all of them rather than a sparse patch.
#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ColumnEdit {
    pub current_name: String,
    pub column: NewColumn,
}

/// One engine-agnostic schema-write operation. Each `DatabaseDriver`
/// implementation renders these into its own DDL dialect (see
/// `infrastructure/{postgres,mysql}/ddl.rs`) — the frontend and the
/// `domain` layer never construct raw SQL strings for writes.
///
/// `CreateTable` takes a full column list rather than decomposing into
/// per-column `AddColumn`s — a brand-new table is one `CREATE TABLE`
/// statement, not N `ALTER TABLE`s. Indexes/constraints on a new table
/// are staged as ordinary `AddIndex`/`AddConstraint` statements in the
/// same batch, executed after the `CreateTable` — this composes out of
/// the existing add machinery rather than duplicating it (see Phase-3
/// doc, milestone 7: "should mostly compose out of milestones 2-6").
#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase", tag = "op")]
pub enum DdlStatement {
    CreateTable { table: String, columns: Vec<NewColumn> },
    #[serde(rename_all = "camelCase")]
    RenameTable { table: String, new_name: String },
    DropTable { table: String },
    AddColumn { table: String, column: NewColumn },
    DropColumn { table: String, column: String },
    AlterColumn { table: String, edit: ColumnEdit },
    AddIndex { table: String, index: NewIndex },
    DropIndex { table: String, index: String },
    AddConstraint { table: String, constraint: NewConstraint },
    DropConstraint { table: String, constraint: String },
}

/// One rendered statement plus the raw SQL that will run for it — what
/// the frontend's DDL-preview step shows before the user confirms
/// execution (see Phase-3 doc, "generate DDL, show it, let you
/// review/edit it, then run it").
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DdlPreview {
    pub sql: String,
}

/// The result of running one `DdlStatement` within a batch.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DdlExecutionResult {
    pub sql: String,
    pub success: bool,
    pub error: Option<String>,
}

/// The result of running a batch of `DdlStatement`s together.
///
/// Postgres runs the whole batch inside one transaction: if any statement
/// fails, every statement in the batch is rolled back and `rolled_back`
/// is `true` — nothing in the batch is left applied.
///
/// MySQL cannot offer this guarantee — every DDL statement there
/// auto-commits immediately, so a failure can never undo earlier
/// statements in the same batch no matter what the client does. On
/// MySQL, execution simply stops at the first failure and `rolled_back`
/// is always `false`; statements before the failure stay applied.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DdlBatchResult {
    pub results: Vec<DdlExecutionResult>,
    pub rolled_back: bool,
}
