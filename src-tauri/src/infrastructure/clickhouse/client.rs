use serde::Deserialize;
use serde_json::Value as JsonValue;

use crate::error::AppError;

use super::pool::ClickHouseClient;

#[derive(Debug, Deserialize)]
struct JsonColumnMeta {
    name: String,
    #[serde(rename = "type")]
    type_name: String,
}

#[derive(Debug, Deserialize)]
struct JsonResponse {
    #[serde(default)]
    meta: Vec<JsonColumnMeta>,
    #[serde(default)]
    data: Vec<serde_json::Map<String, JsonValue>>,
    #[serde(default)]
    rows: usize,
}

pub struct QueryOutcome {
    pub columns: Vec<String>,
    pub column_types: Vec<String>,
    pub rows: Vec<serde_json::Map<String, JsonValue>>,
    pub row_count: usize,
}

fn clean_clickhouse_error(raw: &str) -> String {
    if let Some(rest) = raw.split("DB::Exception:").nth(1) {
        let message = rest.split(". (").next().unwrap_or(rest).trim();
        if !message.is_empty() {
            return message.to_string();
        }
    }
    raw.trim().to_string()
}

impl ClickHouseClient {
    pub async fn execute(&self, sql: &str) -> Result<(), AppError> {
        let response = self
            .http
            .post(&self.base_url)
            .query(&[
                ("user", self.user.as_str()),
                ("password", self.password.as_str()),
                ("database", self.database.as_str()),
            ])
            .body(sql.to_string())
            .send()
            .await
            .map_err(|e| AppError::new(format!("Connection to ClickHouse failed: {e}")))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::new(clean_clickhouse_error(&body)));
        }
        Ok(())
    }

    pub async fn query(&self, sql: &str) -> Result<QueryOutcome, AppError> {
        let body = format!("{} FORMAT JSON", sql.trim_end().trim_end_matches(';'));

        let response = self
            .http
            .post(&self.base_url)
            .query(&[
                ("user", self.user.as_str()),
                ("password", self.password.as_str()),
                ("database", self.database.as_str()),
            ])
            .body(body)
            .send()
            .await
            .map_err(|e| AppError::new(format!("Connection to ClickHouse failed: {e}")))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::new(clean_clickhouse_error(&text)));
        }

        let text = response
            .text()
            .await
            .map_err(|e| AppError::new(format!("Failed to read ClickHouse response: {e}")))?;

        let parsed: JsonResponse = serde_json::from_str(&text)
            .map_err(|e| AppError::new(format!("Failed to parse ClickHouse response: {e}")))?;

        let columns = parsed.meta.iter().map(|c| c.name.clone()).collect();
        let column_types = parsed.meta.iter().map(|c| c.type_name.clone()).collect();

        Ok(QueryOutcome {
            columns,
            column_types,
            rows: parsed.data,
            row_count: parsed.rows,
        })
    }

    pub async fn ping(&self) -> Result<(), AppError> {
        self.query("SELECT 1").await?;
        Ok(())
    }
}
