use rust_decimal::Decimal;
use serde_json::Value as JsonValue;
use tiberius::{ColumnData, Query};

use crate::domain::query::RawQueryResult;
use crate::domain::table::{FilterOperator, SortDirection, TableFilter, TableRowsResult, TableSort};
use crate::error::AppError;

use super::metadata::quote_ident;
use super::pool::MssqlClient;

fn build_where_clause(filters: &[TableFilter], params: &mut Vec<String>) -> String {
    if filters.is_empty() {
        return String::new();
    }

    let clauses: Vec<String> = filters
        .iter()
        .map(|filter| {
            let col = quote_ident(&filter.column);
            let idx = params.len() + 1;
            match filter.operator {
                FilterOperator::IsNull => format!("{col} is null"),
                FilterOperator::IsNotNull => format!("{col} is not null"),
                FilterOperator::Equals => {
                    params.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as nvarchar(max)) = @P{idx}")
                }
                FilterOperator::NotEquals => {
                    params.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as nvarchar(max)) <> @P{idx}")
                }
                FilterOperator::Like => {
                    params.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as nvarchar(max)) like @P{idx}")
                }
                FilterOperator::Ilike => {
                    params.push(filter.value.clone().unwrap_or_default().to_lowercase());
                    format!("lower(cast({col} as nvarchar(max))) like @P{idx}")
                }
                FilterOperator::NotLike => {
                    params.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as nvarchar(max)) not like @P{idx}")
                }
                FilterOperator::GreaterThan => {
                    params.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as float) > cast(@P{idx} as float)")
                }
                FilterOperator::GreaterOrEquals => {
                    params.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as float) >= cast(@P{idx} as float)")
                }
                FilterOperator::LessThan => {
                    params.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as float) < cast(@P{idx} as float)")
                }
                FilterOperator::LessOrEquals => {
                    params.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as float) <= cast(@P{idx} as float)")
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
                        return "0=1".to_string();
                    }
                    let placeholders: Vec<String> = items
                        .into_iter()
                        .map(|v| {
                            params.push(v);
                            format!("@P{}", params.len())
                        })
                        .collect();
                    format!("cast({col} as nvarchar(max)) in ({})", placeholders.join(", "))
                }
            }
        })
        .collect();

    format!(" where {}", clauses.join(" and "))
}

