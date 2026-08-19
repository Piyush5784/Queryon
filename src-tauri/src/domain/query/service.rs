use std::time::Instant;

use deadpool_postgres::Pool;

use crate::error::AppError;
use crate::infrastructure::postgres::executor;

use super::models::QueryResult;

const MAX_RESULT_ROWS: usize = 5000;

pub async fn execute_query(pool: &Pool, sql: &str) -> Result<QueryResult, AppError> {
    if sql.trim().is_empty() {
        return Err(AppError::new("Cannot execute an empty query."));
    }

    let client = pool
        .get()
        .await
        .map_err(|e| AppError::new(format!("Connection lost: {e}")))?;

    let start = Instant::now();
    let result = executor::execute_query(&client, sql, MAX_RESULT_ROWS).await?;
    let duration_ms = start.elapsed().as_millis() as u32;

    Ok(match result {
        executor::RawQueryResult::Rows { columns, rows, row_count } => {
            let truncated = row_count > MAX_RESULT_ROWS;
            let encoded_rows = rows
                .into_iter()
                .map(|row| row.into_iter().map(executor::encode_cell).collect())
                .collect();
            QueryResult::Rows {
                columns,
                rows: encoded_rows,
                row_count: row_count as u32,
                truncated,
                duration_ms,
            }
        }
        executor::RawQueryResult::Affected { row_count } => {
            QueryResult::Affected { row_count: row_count as u32, duration_ms }
        }
    })
}
