use libsql::{Connection, Rows, Value as LibsqlValue};
use serde_json::Value as JsonValue;

use crate::domain::query::RawQueryResult;
use crate::domain::table::{FilterOperator, SortDirection, TableFilter, TableRowsResult, TableSort};
use crate::error::AppError;

use super::metadata::quote_ident;

fn describe(err: libsql::Error) -> AppError {
    AppError::new(crate::error::describe_libsql_error(&err))
}

fn build_where_clause(filters: &[TableFilter], values: &mut Vec<LibsqlValue>) -> String {
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
                FilterOperator::Equals => {
                    values.push(LibsqlValue::Text(filter.value.clone().unwrap_or_default()));
                    format!("cast({col} as text) = ?")
                }
                FilterOperator::NotEquals => {
                    values.push(LibsqlValue::Text(filter.value.clone().unwrap_or_default()));
                    format!("cast({col} as text) <> ?")
                }
                FilterOperator::Like => {
                    values.push(LibsqlValue::Text(filter.value.clone().unwrap_or_default()));
                    format!("cast({col} as text) like ?")
                }
                FilterOperator::Ilike => {
                    values.push(LibsqlValue::Text(filter.value.clone().unwrap_or_default().to_lowercase()));
                    format!("lower(cast({col} as text)) like ?")
                }
                FilterOperator::NotLike => {
                    values.push(LibsqlValue::Text(filter.value.clone().unwrap_or_default()));
                    format!("cast({col} as text) not like ?")
                }
                FilterOperator::GreaterThan => {
                    values.push(LibsqlValue::Text(filter.value.clone().unwrap_or_default()));
                    format!("cast({col} as real) > cast(? as real)")
                }
                FilterOperator::GreaterOrEquals => {
                    values.push(LibsqlValue::Text(filter.value.clone().unwrap_or_default()));
                    format!("cast({col} as real) >= cast(? as real)")
                }
                FilterOperator::LessThan => {
                    values.push(LibsqlValue::Text(filter.value.clone().unwrap_or_default()));
                    format!("cast({col} as real) < cast(? as real)")
                }
                FilterOperator::LessOrEquals => {
                    values.push(LibsqlValue::Text(filter.value.clone().unwrap_or_default()));
                    format!("cast({col} as real) <= cast(? as real)")
                }
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
                        return "0".to_string();
                    }
                    let placeholders: Vec<&str> = items
                        .into_iter()
                        .map(|v| {
                            values.push(LibsqlValue::Text(v));
                            "?"
                        })
                        .collect();
                    format!("cast({col} as text) in ({})", placeholders.join(", "))
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

async fn column_names(rows: &Rows) -> Vec<String> {
    (0..rows.column_count())
        .map(|i| rows.column_name(i).unwrap_or_default().to_string())
        .collect()
}

pub async fn fetch_rows(
    conn: &Connection,
    table: &str,
    limit: i64,
    offset: i64,
    filters: &[TableFilter],
    sort: &[TableSort],
) -> Result<TableRowsResult, AppError> {
    let mut params: Vec<LibsqlValue> = Vec::new();
    let where_clause = build_where_clause(filters, &mut params);
    let order_by_clause = build_order_by_clause(sort);

    let sql = format!(
        "select * from {}{}{} limit ? offset ?",
        quote_ident(table),
        where_clause,
        order_by_clause
    );
    params.push(LibsqlValue::Integer(limit));
    params.push(LibsqlValue::Integer(offset));

    let mut rows = conn
        .query(&sql, params)
        .await
        .map_err(|e| AppError::new(format!("Failed to fetch rows: {}", crate::error::describe_libsql_error(&e))))?;

    let columns = column_names(&rows).await;
    let column_count = rows.column_count();

    let mut encoded_rows = Vec::new();
    while let Some(row) = rows.next().await.map_err(describe)? {
        let mut encoded = Vec::with_capacity(column_count as usize);
        for i in 0..column_count {
            let value = row.get_value(i).map_err(describe)?;
            encoded.push(crate::domain::query::encode_cell(libsql_value_to_json(&value)));
        }
        encoded_rows.push(encoded);
    }

    let row_count = encoded_rows.len();
    let has_more = row_count as i64 == limit;

    Ok(TableRowsResult {
        columns,
        rows: encoded_rows,
        row_count: row_count as u32,
        has_more,
        duration_ms: 0,
    })
}

pub async fn count_rows(conn: &Connection, table: &str, filters: &[TableFilter]) -> Result<u64, AppError> {
    let mut params: Vec<LibsqlValue> = Vec::new();
    let where_clause = build_where_clause(filters, &mut params);

    let sql = format!("select count(*) from {}{}", quote_ident(table), where_clause);

    let mut rows = conn
        .query(&sql, params)
        .await
        .map_err(|e| AppError::new(format!("Failed to count rows: {}", crate::error::describe_libsql_error(&e))))?;

    let row = rows
        .next()
        .await
        .map_err(describe)?
        .ok_or_else(|| AppError::new("Failed to read row count: no rows returned."))?;
    let count: i64 = row.get(0).map_err(describe)?;
    Ok(count as u64)
}

