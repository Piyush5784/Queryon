use duckdb::types::Value as DuckValue;
use duckdb::{Connection, Row};
use serde_json::Value as JsonValue;

use crate::domain::query::RawQueryResult;
use crate::domain::table::{FilterOperator, SortDirection, TableFilter, TableRowsResult, TableSort};
use crate::error::AppError;

fn clean(e: duckdb::Error) -> AppError {
    AppError::new(e.to_string())
}

fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

fn quote_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub fn value_to_json(value: &DuckValue) -> JsonValue {
    match value {
        DuckValue::Null => JsonValue::Null,
        DuckValue::Boolean(b) => JsonValue::Bool(*b),
        DuckValue::TinyInt(n) => JsonValue::from(*n),
        DuckValue::SmallInt(n) => JsonValue::from(*n),
        DuckValue::Int(n) => JsonValue::from(*n),
        DuckValue::BigInt(n) => JsonValue::from(*n),
        DuckValue::HugeInt(n) => JsonValue::from(n.to_string()),
        DuckValue::UTinyInt(n) => JsonValue::from(*n),
        DuckValue::USmallInt(n) => JsonValue::from(*n),
        DuckValue::UInt(n) => JsonValue::from(*n),
        DuckValue::UBigInt(n) => JsonValue::from(*n),
        DuckValue::Float(n) => serde_json::Number::from_f64(*n as f64).map(JsonValue::Number).unwrap_or(JsonValue::Null),
        DuckValue::Double(n) => serde_json::Number::from_f64(*n).map(JsonValue::Number).unwrap_or(JsonValue::Null),
        DuckValue::Decimal(d) => JsonValue::from(d.to_string()),
        DuckValue::Text(s) => JsonValue::String(s.clone()),
        DuckValue::Blob(b) => JsonValue::String(format!("\\x{}", b.iter().map(|x| format!("{x:02x}")).collect::<String>())),
        DuckValue::Timestamp(_, _) | DuckValue::Date32(_) | DuckValue::Time64(_, _) => JsonValue::String(format!("{value:?}")),
        DuckValue::List(items) | DuckValue::Array(items) => JsonValue::Array(items.iter().map(value_to_json).collect()),
        DuckValue::Struct(fields) => {
            let map: serde_json::Map<String, JsonValue> =
                fields.iter().map(|(k, v)| (k.clone(), value_to_json(v))).collect();
            JsonValue::Object(map)
        }
        other => JsonValue::String(format!("{other:?}")),
    }
}

fn json_to_duck_param(value: &JsonValue) -> DuckValue {
    match value {
        JsonValue::Null => DuckValue::Null,
        JsonValue::Bool(b) => DuckValue::Boolean(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                DuckValue::BigInt(i)
            } else if let Some(f) = n.as_f64() {
                DuckValue::Double(f)
            } else {
                DuckValue::Text(n.to_string())
            }
        }
        JsonValue::String(s) => DuckValue::Text(s.clone()),
        other => DuckValue::Text(other.to_string()),
    }
}

fn row_to_json_vec(row: &Row, column_count: usize) -> Result<Vec<JsonValue>, duckdb::Error> {
    let mut values = Vec::with_capacity(column_count);
    for i in 0..column_count {
        let value: DuckValue = row.get(i)?;
        values.push(value_to_json(&value));
    }
    Ok(values)
}

fn render_filter(filter: &TableFilter) -> (String, Option<String>) {
    let col = quote_ident(&filter.column);
    match filter.operator {
        FilterOperator::IsNull => (format!("{col} is null"), None),
        FilterOperator::IsNotNull => (format!("{col} is not null"), None),
        FilterOperator::In => {
            let items: Vec<String> = filter
                .value
                .as_deref()
                .unwrap_or("")
                .split(',')
                .map(|v| quote_literal(v.trim()))
                .collect();
            (format!("cast({col} as varchar) in ({})", items.join(", ")), None)
        }
        FilterOperator::Equals => (format!("cast({col} as varchar) = ?"), filter.value.clone()),
        FilterOperator::NotEquals => (format!("cast({col} as varchar) <> ?"), filter.value.clone()),
        FilterOperator::Like => (format!("cast({col} as varchar) like ?"), filter.value.clone()),
        FilterOperator::Ilike => (format!("cast({col} as varchar) ilike ?"), filter.value.clone()),
        FilterOperator::NotLike => (format!("cast({col} as varchar) not like ?"), filter.value.clone()),
        FilterOperator::GreaterThan => (format!("cast({col} as double) > cast(? as double)"), filter.value.clone()),
        FilterOperator::GreaterOrEquals => (format!("cast({col} as double) >= cast(? as double)"), filter.value.clone()),
        FilterOperator::LessThan => (format!("cast({col} as double) < cast(? as double)"), filter.value.clone()),
        FilterOperator::LessOrEquals => (format!("cast({col} as double) <= cast(? as double)"), filter.value.clone()),
    }
}

