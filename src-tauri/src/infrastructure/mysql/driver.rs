use std::collections::HashMap;

use serde_json::Value as JsonValue;
use sqlx::mysql::MySqlPool;
use sqlx::Row;

use crate::domain::driver::DatabaseDriver;
use crate::domain::query::RawQueryResult;
use crate::domain::schema::{
    ColumnInfo, ConstraintInfo, DdlBatchResult, DdlPreview, DdlStatement, IndexInfo, TableRef,
};
use crate::domain::table::{TableFilter, TableRowsResult, TableSort};
use crate::error::AppError;

use super::{ddl, executor, metadata};

const MAX_PAGE_SIZE: i64 = 10_000;

pub struct MySqlDriver {
    pool: MySqlPool,
    database: String,
}

impl MySqlDriver {
    pub fn new(pool: MySqlPool, database: String) -> Self {
        Self { pool, database }
    }
}

/// Converts one insert-row value to its text representation. Returns
/// `None` for `Null`, meaning the column is omitted from the insert
/// entirely so its table default (or nullability) applies, rather than
/// binding an explicit SQL `NULL` that would override a `DEFAULT`.
fn json_to_insert_text(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::Null => None,
        JsonValue::String(s) => Some(s.clone()),
        JsonValue::Number(n) => Some(n.to_string()),
        JsonValue::Bool(b) => Some(b.to_string()),
        other => Some(other.to_string()),
    }
}

fn validate_identifier(ident: &str) -> Result<(), AppError> {
    let mut chars = ident.chars();
    let first_ok = chars
        .next()
        .map(|c| c.is_ascii_alphabetic() || c == '_')
        .unwrap_or(false);
    let rest_ok = chars.all(|c| c.is_ascii_alphanumeric() || c == '_');

    if ident.is_empty() || !first_ok || !rest_ok {
        return Err(AppError::new(format!("Invalid identifier: {ident}")));
    }
    Ok(())
}

/// Validates every *identifier* field on a `DdlStatement` before it
/// reaches `ddl::render`/`ddl::execute_all`. See the identical function
/// in `infrastructure/postgres/driver.rs` for why `data_type` and
/// `check_expression` are deliberately not validated here.
fn validate_ddl_statement(statement: &DdlStatement) -> Result<(), AppError> {
    match statement {
        DdlStatement::AddColumn { table, column } => {
            validate_identifier(table)?;
            validate_identifier(&column.name)
        }
        DdlStatement::DropColumn { table, column } => {
            validate_identifier(table)?;
            validate_identifier(column)
        }
        DdlStatement::AlterColumn { table, edit } => {
            validate_identifier(table)?;
            validate_identifier(&edit.current_name)?;
            validate_identifier(&edit.column.name)
        }
        DdlStatement::AddIndex { table, index } => {
            validate_identifier(table)?;
            validate_identifier(&index.name)?;
            for col in &index.columns {
                validate_identifier(col)?;
            }
            Ok(())
        }
        DdlStatement::DropIndex { table, index } => {
            validate_identifier(table)?;
            validate_identifier(index)
        }
        DdlStatement::AddConstraint { table, constraint } => {
            validate_identifier(table)?;
            validate_identifier(&constraint.name)?;
            for col in &constraint.columns {
                validate_identifier(col)?;
            }
            if let Some(ref_table) = &constraint.referenced_table {
                validate_identifier(ref_table)?;
            }
            for col in &constraint.referenced_columns {
                validate_identifier(col)?;
            }
            Ok(())
        }
        DdlStatement::DropConstraint { table, constraint } => {
            validate_identifier(table)?;
            validate_identifier(constraint)
        }
    }
}

fn pk_values_from_row(
    columns: &[ColumnInfo],
    row: &HashMap<String, JsonValue>,
) -> Result<Vec<(String, JsonValue)>, AppError> {
    let pk_columns: Vec<&str> = columns
        .iter()
        .filter(|c| c.is_primary_key)
        .map(|c| c.name.as_str())
        .collect();

    if pk_columns.is_empty() {
        return Err(AppError::new(
            "This table has no primary key — cannot safely target a single row to update.",
        ));
    }

    let mut pk_values = Vec::with_capacity(pk_columns.len());
    for pk_col in &pk_columns {
        let value = row
            .get(*pk_col)
            .ok_or_else(|| AppError::new(format!("Missing primary key value for '{pk_col}'.")))?;
        pk_values.push((pk_col.to_string(), value.clone()));
    }
    Ok(pk_values)
}

