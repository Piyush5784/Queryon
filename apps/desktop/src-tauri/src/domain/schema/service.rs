use crate::domain::driver::DatabaseDriver;
use crate::error::AppError;

use super::models::{ColumnInfo, ConstraintInfo, DdlBatchResult, DdlPreview, DdlStatement, IndexInfo, TableRef};

pub async fn list_tables(driver: &dyn DatabaseDriver) -> Result<Vec<TableRef>, AppError> {
    driver.list_tables().await
}

pub async fn get_table_columns(
    driver: &dyn DatabaseDriver,
    schema: &str,
    table: &str,
) -> Result<Vec<ColumnInfo>, AppError> {
    driver.get_table_columns(schema, table).await
}

pub async fn list_indexes(
    driver: &dyn DatabaseDriver,
    schema: &str,
    table: &str,
) -> Result<Vec<IndexInfo>, AppError> {
    driver.list_indexes(schema, table).await
}

pub async fn list_constraints(
    driver: &dyn DatabaseDriver,
    schema: &str,
    table: &str,
) -> Result<Vec<ConstraintInfo>, AppError> {
    driver.list_constraints(schema, table).await
}

pub async fn get_table_ddl(
    driver: &dyn DatabaseDriver,
    schema: &str,
    table: &str,
) -> Result<String, AppError> {
    driver.get_table_ddl(schema, table).await
}

pub async fn render_ddl(
    driver: &dyn DatabaseDriver,
    schema: &str,
    statements: &[DdlStatement],
) -> Result<Vec<DdlPreview>, AppError> {
    driver.render_ddl(schema, statements).await
}

pub async fn execute_ddl(
    driver: &dyn DatabaseDriver,
    schema: &str,
    statements: &[DdlStatement],
) -> Result<DdlBatchResult, AppError> {
    driver.execute_ddl(schema, statements).await
}
