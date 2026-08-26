use serde_json::Value as JsonValue;
use trino_rust_client::client::Client;
use trino_rust_client::Row;

use crate::domain::query::RawQueryResult;
use crate::domain::table::{FilterOperator, SortDirection, TableFilter, TableRowsResult, TableSort};
use crate::error::AppError;

use super::metadata::quote_ident;

fn quote_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn describe(err: trino_rust_client::error::Error) -> AppError {
    AppError::new(crate::error::describe_trino_error(&err))
}

fn json_to_sql_literal(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => "null".to_string(),
        JsonValue::String(s) => quote_literal(s),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::Bool(b) => b.to_string(),
        other => quote_literal(&other.to_string()),
    }
}

fn build_where_clause(filters: &[TableFilter]) -> String {
    if filters.is_empty() {
        return String::new();
    }

    let clauses: Vec<String> = filters
        .iter()
        .map(|filter| {
            let col = quote_ident(&filter.column);
            match filter.operator {
                FilterOperator::IsNull => format!("{col} is null"),
                FilterOperator::IsNotNull => format!("{col} is not null"),
                FilterOperator::Equals => format!("cast({col} as varchar) = {}", quote_literal(&filter.value.clone().unwrap_or_default())),
                FilterOperator::NotEquals => format!("cast({col} as varchar) <> {}", quote_literal(&filter.value.clone().unwrap_or_default())),
                FilterOperator::Like => format!("cast({col} as varchar) like {}", quote_literal(&filter.value.clone().unwrap_or_default())),
                FilterOperator::Ilike => format!("lower(cast({col} as varchar)) like {}", quote_literal(&filter.value.clone().unwrap_or_default().to_lowercase())),
                FilterOperator::NotLike => format!("cast({col} as varchar) not like {}", quote_literal(&filter.value.clone().unwrap_or_default())),
                FilterOperator::GreaterThan => format!("cast({col} as double) > cast({} as double)", quote_literal(&filter.value.clone().unwrap_or_default())),
                FilterOperator::GreaterOrEquals => format!("cast({col} as double) >= cast({} as double)", quote_literal(&filter.value.clone().unwrap_or_default())),
                FilterOperator::LessThan => format!("cast({col} as double) < cast({} as double)", quote_literal(&filter.value.clone().unwrap_or_default())),
                FilterOperator::LessOrEquals => format!("cast({col} as double) <= cast({} as double)", quote_literal(&filter.value.clone().unwrap_or_default())),
                FilterOperator::In => {
                    let items: Vec<String> = filter
                        .value
                        .as_deref()
                        .unwrap_or("")
                        .split(',')
                        .map(|v| v.trim().to_string())
                        .filter(|v| !v.is_empty())
                        .collect();
                    if items.is_empty() {
                        return "1 = 0".to_string();
                    }
                    let quoted: Vec<String> = items.into_iter().map(|v| quote_literal(&v)).collect();
                    format!("cast({col} as varchar) in ({})", quoted.join(", "))
                }
            }
        })
        .collect();

    format!(" where {}", clauses.join(" and "))
}

fn build_order_by_clause(sort: &[TableSort]) -> String {
    if sort.is_empty() {
        return String::new();
    }
    let clauses: Vec<String> = sort
        .iter()
        .map(|s| {
            let direction = match s.direction {
                SortDirection::Asc => "asc",
                SortDirection::Desc => "desc",
            };
            format!("{} {}", quote_ident(&s.column), direction)
        })
        .collect();
    format!(" order by {}", clauses.join(", "))
}

fn qualified_table(schema: &str, table: &str) -> String {
    format!("{}.{}", quote_ident(schema), quote_ident(table))
}

pub async fn fetch_rows(
    client: &Client,
    schema: &str,
    table: &str,
    limit: i64,
    offset: i64,
    filters: &[TableFilter],
    sort: &[TableSort],
) -> Result<TableRowsResult, AppError> {
    let where_clause = build_where_clause(filters);
    let order_by_clause = build_order_by_clause(sort);

    let sql = format!(
        "select * from {}{}{} offset {} limit {}",
        qualified_table(schema, table),
        where_clause,
        order_by_clause,
        offset,
        limit
    );

    let dataset = client.get_all::<Row>(&sql).await.map_err(describe)?;
    let (types, data) = dataset.split();
    let columns: Vec<String> = types.iter().map(|(name, _)| name.clone()).collect();

    let mut encoded_rows = Vec::with_capacity(data.len());
    for row in data {
        let values = row.into_json();
        encoded_rows.push(values.into_iter().map(crate::domain::query::encode_cell).collect());
    }

    let row_count = encoded_rows.len();
    let has_more = row_count as i64 == limit;

    Ok(TableRowsResult { columns, rows: encoded_rows, row_count: row_count as u32, has_more, duration_ms: 0 })
}

pub async fn count_rows(client: &Client, schema: &str, table: &str, filters: &[TableFilter]) -> Result<u64, AppError> {
    let where_clause = build_where_clause(filters);
    let sql = format!("select count(*) from {}{}", qualified_table(schema, table), where_clause);
    let dataset = client.get_all::<Row>(&sql).await.map_err(describe)?;
    let row = dataset
        .into_vec()
        .into_iter()
        .next()
        .ok_or_else(|| AppError::new("Failed to read row count: no rows returned."))?;
    let values = row.into_json();
    let count = values.first().and_then(JsonValue::as_i64).unwrap_or(0);
    Ok(count as u64)
}

pub async fn insert_row(client: &Client, schema: &str, table: &str, values: &[(String, JsonValue)]) -> Result<(), AppError> {
    if values.is_empty() {
        return Err(AppError::new("Cannot insert a row with no columns."));
    }

    let column_names: Vec<String> = values.iter().map(|(c, _)| quote_ident(c)).collect();
    let literals: Vec<String> = values.iter().map(|(_, v)| json_to_sql_literal(v)).collect();

    let sql = format!(
        "insert into {} ({}) values ({})",
        qualified_table(schema, table),
        column_names.join(", "),
        literals.join(", ")
    );

    client.execute(&sql).await.map_err(describe)?;
    Ok(())
}

fn looks_like_select(sql: &str) -> bool {
    let trimmed = sql.trim_start().to_lowercase();
    trimmed.starts_with("select") || trimmed.starts_with("show") || trimmed.starts_with("describe")
        || trimmed.starts_with("explain") || trimmed.starts_with("with")
}

pub async fn execute_query(client: &Client, sql: &str, offset: u64, limit: u64) -> Result<RawQueryResult, AppError> {
    if !looks_like_select(sql) {
        let result = client.execute(sql).await.map_err(describe)?;
        return Ok(RawQueryResult::Affected { row_count: result.update_count.unwrap_or(0) });
    }

    let dataset = client.get_all::<Row>(sql).await.map_err(describe)?;
    let (types, data) = dataset.split();
    let columns: Vec<String> = types.iter().map(|(name, _)| name.clone()).collect();
    let total_row_count = Some(data.len());

    let page: Vec<Row> = data.into_iter().skip(offset as usize).take(limit as usize).collect();
    let json_rows: Vec<Vec<JsonValue>> = page.into_iter().map(Row::into_json).collect();

    Ok(RawQueryResult::Rows { columns, rows: json_rows, total_row_count })
}
