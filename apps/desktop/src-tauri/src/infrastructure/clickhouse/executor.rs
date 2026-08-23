use serde_json::Value as JsonValue;

use crate::domain::query::RawQueryResult;
use crate::domain::table::{FilterOperator, TableFilter, TableRowsResult, TableSort, SortDirection};
use crate::domain::query::encode_cell;
use crate::error::AppError;

use super::client::QueryOutcome;
use super::pool::ClickHouseClient;

fn quote_literal(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

fn json_to_sql_literal(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => "NULL".to_string(),
        JsonValue::String(s) => quote_literal(s),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::Bool(b) => b.to_string(),
        other => quote_literal(&other.to_string()),
    }
}

fn render_filter(filter: &TableFilter) -> String {
    let column = format!("`{}`", filter.column);
    match filter.operator {
        FilterOperator::IsNull => format!("{column} IS NULL"),
        FilterOperator::IsNotNull => format!("{column} IS NOT NULL"),
        FilterOperator::In => {
            let items = filter
                .value
                .as_deref()
                .unwrap_or_default()
                .split(',')
                .map(|v| quote_literal(v.trim()))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{column} IN ({items})")
        }
        _ => {
            let value = filter.value.as_deref().unwrap_or_default();
            let op = match filter.operator {
                FilterOperator::Equals => "=",
                FilterOperator::NotEquals => "!=",
                FilterOperator::GreaterThan => ">",
                FilterOperator::GreaterOrEquals => ">=",
                FilterOperator::LessThan => "<",
                FilterOperator::LessOrEquals => "<=",
                FilterOperator::Like => "LIKE",
                FilterOperator::Ilike => "ILIKE",
                FilterOperator::NotLike => "NOT LIKE",
                _ => unreachable!(),
            };
            format!("{column} {op} {}", quote_literal(value))
        }
    }
}

fn render_where(filters: &[TableFilter]) -> String {
    if filters.is_empty() {
        return String::new();
    }
    let clauses: Vec<String> = filters.iter().map(render_filter).collect();
    format!(" WHERE {}", clauses.join(" AND "))
}

fn render_order_by(sort: &[TableSort]) -> String {
    if sort.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = sort
        .iter()
        .map(|s| {
            let dir = match s.direction {
                SortDirection::Asc => "ASC",
                SortDirection::Desc => "DESC",
            };
            format!("`{}` {dir}", s.column)
        })
        .collect();
    format!(" ORDER BY {}", parts.join(", "))
}

fn outcome_to_rows(outcome: QueryOutcome) -> (Vec<String>, Vec<Vec<JsonValue>>) {
    let columns = outcome.columns.clone();
    let rows = outcome
        .rows
        .into_iter()
        .map(|row| columns.iter().map(|c| row.get(c).cloned().unwrap_or(JsonValue::Null)).collect())
        .collect();
    (columns, rows)
}

pub async fn fetch_rows(
    client: &ClickHouseClient,
    table: &str,
    limit: i64,
    offset: i64,
    filters: &[TableFilter],
    sort: &[TableSort],
) -> Result<TableRowsResult, AppError> {
    let start = std::time::Instant::now();
    let sql = format!(
        "select * from `{table}`{}{} limit {limit} offset {offset}",
        render_where(filters),
        render_order_by(sort)
    );
    let outcome = client.query(&sql).await?;
    let (columns, rows) = outcome_to_rows(outcome);

    let encoded_rows: Vec<Vec<String>> = rows
        .into_iter()
        .map(|row| row.into_iter().map(encode_cell).collect())
        .collect();

    let has_more = encoded_rows.len() as i64 == limit;

    Ok(TableRowsResult {
        columns,
        row_count: encoded_rows.len() as u32,
        rows: encoded_rows,
        has_more,
        duration_ms: start.elapsed().as_millis() as u32,
    })
}