fn build_where_clause(filters: &[TableFilter], params: &mut Vec<String>) -> String {
    if filters.is_empty() {
        return String::new();
    }
    let clauses: Vec<String> = filters
        .iter()
        .map(|f| {
            let (clause, value) = render_filter(f);
            if let Some(v) = value {
                params.push(v);
            }
            clause
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
            let dir = match s.direction {
                SortDirection::Asc => "asc",
                SortDirection::Desc => "desc",
            };
            format!("{} {dir}", quote_ident(&s.column))
        })
        .collect();
    format!(" order by {}", clauses.join(", "))
}

pub fn fetch_rows(
    conn: &Connection,
    table: &str,
    limit: i64,
    offset: i64,
    filters: &[TableFilter],
    sort: &[TableSort],
) -> Result<TableRowsResult, AppError> {
    let start = std::time::Instant::now();
    let mut params: Vec<String> = Vec::new();
    let where_clause = build_where_clause(filters, &mut params);
    let order_by_clause = build_order_by_clause(sort);

    let sql = format!(
        "select * from {}{}{} limit {limit} offset {offset}",
        quote_ident(table),
        where_clause,
        order_by_clause
    );

    let mut stmt = conn.prepare(&sql).map_err(clean)?;
    let param_refs: Vec<&dyn duckdb::ToSql> = params.iter().map(|p| p as &dyn duckdb::ToSql).collect();

    let mut rows = stmt.query(param_refs.as_slice()).map_err(clean)?;
    let column_count = rows.as_ref().map(|s| s.column_count()).unwrap_or(0);

    let mut encoded_rows = Vec::new();
    while let Some(row) = rows.next().map_err(clean)? {
        let values = row_to_json_vec(row, column_count).map_err(clean)?;
        encoded_rows.push(values.into_iter().map(crate::domain::query::encode_cell).collect::<Vec<_>>());
    }

    let column_names = rows.as_ref().map(|s| s.column_names()).unwrap_or_default();
    let row_count = encoded_rows.len();
    let has_more = row_count as i64 == limit;

    Ok(TableRowsResult {
        columns: column_names,
        rows: encoded_rows,
        row_count: row_count as u32,
        has_more,
        duration_ms: start.elapsed().as_millis() as u32,
    })
}

pub fn count_rows(conn: &Connection, table: &str, filters: &[TableFilter]) -> Result<u64, AppError> {
    let mut params: Vec<String> = Vec::new();
    let where_clause = build_where_clause(filters, &mut params);
    let sql = format!("select count(*) from {}{}", quote_ident(table), where_clause);

    let mut stmt = conn.prepare(&sql).map_err(clean)?;
    let param_refs: Vec<&dyn duckdb::ToSql> = params.iter().map(|p| p as &dyn duckdb::ToSql).collect();
    let count: i64 = stmt.query_row(param_refs.as_slice(), |row| row.get(0)).map_err(clean)?;
    Ok(count as u64)
}

fn looks_like_select(sql: &str) -> bool {
    let trimmed = sql.trim_start().to_lowercase();
    trimmed.starts_with("select") || trimmed.starts_with("with") || trimmed.starts_with("pragma")
        || trimmed.starts_with("describe") || trimmed.starts_with("show")
}

pub fn execute_query(conn: &Connection, sql: &str, offset: u64, limit: u64) -> Result<RawQueryResult, AppError> {
    let trimmed = sql.trim().trim_end_matches(';');

    if !looks_like_select(trimmed) {
        let affected = conn.execute(trimmed, []).map_err(clean)?;
        return Ok(RawQueryResult::Affected { row_count: affected as u64 });
    }

    let page_sql = format!("select * from ({trimmed}) as q limit {limit} offset {offset}");
    let page_result = (|| -> Result<RawQueryResult, duckdb::Error> {
        let mut stmt = conn.prepare(&page_sql)?;
        let mut rows = stmt.query([])?;
        let column_count = rows.as_ref().map(|s| s.column_count()).unwrap_or(0);

        let mut collected = Vec::new();
        while let Some(row) = rows.next()? {
            collected.push(row_to_json_vec(row, column_count)?);
        }
        let column_names = rows.as_ref().map(|s| s.column_names()).unwrap_or_default();

        let count_sql = format!("select count(*) from ({trimmed}) as q");
        let total: i64 = conn.query_row(&count_sql, [], |row| row.get(0))?;

        Ok(RawQueryResult::Rows { columns: column_names, rows: collected, total_row_count: Some(total as usize) })
    })();

    match page_result {
        Ok(result) => Ok(result),
        Err(_) => {
            let mut stmt = conn.prepare(trimmed).map_err(clean)?;
            let mut rows = stmt.query([]).map_err(clean)?;
            let column_count = rows.as_ref().map(|s| s.column_count()).unwrap_or(0);

            let mut collected = Vec::new();
            while let Some(row) = rows.next().map_err(clean)? {
                collected.push(row_to_json_vec(row, column_count).map_err(clean)?);
            }
            let column_names = rows.as_ref().map(|s| s.column_names()).unwrap_or_default();
            Ok(RawQueryResult::Rows { columns: column_names, rows: collected, total_row_count: None })
        }
    }
}