fn build_order_by_clause(sort: &[TableSort]) -> String {
    if sort.is_empty() {
        // SQL Server's OFFSET/FETCH requires an ORDER BY clause to be
        // present at all — confirmed live ("Invalid usage of the option
        // NEXT in the FETCH statement" without one). `(SELECT NULL)` is
        // the standard no-op ordering idiom for this case.
        return " order by (select null)".to_string();
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

pub async fn fetch_rows(
    client: &mut MssqlClient,
    schema: &str,
    table: &str,
    limit: i64,
    offset: i64,
    filters: &[TableFilter],
    sort: &[TableSort],
) -> Result<TableRowsResult, AppError> {
    let mut params: Vec<String> = Vec::new();
    let where_clause = build_where_clause(filters, &mut params);
    let order_by_clause = build_order_by_clause(sort);

    let sql = format!(
        "select * from {}.{}{}{} offset {} rows fetch next {} rows only",
        quote_ident(schema),
        quote_ident(table),
        where_clause,
        order_by_clause,
        offset,
        limit
    );

    let mut query = Query::new(sql);
    for p in &params {
        query.bind(p.clone());
    }

    let stream = query.query(client).await.map_err(|e| AppError::new(format!("Failed to fetch rows: {e}")))?;
    let rows = stream.into_first_result().await.map_err(|e| AppError::new(format!("Failed to fetch rows: {e}")))?;

    let columns = rows
        .first()
        .map(|r| r.columns().iter().map(|c| c.name().to_string()).collect::<Vec<_>>())
        .unwrap_or_default();

    let row_count = rows.len();
    let has_more = row_count as i64 == limit;

    let encoded_rows = rows
        .iter()
        .map(|row| row_to_json_cells(row).into_iter().map(crate::domain::query::encode_cell).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    Ok(TableRowsResult { columns, rows: encoded_rows, row_count: row_count as u32, has_more, duration_ms: 0 })
}

pub async fn count_rows(
    client: &mut MssqlClient,
    schema: &str,
    table: &str,
    filters: &[TableFilter],
) -> Result<u64, AppError> {
    let mut params: Vec<String> = Vec::new();
    let where_clause = build_where_clause(filters, &mut params);

    let sql = format!("select count(*) as c from {}.{}{}", quote_ident(schema), quote_ident(table), where_clause);
    let mut query = Query::new(sql);
    for p in &params {
        query.bind(p.clone());
    }

    let row = query
        .query(client)
        .await
        .map_err(|e| AppError::new(format!("Failed to count rows: {e}")))?
        .into_row()
        .await
        .map_err(|e| AppError::new(format!("Failed to count rows: {e}")))?
        .ok_or_else(|| AppError::new("Failed to count rows: no result returned"))?;

    let count: i32 = row.get("c").unwrap_or(0);
    Ok(count.max(0) as u64)
}

/// Same wrap-and-page approach as the other engines' `execute_query` —
/// see `mysql::executor`'s doc comment. SQL Server's own `COUNT(*)
/// OVER()` window function works identically (confirmed live), and the
/// same `ORDER BY (SELECT NULL)` fallback from `fetch_rows` is needed
/// here too since `OFFSET/FETCH` requires an explicit ordering.
pub async fn execute_query(client: &mut MssqlClient, sql: &str, offset: u64, limit: u64) -> Result<RawQueryResult, AppError> {
    if !looks_like_select(sql) {
        let result = Query::new(sql.to_string())
            .execute(client)
            .await
            .map_err(|e| AppError::new(e.to_string()))?;
        return Ok(RawQueryResult::Affected { row_count: result.rows_affected().iter().sum() });
    }

    match execute_query_page(client, sql, offset, limit).await {
        Ok(result) => Ok(result),
        Err(WrapOrRuntimeError::Runtime(e)) => Err(AppError::new(e.to_string())),
        Err(WrapOrRuntimeError::Wrap) => {
            let stream = Query::new(sql.to_string())
                .query(client)
                .await
                .map_err(|e| AppError::new(e.to_string()))?;
            let rows = stream.into_first_result().await.map_err(|e| AppError::new(e.to_string()))?;
            let columns = rows
                .first()
                .map(|r| r.columns().iter().map(|c| c.name().to_string()).collect::<Vec<_>>())
                .unwrap_or_default();
            let json_rows = rows.iter().map(row_to_json_cells).collect::<Vec<_>>();
            Ok(RawQueryResult::Rows { columns, rows: json_rows, total_row_count: None })
        }
    }
}

enum WrapOrRuntimeError {
    Wrap,
    Runtime(tiberius::error::Error),
}

/// tiberius surfaces a parse failure as a generic `Server` error with no
/// stable numeric code exposed through its public API the way
/// Postgres/MySQL's drivers do — so, same as SQLite's executor (see its
/// doc comment), classification here is fallback-and-see: any failure to
/// run the wrapped query is treated as "can't be wrapped" and triggers
/// the unwrapped retry, rather than trying to tell "genuinely can't be
/// wrapped" apart from "wrapped SQL has some other real bug". Both cases
/// end up running `sql` again unwrapped, so the outcome is the same
/// either way.
fn classify_wrap_error(_e: tiberius::error::Error) -> WrapOrRuntimeError {
    WrapOrRuntimeError::Wrap
}

async fn execute_query_page(
    client: &mut MssqlClient,
    sql: &str,
    offset: u64,
    limit: u64,
) -> Result<RawQueryResult, WrapOrRuntimeError> {
    let page_sql = format!(
        "select *, count(*) over() as __total_row_count from ({sql}) as q order by (select null) offset {offset} rows fetch next {limit} rows only"
    );
    let stream = Query::new(page_sql).query(client).await.map_err(classify_wrap_error)?;
    let rows = stream.into_first_result().await.map_err(WrapOrRuntimeError::Runtime)?;

    let all_columns: Vec<String> = rows
        .first()
        .map(|r| r.columns().iter().map(|c| c.name().to_string()).collect::<Vec<_>>())
        .unwrap_or_default();
    let total_col_index = all_columns.len().saturating_sub(1);
    let columns = all_columns.get(..total_col_index).unwrap_or_default().to_vec();

    let total: i64 = match rows.first() {
        Some(row) => {
            let cells: Vec<_> = row.cells().collect();
            match cells.get(total_col_index) {
                Some((_, ColumnData::I32(Some(v)))) => *v as i64,
                Some((_, ColumnData::I64(Some(v)))) => *v,
                _ => 0,
            }
        }
        None => {
            let count_sql = format!("select count(*) as c from ({sql}) as q");
            let count_row = Query::new(count_sql)
                .query(client)
                .await
                .map_err(WrapOrRuntimeError::Runtime)?
                .into_row()
                .await
                .map_err(WrapOrRuntimeError::Runtime)?;
            count_row.and_then(|r| r.get::<i32, _>("c")).unwrap_or(0) as i64
        }
    };

    let json_rows = rows
        .iter()
        .map(|row| row_to_json_cells(row).into_iter().take(total_col_index).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    Ok(RawQueryResult::Rows { columns, rows: json_rows, total_row_count: Some(total as usize) })
}

fn looks_like_select(sql: &str) -> bool {
    let trimmed = sql.trim_start().to_lowercase();
    trimmed.starts_with("select") || trimmed.starts_with("with") || trimmed.starts_with("exec")
}

pub async fn update_cell_text(
    client: &mut MssqlClient,
    schema: &str,
    table: &str,
    pk_values: &[(String, JsonValue)],
    column: &str,
    new_value: Option<&str>,
) -> Result<(), AppError> {
    let mut params: Vec<String> = vec![new_value.unwrap_or_default().to_string()];
    let is_null = new_value.is_none();
    let where_clause = append_pk_params(&mut params, pk_values);

    let sql = format!(
        "update {}.{} set {} = {} where {}",
        quote_ident(schema),
        quote_ident(table),
        quote_ident(column),
        if is_null { "NULL".to_string() } else { "@P1".to_string() },
        where_clause
    );

    let bind_params: Vec<String> = if is_null { params[1..].to_vec() } else { params };
    execute_single_row_update(client, &sql, &bind_params).await
}

pub async fn update_json_cell(
    client: &mut MssqlClient,
    schema: &str,
    table: &str,
    pk_values: &[(String, JsonValue)],
    column: &str,
    value: &JsonValue,
) -> Result<(), AppError> {
    let value_text = value.to_string();
    let mut params: Vec<String> = vec![value_text];
    let where_clause = append_pk_params(&mut params, pk_values);

    let sql = format!(
        "update {}.{} set {} = @P1 where {}",
        quote_ident(schema),
        quote_ident(table),
        quote_ident(column),
        where_clause
    );

    execute_single_row_update(client, &sql, &params).await
}

pub async fn insert_row(
    client: &mut MssqlClient,
    schema: &str,
    table: &str,
    values: &[(String, String)],
) -> Result<(), AppError> {
    if values.is_empty() {
        return Err(AppError::new("Cannot insert a row with no columns."));
    }

    let mut params: Vec<String> = Vec::with_capacity(values.len());
    let mut column_names = Vec::with_capacity(values.len());
    let mut placeholders = Vec::with_capacity(values.len());

    for (column, value) in values {
        params.push(value.clone());
        column_names.push(quote_ident(column));
        placeholders.push(format!("@P{}", params.len()));
    }

    let sql = format!(
        "insert into {}.{} ({}) values ({})",
        quote_ident(schema),
        quote_ident(table),
        column_names.join(", "),
        placeholders.join(", ")
    );

    let mut query = Query::new(sql);
    for p in &params {
        query.bind(p.clone());
    }
    query.execute(client).await.map_err(|e| AppError::new(format!("Failed to insert row: {e}")))?;
    Ok(())
}

pub async fn delete_rows(
    client: &mut MssqlClient,
    schema: &str,
    table: &str,
    rows_pk_values: &[Vec<(String, JsonValue)>],
) -> Result<u64, AppError> {
    if rows_pk_values.is_empty() {
        return Ok(0);
    }

    let mut params: Vec<String> = Vec::new();
    let mut row_clauses: Vec<String> = Vec::with_capacity(rows_pk_values.len());

    for pk_values in rows_pk_values {
        let mut clause_parts = Vec::with_capacity(pk_values.len());
        for (pk_col, pk_value) in pk_values {
            params.push(json_pk_to_text_param(pk_value).unwrap_or_default());
            let idx = params.len();
            clause_parts.push(format!("cast({} as nvarchar(max)) = @P{idx}", quote_ident(pk_col)));
        }
        row_clauses.push(format!("({})", clause_parts.join(" and ")));
    }

    let where_clause = row_clauses.join(" or ");
    let sql = format!("delete from {}.{} where {}", quote_ident(schema), quote_ident(table), where_clause);

    let mut query = Query::new(sql);
    for p in &params {
        query.bind(p.clone());
    }
    let result = query.execute(client).await.map_err(|e| AppError::new(format!("Failed to delete rows: {e}")))?;

    let affected: u64 = result.rows_affected().iter().sum();
    let expected = rows_pk_values.len() as u64;
    if affected != expected {
        return Err(AppError::new(format!(
            "Expected to delete {expected} row(s) but {affected} matched — the data may have changed. Refresh and try again."
        )));
    }

    Ok(affected)
}

fn append_pk_params(params: &mut Vec<String>, pk_values: &[(String, JsonValue)]) -> String {
    let mut where_clause = String::new();
    for (i, (pk_col, pk_value)) in pk_values.iter().enumerate() {
        if i > 0 {
            where_clause.push_str(" and ");
        }
        params.push(json_pk_to_text_param(pk_value).unwrap_or_default());
        let idx = params.len();
        where_clause.push_str(&format!("cast({} as nvarchar(max)) = @P{idx}", quote_ident(pk_col)));
    }
    where_clause
}

async fn execute_single_row_update(client: &mut MssqlClient, sql: &str, params: &[String]) -> Result<(), AppError> {
    let mut query = Query::new(sql.to_string());
    for p in params {
        query.bind(p.clone());
    }
    let result = query.execute(client).await.map_err(|e| AppError::new(format!("Failed to update cell: {e}")))?;

    let affected: u64 = result.rows_affected().iter().sum();
    if affected == 0 {
        return Err(AppError::new("No matching row found — it may have been deleted or modified."));
    }
    if affected > 1 {
        return Err(AppError::new("Update matched more than one row — refusing to apply to avoid unintended changes."));
    }
    Ok(())
}

fn json_pk_to_text_param(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::String(s) => Some(s.clone()),
        JsonValue::Number(n) => Some(n.to_string()),
        JsonValue::Bool(b) => Some(b.to_string()),
        JsonValue::Null => None,
        _ => Some(value.to_string()),
    }
}

/// Converts one row to a `Vec<JsonValue>`, one per column, in column
/// order. Every non-datetime type reads its already-decoded `ColumnData`
/// straight off `row.cells()` — but Date/Time/DateTime2/DateTime/
/// SmallDateTime only implement tiberius's `FromSql` (the trait behind
/// `Row::try_get`), not any conversion from a bare `&ColumnData` value on
/// its own, so those columns are re-read through `try_get` by index
/// instead of matched from `cells()` directly.
fn row_to_json_cells(row: &tiberius::Row) -> Vec<JsonValue> {
    row.columns()
        .iter()
        .enumerate()
        .map(|(idx, col)| match col.column_type() {
            tiberius::ColumnType::Daten => row
                .try_get::<chrono::NaiveDate, _>(idx)
                .ok()
                .flatten()
                .map(|d| JsonValue::String(d.to_string()))
                .unwrap_or(JsonValue::Null),
            tiberius::ColumnType::Timen => row
                .try_get::<chrono::NaiveTime, _>(idx)
                .ok()
                .flatten()
                .map(|t| JsonValue::String(t.to_string()))
                .unwrap_or(JsonValue::Null),
            tiberius::ColumnType::Datetime2
            | tiberius::ColumnType::Datetime
            | tiberius::ColumnType::Datetimen
            | tiberius::ColumnType::Datetime4 => row
                .try_get::<chrono::NaiveDateTime, _>(idx)
                .ok()
                .flatten()
                .map(|d| JsonValue::String(d.to_string()))
                .unwrap_or(JsonValue::Null),
            _ => row.cells().nth(idx).map(|(_, data)| column_data_to_json(data)).unwrap_or(JsonValue::Null),
        })
        .collect()
}

fn column_data_to_json(data: &ColumnData) -> JsonValue {
    match data {
        ColumnData::U8(v) => v.map(|v| JsonValue::from(v)).unwrap_or(JsonValue::Null),
        ColumnData::I16(v) => v.map(|v| JsonValue::from(v)).unwrap_or(JsonValue::Null),
        ColumnData::I32(v) => v.map(|v| JsonValue::from(v)).unwrap_or(JsonValue::Null),
        ColumnData::I64(v) => v.map(|v| JsonValue::from(v)).unwrap_or(JsonValue::Null),
        ColumnData::F32(v) => v.map(|v| JsonValue::from(v)).unwrap_or(JsonValue::Null),
        ColumnData::F64(v) => v.map(|v| JsonValue::from(v)).unwrap_or(JsonValue::Null),
        ColumnData::Bit(v) => v.map(JsonValue::from).unwrap_or(JsonValue::Null),
        ColumnData::String(v) => v.as_ref().map(|s| JsonValue::String(s.to_string())).unwrap_or(JsonValue::Null),
        ColumnData::Guid(v) => v.map(|v| JsonValue::String(v.to_string())).unwrap_or(JsonValue::Null),
        ColumnData::Binary(v) => v
            .as_ref()
            .map(|b| JsonValue::String(format!("\\x{}", b.iter().map(|x| format!("{x:02x}")).collect::<String>())))
            .unwrap_or(JsonValue::Null),
        ColumnData::Numeric(v) => v.map(|d| JsonValue::String(numeric_to_decimal_string(d))).unwrap_or(JsonValue::Null),
        ColumnData::Xml(v) => v.as_ref().map(|x| JsonValue::String(x.to_string())).unwrap_or(JsonValue::Null),
        // Date/Time/DateTime2/DateTime/SmallDateTime are read through
        // `Row::try_get::<chrono::_, _>` at the call site instead of
        // matched here — tiberius exposes `FromSql` (used by
        // `Row::try_get`) for these, not a `TryFrom<ColumnData>` a bare
        // `&ColumnData` value could go through directly, so the
        // row+index is needed, not just the cell's already-extracted
        // `ColumnData`. See `row_to_json_cells`.
        _ => JsonValue::Null,
    }
}

fn numeric_to_decimal_string(n: tiberius::numeric::Numeric) -> String {
    Decimal::from_i128_with_scale(n.value(), n.scale() as u32).to_string()
}