fn looks_like_select(sql: &str) -> bool {
    let trimmed = sql.trim_start().to_lowercase();
    trimmed.starts_with("select") || trimmed.starts_with("pragma") || trimmed.starts_with("explain")
        || trimmed.starts_with("with")
}

pub async fn execute_query(
    conn: &Connection,
    sql: &str,
    offset: u64,
    limit: u64,
) -> Result<RawQueryResult, AppError> {
    if !looks_like_select(sql) {
        let affected = conn
            .execute(sql, ())
            .await
            .map_err(|e| AppError::new(crate::error::describe_libsql_error(&e)))?;
        return Ok(RawQueryResult::Affected { row_count: affected });
    }

    match execute_query_page(conn, sql, offset, limit).await {
        Ok(result) => Ok(result),
        Err(_) => {
            let mut rows = conn
                .query(sql, ())
                .await
                .map_err(|e| AppError::new(crate::error::describe_libsql_error(&e)))?;
            let columns = column_names(&rows).await;
            let column_count = rows.column_count();
            let mut json_rows = Vec::new();
            while let Some(row) = rows.next().await.map_err(describe)? {
                let mut values = Vec::with_capacity(column_count as usize);
                for i in 0..column_count {
                    values.push(libsql_value_to_json(&row.get_value(i).map_err(describe)?));
                }
                json_rows.push(values);
            }
            Ok(RawQueryResult::Rows { columns, rows: json_rows, total_row_count: None })
        }
    }
}

async fn execute_query_page(
    conn: &Connection,
    sql: &str,
    offset: u64,
    limit: u64,
) -> Result<RawQueryResult, AppError> {
    let page_sql =
        format!("select *, count(*) over() as __total_row_count from ({sql}) as q limit {limit} offset {offset}");
    let mut rows = conn.query(&page_sql, ()).await.map_err(describe)?;

    let all_columns = column_names(&rows).await;
    let total_col_index = all_columns.len().saturating_sub(1);
    let columns = all_columns.get(..total_col_index).unwrap_or_default().to_vec();

    let mut buffered_rows: Vec<libsql::Row> = Vec::new();
    while let Some(row) = rows.next().await.map_err(describe)? {
        buffered_rows.push(row);
    }

    let total: i64 = match buffered_rows.first() {
        Some(row) => row.get(total_col_index as i32).map_err(describe)?,
        None => {
            let count_sql = format!("select count(*) from ({sql}) as q");
            let mut count_rows = conn.query(&count_sql, ()).await.map_err(describe)?;
            let count_row = count_rows
                .next()
                .await
                .map_err(describe)?
                .ok_or_else(|| AppError::new("Failed to count query results."))?;
            count_row.get(0).map_err(describe)?
        }
    };

    let mut json_rows = Vec::with_capacity(buffered_rows.len());
    for row in &buffered_rows {
        let mut values = Vec::with_capacity(total_col_index);
        for i in 0..total_col_index {
            values.push(libsql_value_to_json(&row.get_value(i as i32).map_err(describe)?));
        }
        json_rows.push(values);
    }

    Ok(RawQueryResult::Rows {
        columns,
        rows: json_rows,
        total_row_count: Some(total as usize),
    })
}

pub async fn update_cell_text(
    conn: &Connection,
    table: &str,
    pk_values: &[(String, JsonValue)],
    column: &str,
    new_value: Option<&str>,
) -> Result<(), AppError> {
    let mut params: Vec<LibsqlValue> = vec![json_to_param(new_value.map(JsonValue::from).unwrap_or(JsonValue::Null))];
    let where_clause = append_pk_params(&mut params, pk_values);

    let sql = format!(
        "update {} set {} = ? where {}",
        quote_ident(table),
        quote_ident(column),
        where_clause
    );

    execute_single_row_update(conn, &sql, params).await
}

pub async fn update_json_cell(
    conn: &Connection,
    table: &str,
    pk_values: &[(String, JsonValue)],
    column: &str,
    value: &JsonValue,
) -> Result<(), AppError> {
    let value_text = value.to_string();

    let mut params: Vec<LibsqlValue> = vec![LibsqlValue::Text(value_text)];
    let where_clause = append_pk_params(&mut params, pk_values);

    let sql = format!(
        "update {} set {} = ? where {}",
        quote_ident(table),
        quote_ident(column),
        where_clause
    );

    execute_single_row_update(conn, &sql, params).await
}

