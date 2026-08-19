use std::time::Instant;

use deadpool_postgres::Pool;

use crate::error::{clean_postgres_error, AppError};
use crate::infrastructure::postgres::pool::build_pool;

use super::models::ConnectionProfile;

pub async fn open_pool_and_verify(profile: &ConnectionProfile) -> Result<(Pool, String), AppError> {
    let build_start = Instant::now();
    let pool = build_pool(profile)?;
    log::info!("db_connect: build_pool took {:?}", build_start.elapsed());

    let verify_start = Instant::now();
    let server_version = fetch_server_version(&pool).await?;
    log::info!("db_connect: fetch_server_version took {:?}", verify_start.elapsed());

    Ok((pool, server_version))
}

pub async fn fetch_server_version(pool: &Pool) -> Result<String, AppError> {
    let checkout_start = Instant::now();
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::new(format!("Could not connect: {}", clean_postgres_error(&e.to_string()))))?;
    log::info!(
        "fetch_server_version: pool checkout took {:?}",
        checkout_start.elapsed()
    );

    let query_start = Instant::now();
    let row = client
        .query_one("SHOW server_version", &[])
        .await
        .map_err(|e| AppError::new(format!("Connected, but failed to query server: {e}")))?;
    log::info!(
        "fetch_server_version: query took {:?}",
        query_start.elapsed()
    );

    Ok(row.get::<_, String>(0))
}