pub async fn count_rows(client: &ClickHouseClient, table: &str, filters: &[TableFilter]) -> Result<u64, AppError> {
    let sql = format!("select count() as cnt from `{table}`{}", render_where(filters));
    let outcome = client.query(&sql).await?;
    let row = outcome.rows.into_iter().next();
    Ok(row.and_then(|r| r.get("cnt").and_then(|v| v.as_u64())).unwrap_or(0))
}

pub async fn update_cell_text(
    client: &ClickHouseClient,
    table: &str,
    pk_values: &[(String, JsonValue)],
    column: &str,
    new_value: Option<&str>,
) -> Result<(), AppError> {
    let value_sql = match new_value {
        Some(v) => quote_literal(v),
        None => "NULL".to_string(),
    };
    let where_clause = render_pk_where(pk_values);
    let sql = format!("alter table `{table}` update `{column}` = {value_sql} where {where_clause} settings mutations_sync = 1");
    client.execute(&sql).await
}

pub async fn update_json_cell(
    client: &ClickHouseClient,
    table: &str,
    pk_values: &[(String, JsonValue)],
    column: &str,
    value: &JsonValue,
) -> Result<(), AppError> {
    let where_clause = render_pk_where(pk_values);
    let sql = format!(
        "alter table `{table}` update `{column}` = {} where {where_clause} settings mutations_sync = 1",
        json_to_sql_literal(value)
    );
    client.execute(&sql).await
}

fn render_pk_where(pk_values: &[(String, JsonValue)]) -> String {
    pk_values
        .iter()
        .map(|(col, val)| format!("`{col}` = {}", json_to_sql_literal(val)))
        .collect::<Vec<_>>()
        .join(" AND ")
}

pub async fn delete_rows(
    client: &ClickHouseClient,
    table: &str,
    rows_pk_values: &[Vec<(String, JsonValue)>],
) -> Result<u64, AppError> {
    let mut deleted = 0u64;
    for pk_values in rows_pk_values {
        let where_clause = render_pk_where(pk_values);
        let sql = format!("alter table `{table}` delete where {where_clause} settings mutations_sync = 1");
        client.execute(&sql).await?;
        deleted += 1;
    }
    Ok(deleted)
}

pub async fn insert_row(
    client: &ClickHouseClient,
    table: &str,
    values: &[(String, String)],
) -> Result<(), AppError> {
    if values.is_empty() {
        return Err(AppError::new("Cannot insert a row with no values."));
    }
    let columns = values.iter().map(|(c, _)| format!("`{c}`")).collect::<Vec<_>>().join(", ");
    let literals = values.iter().map(|(_, v)| quote_literal(v)).collect::<Vec<_>>().join(", ");
    let sql = format!("insert into `{table}` ({columns}) values ({literals})");
    client.execute(&sql).await
}

pub async fn execute_query(client: &ClickHouseClient, sql: &str, offset: u64, limit: u64) -> Result<RawQueryResult, AppError> {
    let trimmed = sql.trim().trim_end_matches(';');
    let is_select = trimmed.to_uppercase().starts_with("SELECT")
        || trimmed.to_uppercase().starts_with("SHOW")
        || trimmed.to_uppercase().starts_with("DESCRIBE")
        || trimmed.to_uppercase().starts_with("WITH");

    if !is_select {
        client.execute(trimmed).await?;
        return Ok(RawQueryResult::Affected { row_count: 0 });
    }

    let wrapped = format!("select * from ({trimmed}) as q limit {limit} offset {offset}");
    let outcome = client.query(&wrapped).await?;
    let (columns, rows) = outcome_to_rows(outcome);

    let count_sql = format!("select count() as cnt from ({trimmed}) as q");
    let total_row_count = match client.query(&count_sql).await {
        Ok(count_outcome) => count_outcome
            .rows
            .into_iter()
            .next()
            .and_then(|r| r.get("cnt").and_then(|v| v.as_u64()))
            .map(|c| c as usize),
        Err(_) => None,
    };

    Ok(RawQueryResult::Rows { columns, rows, total_row_count })
}