pub async fn insert_row(conn: &Connection, table: &str, values: &[(String, String)]) -> Result<(), AppError> {
    if values.is_empty() {
        return Err(AppError::new("Cannot insert a row with no columns."));
    }

    let mut params: Vec<LibsqlValue> = Vec::with_capacity(values.len());
    let mut column_names = Vec::with_capacity(values.len());
    let mut placeholders = Vec::with_capacity(values.len());

    for (column, value) in values {
        params.push(LibsqlValue::Text(value.clone()));
        column_names.push(quote_ident(column));
        placeholders.push("?".to_string());
    }

    let sql = format!(
        "insert into {} ({}) values ({})",
        quote_ident(table),
        column_names.join(", "),
        placeholders.join(", ")
    );

    conn.execute(&sql, params)
        .await
        .map_err(|e| AppError::new(format!("Failed to insert row: {}", crate::error::describe_libsql_error(&e))))?;

    Ok(())
}

pub async fn delete_rows(
    conn: &Connection,
    table: &str,
    rows_pk_values: &[Vec<(String, JsonValue)>],
) -> Result<u64, AppError> {
    if rows_pk_values.is_empty() {
        return Ok(0);
    }

    let mut params: Vec<LibsqlValue> = Vec::new();
    let mut row_clauses: Vec<String> = Vec::with_capacity(rows_pk_values.len());

    for pk_values in rows_pk_values {
        let mut clause_parts = Vec::with_capacity(pk_values.len());
        for (pk_col, pk_value) in pk_values {
            params.push(json_pk_to_param(pk_value));
            clause_parts.push(format!("cast({} as text) = ?", quote_ident(pk_col)));
        }
        row_clauses.push(format!("({})", clause_parts.join(" and ")));
    }

    let where_clause = row_clauses.join(" or ");
    let sql = format!("delete from {} where {}", quote_ident(table), where_clause);

    let affected = conn
        .execute(&sql, params)
        .await
        .map_err(|e| AppError::new(format!("Failed to delete rows: {}", crate::error::describe_libsql_error(&e))))?;

    let expected = rows_pk_values.len() as u64;
    if affected != expected {
        return Err(AppError::new(format!(
            "Expected to delete {expected} row(s) but {affected} matched — the data may have changed. Refresh and try again."
        )));
    }

    Ok(affected)
}

fn append_pk_params(params: &mut Vec<LibsqlValue>, pk_values: &[(String, JsonValue)]) -> String {
    let mut where_clause = String::new();
    for (i, (pk_col, pk_value)) in pk_values.iter().enumerate() {
        if i > 0 {
            where_clause.push_str(" and ");
        }
        params.push(json_pk_to_param(pk_value));
        where_clause.push_str(&format!("cast({} as text) = ?", quote_ident(pk_col)));
    }
    where_clause
}

async fn execute_single_row_update(conn: &Connection, sql: &str, params: Vec<LibsqlValue>) -> Result<(), AppError> {
    let affected = conn
        .execute(sql, params)
        .await
        .map_err(|e| AppError::new(format!("Failed to update cell: {}", crate::error::describe_libsql_error(&e))))?;

    if affected == 0 {
        return Err(AppError::new("No matching row found — it may have been deleted or modified."));
    }
    if affected > 1 {
        return Err(AppError::new(
            "Update matched more than one row — refusing to apply to avoid unintended changes.",
        ));
    }

    Ok(())
}

fn json_to_param(value: JsonValue) -> LibsqlValue {
    match value {
        JsonValue::Null => LibsqlValue::Null,
        JsonValue::String(s) => LibsqlValue::Text(s),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                LibsqlValue::Integer(i)
            } else {
                LibsqlValue::Real(n.as_f64().unwrap_or(0.0))
            }
        }
        JsonValue::Bool(b) => LibsqlValue::Integer(if b { 1 } else { 0 }),
        other => LibsqlValue::Text(other.to_string()),
    }
}

fn json_pk_to_param(value: &JsonValue) -> LibsqlValue {
    match value {
        JsonValue::String(s) => LibsqlValue::Text(s.clone()),
        JsonValue::Number(n) => LibsqlValue::Text(n.to_string()),
        JsonValue::Bool(b) => LibsqlValue::Text(b.to_string()),
        JsonValue::Null => LibsqlValue::Null,
        _ => LibsqlValue::Text(value.to_string()),
    }
}

fn libsql_value_to_json(value: &LibsqlValue) -> JsonValue {
    match value {
        LibsqlValue::Null => JsonValue::Null,
        LibsqlValue::Integer(i) => JsonValue::from(*i),
        LibsqlValue::Real(r) => JsonValue::from(*r),
        LibsqlValue::Text(s) => JsonValue::String(s.clone()),
        LibsqlValue::Blob(b) => JsonValue::String(format!("\\x{}", hex_encode(b))),
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