fn append_pk_clause(pk_values: &[(String, JsonValue)], params: &mut Vec<DuckValue>) -> String {
    pk_values
        .iter()
        .map(|(col, val)| {
            params.push(json_to_duck_param(val));
            format!("{} = ?", quote_ident(col))
        })
        .collect::<Vec<_>>()
        .join(" and ")
}

pub fn update_cell_text(
    conn: &Connection,
    table: &str,
    pk_values: &[(String, JsonValue)],
    column: &str,
    new_value: Option<&str>,
) -> Result<(), AppError> {
    let mut params: Vec<DuckValue> = vec![match new_value {
        Some(v) => DuckValue::Text(v.to_string()),
        None => DuckValue::Null,
    }];
    let where_clause = append_pk_clause(pk_values, &mut params);
    let sql = format!("update {} set {} = ? where {where_clause}", quote_ident(table), quote_ident(column));
    execute_single_row_update(conn, &sql, params)
}

pub fn update_json_cell(
    conn: &Connection,
    table: &str,
    pk_values: &[(String, JsonValue)],
    column: &str,
    value: &JsonValue,
) -> Result<(), AppError> {
    let mut params: Vec<DuckValue> = vec![json_to_duck_param(value)];
    let where_clause = append_pk_clause(pk_values, &mut params);
    let sql = format!("update {} set {} = ? where {where_clause}", quote_ident(table), quote_ident(column));
    execute_single_row_update(conn, &sql, params)
}

fn execute_single_row_update(conn: &Connection, sql: &str, params: Vec<DuckValue>) -> Result<(), AppError> {
    let param_refs: Vec<&dyn duckdb::ToSql> = params.iter().map(|p| p as &dyn duckdb::ToSql).collect();
    let affected = conn.execute(sql, param_refs.as_slice()).map_err(clean)?;
    if affected == 0 {
        return Err(AppError::new("No matching row found — it may have been deleted or modified."));
    }
    if affected > 1 {
        return Err(AppError::new("Update matched more than one row — refusing to apply to avoid unintended changes."));
    }
    Ok(())
}

pub fn insert_row(conn: &Connection, table: &str, values: &[(String, String)]) -> Result<(), AppError> {
    if values.is_empty() {
        return Err(AppError::new("Cannot insert a row with no columns."));
    }
    let columns: Vec<String> = values.iter().map(|(c, _)| quote_ident(c)).collect();
    let placeholders: Vec<&str> = values.iter().map(|_| "?").collect();
    let params: Vec<DuckValue> = values.iter().map(|(_, v)| DuckValue::Text(v.clone())).collect();
    let param_refs: Vec<&dyn duckdb::ToSql> = params.iter().map(|p| p as &dyn duckdb::ToSql).collect();

    let sql = format!(
        "insert into {} ({}) values ({})",
        quote_ident(table),
        columns.join(", "),
        placeholders.join(", ")
    );
    conn.execute(&sql, param_refs.as_slice()).map_err(clean)?;
    Ok(())
}

pub fn delete_rows(conn: &Connection, table: &str, rows_pk_values: &[Vec<(String, JsonValue)>]) -> Result<u64, AppError> {
    if rows_pk_values.is_empty() {
        return Ok(0);
    }

    let mut deleted = 0u64;
    for pk_values in rows_pk_values {
        let mut params: Vec<DuckValue> = Vec::new();
        let where_clause = append_pk_clause(pk_values, &mut params);
        let param_refs: Vec<&dyn duckdb::ToSql> = params.iter().map(|p| p as &dyn duckdb::ToSql).collect();
        let sql = format!("delete from {} where {where_clause}", quote_ident(table));
        let affected = conn.execute(&sql, param_refs.as_slice()).map_err(clean)?;
        deleted += affected as u64;
    }
    Ok(deleted)
}