#[async_trait::async_trait]
impl DatabaseDriver for MySqlDriver {
    async fn server_version(&self) -> Result<String, AppError> {
        let row = sqlx::query("select version()")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AppError::new(format!("Connected, but failed to query server: {e}")))?;
        row.try_get::<String, _>(0)
            .map_err(|e| AppError::new(format!("Connected, but failed to read server version: {e}")))
    }

    async fn list_tables(&self) -> Result<Vec<TableRef>, AppError> {
        metadata::list_tables(&self.pool, &self.database).await
    }

    async fn get_table_columns(&self, schema: &str, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
        metadata::get_table_columns(&self.pool, schema, table).await
    }

    async fn list_indexes(&self, schema: &str, table: &str) -> Result<Vec<IndexInfo>, AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;
        metadata::list_indexes(&self.pool, schema, table).await
    }

    async fn list_constraints(&self, schema: &str, table: &str) -> Result<Vec<ConstraintInfo>, AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;
        metadata::list_constraints(&self.pool, schema, table).await
    }

    async fn render_ddl(&self, schema: &str, statements: &[DdlStatement]) -> Result<Vec<DdlPreview>, AppError> {
        validate_identifier(schema)?;
        for statement in statements {
            validate_ddl_statement(statement)?;
        }
        ddl::render_all(&self.pool, schema, statements).await
    }

    async fn execute_ddl(
        &self,
        schema: &str,
        statements: &[DdlStatement],
    ) -> Result<DdlBatchResult, AppError> {
        validate_identifier(schema)?;
        for statement in statements {
            validate_ddl_statement(statement)?;
        }
        ddl::execute_all(&self.pool, schema, statements).await
    }

    async fn get_table_ddl(&self, schema: &str, table: &str) -> Result<String, AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;
        metadata::get_table_ddl(&self.pool, table).await
    }

    async fn fetch_table_rows(
        &self,
        schema: &str,
        table: &str,
        limit: i64,
        offset: i64,
        filters: &[TableFilter],
        sort: &[TableSort],
    ) -> Result<TableRowsResult, AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;
        for filter in filters {
            validate_identifier(&filter.column)?;
        }
        for sort in sort {
            validate_identifier(&sort.column)?;
        }

        let limit = limit.clamp(1, MAX_PAGE_SIZE);
        let offset = offset.max(0);

        executor::fetch_rows(&self.pool, schema, table, limit, offset, filters, sort).await
    }

    async fn count_table_rows(
        &self,
        schema: &str,
        table: &str,
        filters: &[TableFilter],
    ) -> Result<u64, AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;
        for filter in filters {
            validate_identifier(&filter.column)?;
        }

        executor::count_rows(&self.pool, schema, table, filters).await
    }

    async fn update_json_cell(
        &self,
        schema: &str,
        table: &str,
        row: &HashMap<String, JsonValue>,
        column: &str,
        value: &JsonValue,
    ) -> Result<(), AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;
        validate_identifier(column)?;

        let columns = metadata::get_table_columns(&self.pool, schema, table).await?;
        let pk_values = pk_values_from_row(&columns, row)?;

        executor::update_json_cell(&self.pool, schema, table, &pk_values, column, value).await
    }

    async fn update_cell_text(
        &self,
        schema: &str,
        table: &str,
        row: &HashMap<String, JsonValue>,
        column: &str,
        new_value: Option<&str>,
    ) -> Result<(), AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;
        validate_identifier(column)?;

        let columns = metadata::get_table_columns(&self.pool, schema, table).await?;
        if !columns.iter().any(|c| c.name == column) {
            return Err(AppError::new(format!("Unknown column '{column}'.")));
        }

        let pk_values = pk_values_from_row(&columns, row)?;

        executor::update_cell_text(&self.pool, schema, table, &pk_values, column, new_value).await
    }

    async fn delete_rows(
        &self,
        schema: &str,
        table: &str,
        rows: &[HashMap<String, JsonValue>],
    ) -> Result<u64, AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;

        if rows.is_empty() {
            return Ok(0);
        }

        let columns = metadata::get_table_columns(&self.pool, schema, table).await?;

        let mut rows_pk_values = Vec::with_capacity(rows.len());
        for row in rows {
            rows_pk_values.push(pk_values_from_row(&columns, row)?);
        }

        executor::delete_rows(&self.pool, schema, table, &rows_pk_values).await
    }

    async fn insert_row(
        &self,
        schema: &str,
        table: &str,
        values: &HashMap<String, JsonValue>,
    ) -> Result<(), AppError> {
        validate_identifier(schema)?;
        validate_identifier(table)?;

        let mut insert_values = Vec::with_capacity(values.len());
        for (column, value) in values {
            let Some(text) = json_to_insert_text(value) else { continue };
            validate_identifier(column)?;
            insert_values.push((column.clone(), text));
        }

        executor::insert_row(&self.pool, schema, table, &insert_values).await
    }

    async fn execute_query(&self, sql: &str, max_rows: usize) -> Result<RawQueryResult, AppError> {
        if sql.trim().is_empty() {
            return Err(AppError::new("Cannot execute an empty query."));
        }
        executor::execute_query(&self.pool, sql, max_rows).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plain_identifiers() {
        assert!(validate_identifier("users").is_ok());
        assert!(validate_identifier("_private").is_ok());
        assert!(validate_identifier("order_items").is_ok());
        assert!(validate_identifier("Users2").is_ok());
    }

    #[test]
    fn rejects_empty_identifier() {
        assert!(validate_identifier("").is_err());
    }

    #[test]
    fn rejects_identifier_starting_with_digit() {
        assert!(validate_identifier("2fast").is_err());
    }

    #[test]
    fn rejects_sql_injection_attempts() {
        assert!(validate_identifier("users; drop table users;--").is_err());
        assert!(validate_identifier("users` OR `1`=`1").is_err());
        assert!(validate_identifier("users--").is_err());
        assert!(validate_identifier("users.other").is_err());
        assert!(validate_identifier("users OR 1=1").is_err());
        assert!(validate_identifier("users)").is_err());
    }

    #[test]
    fn rejects_whitespace_and_quotes() {
        assert!(validate_identifier("my table").is_err());
        assert!(validate_identifier("`users`").is_err());
        assert!(validate_identifier("'users'").is_err());
    }
}
