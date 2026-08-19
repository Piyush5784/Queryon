use crate::domain::driver::DatabaseDriver;
use crate::error::AppError;

use super::models::{ColumnInfo, TableRef};

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
