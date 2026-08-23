use serde_json::Value as JsonValue;
use sqlx::sqlite::{SqliteArguments, SqlitePool, SqliteRow};
use sqlx::{Arguments, AssertSqlSafe, Column, Row, TypeInfo};

use crate::domain::query::RawQueryResult;
use crate::domain::table::{FilterOperator, SortDirection, TableFilter, TableRowsResult, TableSort};
use crate::error::AppError;

use super::metadata::quote_ident;

fn build_where_clause(filters: &[TableFilter], values: &mut Vec<String>) -> String {
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
                    values.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as text) = ?")
                }
                FilterOperator::NotEquals => {
                    values.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as text) <> ?")
                }
                FilterOperator::Like => {
                    values.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as text) like ?")
                }
                FilterOperator::Ilike => {
                    values.push(filter.value.clone().unwrap_or_default().to_lowercase());
                    format!("lower(cast({col} as text)) like ?")
                }
                FilterOperator::NotLike => {
                    values.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as text) not like ?")
                }
                FilterOperator::GreaterThan => {
                    values.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as real) > cast(? as real)")
                }
                FilterOperator::GreaterOrEquals => {
                    values.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as real) >= cast(? as real)")
                }
                FilterOperator::LessThan => {
                    values.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as real) < cast(? as real)")
                }
                FilterOperator::LessOrEquals => {
                    values.push(filter.value.clone().unwrap_or_default());
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
                            values.push(v);
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

pub async fn fetch_rows(
    pool: &SqlitePool,
    table: &str,
    limit: i64,
    offset: i64,
    filters: &[TableFilter],
    sort: &[TableSort],
) -> Result<TableRowsResult, AppError> {
    let mut filter_values: Vec<String> = Vec::new();
    let where_clause = build_where_clause(filters, &mut filter_values);
    let order_by_clause = build_order_by_clause(sort);

    let sql = format!(
        "select * from {}{}{} limit ? offset ?",
        quote_ident(table),
        where_clause,
        order_by_clause
    );

    let mut query = sqlx::query(AssertSqlSafe(sql));
    for value in &filter_values {
        query = query.bind(value.clone());
    }
    query = query.bind(limit).bind(offset);

    let rows = query
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to fetch rows: {}", clean(&e))))?;

    let columns = rows
        .first()
        .map(|r| r.columns().iter().map(|c| c.name().to_string()).collect::<Vec<_>>())
        .unwrap_or_default();

    let row_count = rows.len();
    let has_more = row_count as i64 == limit;

    let encoded_rows = rows
        .iter()
        .map(|row| {
            (0..row.len())
                .map(|i| crate::domain::query::encode_cell(sqlite_value_to_json(row, i)))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    Ok(TableRowsResult {
        columns,
        rows: encoded_rows,
        row_count: row_count as u32,
        has_more,
        duration_ms: 0,
    })
}

pub async fn count_rows(pool: &SqlitePool, table: &str, filters: &[TableFilter]) -> Result<u64, AppError> {
    let mut filter_values: Vec<String> = Vec::new();
    let where_clause = build_where_clause(filters, &mut filter_values);

    let sql = format!("select count(*) from {}{}", quote_ident(table), where_clause);

    let mut query = sqlx::query(AssertSqlSafe(sql));
    for value in &filter_values {
        query = query.bind(value.clone());
    }

    let row = query
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to count rows: {}", clean(&e))))?;

    let count: i64 = row.try_get(0).map_err(|e| AppError::new(format!("Failed to read row count: {e}")))?;
    Ok(count as u64)
}

/// Same wrap-and-page approach as `mysql::executor::execute_query` — see
/// its doc comment. SQLite's `count(*) over()` window function works
/// identically (confirmed live, SQLite 3.25+, well within the bundled
/// version this crate vendors).
pub async fn execute_query(
    conn: &mut sqlx::SqliteConnection,
    sql: &str,
    offset: u64,
    limit: u64,
) -> Result<RawQueryResult, AppError> {
    if !looks_like_select(sql) {
        let result = sqlx::query(AssertSqlSafe(sql))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::new(clean(&e)))?;
        return Ok(RawQueryResult::Affected { row_count: result.rows_affected() });
    }

    match execute_query_page(conn, sql, offset, limit).await {
        Ok(result) => Ok(result),
        Err(WrapOrRuntimeError::Runtime(e)) => Err(AppError::new(clean(&e))),
        Err(WrapOrRuntimeError::Wrap) => {
            let rows = sqlx::query(AssertSqlSafe(sql))
                .fetch_all(&mut *conn)
                .await
                .map_err(|e| AppError::new(clean(&e)))?;
            let columns = rows
                .first()
                .map(|r| r.columns().iter().map(|c| c.name().to_string()).collect::<Vec<_>>())
                .unwrap_or_default();
            let json_rows = rows
                .iter()
                .map(|row| (0..row.len()).map(|i| sqlite_value_to_json(row, i)).collect::<Vec<_>>())
                .collect::<Vec<_>>();
            Ok(RawQueryResult::Rows { columns, rows: json_rows, total_row_count: None })
        }
    }
}

enum WrapOrRuntimeError {
    Wrap,
    Runtime(sqlx::Error),
}

/// SQLite reports a bare "SQL logic error" for most parse failures with
/// no stable numeric code Postgres/MySQL-style code matching could use
/// (confirmed live: attempting an unwrappable statement inside a
/// subquery raises extended code 1 / `SQLITE_ERROR`, the generic
/// catch-all also used for many unrelated failures) — so classification
/// here is a fallback-and-see: any failure to run the wrapped query is
/// treated as "can't be wrapped" and triggers the unwrapped retry, rather
/// than trying to distinguish "genuinely can't be wrapped" from "wrapped
/// SQL has some other bug". Both cases end up running `sql` again
/// unwrapped, so the outcome is the same either way.
fn classify_wrap_error(_e: sqlx::Error) -> WrapOrRuntimeError {
    WrapOrRuntimeError::Wrap
}

async fn execute_query_page(
    conn: &mut sqlx::SqliteConnection,
    sql: &str,
    offset: u64,
    limit: u64,
) -> Result<RawQueryResult, WrapOrRuntimeError> {
    let page_sql =
        format!("select *, count(*) over() as __total_row_count from ({sql}) as q limit {limit} offset {offset}");
    let rows = sqlx::query(AssertSqlSafe(page_sql))
        .fetch_all(&mut *conn)
        .await
        .map_err(classify_wrap_error)?;

    let all_columns: Vec<String> = rows
        .first()
        .map(|r| r.columns().iter().map(|c| c.name().to_string()).collect::<Vec<_>>())
        .unwrap_or_default();
    let total_col_index = all_columns.len().saturating_sub(1);
    let columns = all_columns.get(..total_col_index).unwrap_or_default().to_vec();

    let total: i64 = match rows.first() {
        Some(row) => row.try_get(total_col_index).map_err(WrapOrRuntimeError::Runtime)?,
        None => {
            let count_sql = format!("select count(*) from ({sql}) as q");
            let count_row = sqlx::query(AssertSqlSafe(count_sql))
                .fetch_one(&mut *conn)
                .await
                .map_err(WrapOrRuntimeError::Runtime)?;
            count_row.try_get(0).map_err(WrapOrRuntimeError::Runtime)?
        }
    };

    let json_rows = rows
        .iter()
        .map(|row| (0..total_col_index).map(|i| sqlite_value_to_json(row, i)).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    Ok(RawQueryResult::Rows {
        columns,
        rows: json_rows,
        total_row_count: Some(total as usize),
    })
}

fn looks_like_select(sql: &str) -> bool {
    let trimmed = sql.trim_start().to_lowercase();
    trimmed.starts_with("select") || trimmed.starts_with("pragma") || trimmed.starts_with("explain")
        || trimmed.starts_with("with")
}

pub async fn update_cell_text(
    pool: &SqlitePool,
    table: &str,
    pk_values: &[(String, JsonValue)],
    column: &str,
    new_value: Option<&str>,
) -> Result<(), AppError> {
    let mut args = SqliteArguments::default();
    let _ = args.add(new_value.map(|s| s.to_string()));
    let where_clause = append_pk_params(&mut args, pk_values);

    let sql = format!(
        "update {} set {} = ? where {}",
        quote_ident(table),
        quote_ident(column),
        where_clause
    );

    execute_single_row_update(pool, &sql, args).await
}

pub async fn update_json_cell(
    pool: &SqlitePool,
    table: &str,
    pk_values: &[(String, JsonValue)],
    column: &str,
    value: &JsonValue,
) -> Result<(), AppError> {
    let value_text = value.to_string();

    let mut args = SqliteArguments::default();
    let _ = args.add(&value_text);
    let where_clause = append_pk_params(&mut args, pk_values);

    let sql = format!(
        "update {} set {} = ? where {}",
        quote_ident(table),
        quote_ident(column),
        where_clause
    );

    execute_single_row_update(pool, &sql, args).await
}

pub async fn insert_row(pool: &SqlitePool, table: &str, values: &[(String, String)]) -> Result<(), AppError> {
    if values.is_empty() {
        return Err(AppError::new("Cannot insert a row with no columns."));
    }

    let mut args = SqliteArguments::default();
    let mut column_names = Vec::with_capacity(values.len());
    let mut placeholders = Vec::with_capacity(values.len());

    for (column, value) in values {
        let _ = args.add(value.clone());
        column_names.push(quote_ident(column));
        placeholders.push("?".to_string());
    }

    let sql = format!(
        "insert into {} ({}) values ({})",
        quote_ident(table),
        column_names.join(", "),
        placeholders.join(", ")
    );

    sqlx::query_with(AssertSqlSafe(sql), args)
        .execute(pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to insert row: {}", clean(&e))))?;

    Ok(())
}

pub async fn delete_rows(
    pool: &SqlitePool,
    table: &str,
    rows_pk_values: &[Vec<(String, JsonValue)>],
) -> Result<u64, AppError> {
    if rows_pk_values.is_empty() {
        return Ok(0);
    }

    let mut args = SqliteArguments::default();
    let mut row_clauses: Vec<String> = Vec::with_capacity(rows_pk_values.len());

    for pk_values in rows_pk_values {
        let mut clause_parts = Vec::with_capacity(pk_values.len());
        for (pk_col, pk_value) in pk_values {
            let _ = args.add(json_pk_to_text_param(pk_value));
            clause_parts.push(format!("cast({} as text) = ?", quote_ident(pk_col)));
        }
        row_clauses.push(format!("({})", clause_parts.join(" and ")));
    }

    let where_clause = row_clauses.join(" or ");

    let sql = format!("delete from {} where {}", quote_ident(table), where_clause);

    let result = sqlx::query_with(AssertSqlSafe(sql), args)
        .execute(pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to delete rows: {}", clean(&e))))?;

    let affected = result.rows_affected();
    let expected = rows_pk_values.len() as u64;
    if affected != expected {
        return Err(AppError::new(format!(
            "Expected to delete {expected} row(s) but {affected} matched — the data may have changed. Refresh and try again."
        )));
    }

    Ok(affected)
}

fn append_pk_params(args: &mut SqliteArguments, pk_values: &[(String, JsonValue)]) -> String {
    let mut where_clause = String::new();
    for (i, (pk_col, pk_value)) in pk_values.iter().enumerate() {
        if i > 0 {
            where_clause.push_str(" and ");
        }
        let _ = args.add(json_pk_to_text_param(pk_value));
        where_clause.push_str(&format!("cast({} as text) = ?", quote_ident(pk_col)));
    }
    where_clause
}

async fn execute_single_row_update(pool: &SqlitePool, sql: &str, args: SqliteArguments) -> Result<(), AppError> {
    let result = sqlx::query_with(AssertSqlSafe(sql), args)
        .execute(pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to update cell: {}", clean(&e))))?;

    let affected = result.rows_affected();
    if affected == 0 {
        return Err(AppError::new(
            "No matching row found — it may have been deleted or modified.",
        ));
    }
    if affected > 1 {
        return Err(AppError::new(
            "Update matched more than one row — refusing to apply to avoid unintended changes.",
        ));
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

fn clean(err: &sqlx::Error) -> String {
    crate::error::describe_sqlite_error(err)
}

/// SQLite is dynamically typed per *value*, not per column the way
/// Postgres/MySQL are — `sqlite3_column_type` (surfaced here as
/// `type_info().name()`) reflects the runtime type of this one cell, so
/// unlike `mysql_value_to_json` this switches per-cell rather than being
/// safe to precompute once per column.
fn sqlite_value_to_json(row: &SqliteRow, idx: usize) -> JsonValue {
    let type_name = row.column(idx).type_info().name();

    macro_rules! try_get {
        ($t:ty) => {
            row.try_get::<Option<$t>, _>(idx).ok().flatten()
        };
    }

    match type_name {
        "NULL" => JsonValue::Null,
        "INTEGER" => try_get!(i64).map(JsonValue::from).unwrap_or(JsonValue::Null),
        "REAL" => try_get!(f64).map(JsonValue::from).unwrap_or(JsonValue::Null),
        "BLOB" => try_get!(Vec<u8>)
            .map(|b| JsonValue::String(format!("\\x{}", hex_encode(&b))))
            .unwrap_or(JsonValue::Null),
        _ => try_get!(String).map(JsonValue::String).unwrap_or(JsonValue::Null),
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
