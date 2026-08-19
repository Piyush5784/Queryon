use std::time::Instant;

use deadpool_postgres::Pool;

use crate::error::AppError;
use crate::infrastructure::postgres::metadata;

use super::models::{ColumnInfo, TableRef};

pub async fn list_tables(pool: &Pool) -> Result<Vec<TableRef>, AppError> {
    let checkout_start = Instant::now();
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
    log::info!("list_tables: pool checkout took {:?}", checkout_start.elapsed());

    let query_start = Instant::now();
    let result = metadata::list_tables(&client).await;
    log::info!("list_tables: query took {:?}", query_start.elapsed());
    result
}

pub async fn get_table_columns(
    pool: &Pool,
    schema: &str,
    table: &str,
) -> Result<Vec<ColumnInfo>, AppError> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::new(format!("Connection lost: {e}")))?;
    metadata::get_table_columns(&client, schema, table).await
}
