use rust_decimal::Decimal;
use serde_json::Value as JsonValue;
use sqlx::mysql::{MySqlArguments, MySqlPool, MySqlRow};
use sqlx::{Arguments, AssertSqlSafe, Column, Row, TypeInfo};

use crate::domain::query::RawQueryResult;
use crate::domain::table::{FilterOperator, SortDirection, TableFilter, TableRowsResult, TableSort};
use crate::error::AppError;

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
                    format!("cast({col} as char) = ?")
                }
                FilterOperator::NotEquals => {
                    values.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as char) <> ?")
                }
                FilterOperator::Like => {
                    values.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as char) like ?")
                }
                FilterOperator::Ilike => {
                    values.push(filter.value.clone().unwrap_or_default().to_lowercase());
                    format!("lower(cast({col} as char)) like ?")
                }
                FilterOperator::NotLike => {
                    values.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as char) not like ?")
                }
                FilterOperator::GreaterThan => {
                    values.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as decimal(65,10)) > cast(? as decimal(65,10))")
                }
                FilterOperator::GreaterOrEquals => {
                    values.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as decimal(65,10)) >= cast(? as decimal(65,10))")
                }
                FilterOperator::LessThan => {
                    values.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as decimal(65,10)) < cast(? as decimal(65,10))")
                }
                FilterOperator::LessOrEquals => {
                    values.push(filter.value.clone().unwrap_or_default());
                    format!("cast({col} as decimal(65,10)) <= cast(? as decimal(65,10))")
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
                        return "false".to_string();
                    }
                    let placeholders: Vec<&str> = items
                        .into_iter()
                        .map(|v| {
                            values.push(v);
                            "?"
                        })
                        .collect();
                    format!("cast({col} as char) in ({})", placeholders.join(", "))
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
    pool: &MySqlPool,
    schema: &str,
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
        "select * from {}{}{} limit {limit} offset {offset}",
        quote_qualified(schema, table),
        where_clause,
        order_by_clause
    );

    let mut query = sqlx::query(AssertSqlSafe(sql));
    for value in &filter_values {
        query = query.bind(value.clone());
    }

    let rows = query
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to fetch rows: {}", clean_mysql_error(&e))))?;

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
                .map(|i| crate::domain::query::encode_cell(mysql_value_to_json(row, i)))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    Ok(TableRowsResult {
        columns,
        rows: encoded_rows,
        row_count: row_count as u32,
        has_more,
        // Overwritten by the command layer, which times the full
        // driver-lookup + fetch round trip; see commands/table.rs.
        duration_ms: 0,
    })
}

pub async fn count_rows(
    pool: &MySqlPool,
    schema: &str,
    table: &str,
    filters: &[TableFilter],
) -> Result<u64, AppError> {
    let mut filter_values: Vec<String> = Vec::new();
    let where_clause = build_where_clause(filters, &mut filter_values);

    let sql = format!(
        "select count(*) from {}{}",
        quote_qualified(schema, table),
        where_clause
    );

    let mut query = sqlx::query(AssertSqlSafe(sql));
    for value in &filter_values {
        query = query.bind(value.clone());
    }

    let row = query
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to count rows: {}", clean_mysql_error(&e))))?;

    let count: i64 = row.try_get(0).map_err(|e| AppError::new(format!("Failed to read row count: {e}")))?;
    Ok(count as u64)
}
pub async fn execute_query(
    conn: &mut sqlx::MySqlConnection,
    sql: &str,
    offset: u64,
    limit: u64,
) -> Result<RawQueryResult, AppError> {
    if !looks_like_select(sql) {
        let result = sqlx::query(AssertSqlSafe(sql))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::new(clean_mysql_error(&e)))?;
        return Ok(RawQueryResult::Affected { row_count: result.rows_affected() });
    }

    match execute_query_page(conn, sql, offset, limit).await {
        Ok(result) => Ok(result),
        Err(WrapOrRuntimeError::Runtime(e)) => Err(AppError::new(clean_mysql_error(&e))),
        Err(WrapOrRuntimeError::Wrap) => {
            // `sql` can't be wrapped as a subquery (rare — some
            // statement shapes are only valid at the top level). Fall
            // back to running it unwrapped, once, unpaginated.
            let rows = sqlx::query(AssertSqlSafe(sql))
                .fetch_all(&mut *conn)
                .await
                .map_err(|e| AppError::new(clean_mysql_error(&e)))?;
            let columns = rows
                .first()
                .map(|r| r.columns().iter().map(|c| c.name().to_string()).collect::<Vec<_>>())
                .unwrap_or_default();
            let json_rows = rows
                .iter()
                .map(|row| (0..row.len()).map(|i| mysql_value_to_json(row, i)).collect::<Vec<_>>())
                .collect::<Vec<_>>();
            Ok(RawQueryResult::Rows { columns, rows: json_rows, total_row_count: None })
        }
    }
}
enum WrapOrRuntimeError {
    Wrap,
    Runtime(sqlx::Error),
}

fn classify_wrap_error(e: sqlx::Error) -> WrapOrRuntimeError {
    let is_syntax_error = e
        .as_database_error()
        .and_then(|db_err| db_err.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>())
        .map(|mysql_err| matches!(mysql_err.number(), 1064 | 1149))
        .unwrap_or(false);
    if is_syntax_error {
        WrapOrRuntimeError::Wrap
    } else {
        WrapOrRuntimeError::Runtime(e)
    }
}

async fn execute_query_page(
    conn: &mut sqlx::MySqlConnection,
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

    // With zero matching rows the LIMIT/OFFSET query returns no rows at
    // all, so `count(*) over()` never appears — a plain, uncounted
    // COUNT(*) is the only way to learn the true total in that case.
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
        .map(|row| (0..total_col_index).map(|i| mysql_value_to_json(row, i)).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    Ok(RawQueryResult::Rows {
        columns,
        rows: json_rows,
        total_row_count: Some(total as usize),
    })
}

fn looks_like_select(sql: &str) -> bool {
    let trimmed = sql.trim_start().to_lowercase();
    trimmed.starts_with("select") || trimmed.starts_with("show") || trimmed.starts_with("describe")
        || trimmed.starts_with("explain") || trimmed.starts_with("with")
}

pub async fn update_json_cell(
    pool: &MySqlPool,
    schema: &str,
    table: &str,
    pk_values: &[(String, JsonValue)],
    column: &str,
    value: &JsonValue,
    is_starrocks: bool,
) -> Result<(), AppError> {
    let value_text = value.to_string();

    if is_starrocks {
        let where_clause = starrocks_pk_where_clause(pk_values);
        let sql = format!(
            "update {} set {} = {} where {}",
            quote_qualified(schema, table),
            quote_ident(column),
            quote_literal(&value_text),
            where_clause
        );
        return execute_single_row_update_starrocks(pool, &sql, "update cell").await;
    }

    let mut args = MySqlArguments::default();
    let _ = args.add(&value_text);
    let where_clause = append_pk_params(&mut args, pk_values);

    let sql = format!(
        "update {} set {} = ? where {}",
        quote_qualified(schema, table),
        quote_ident(column),
        where_clause
    );

    execute_single_row_update(pool, &sql, args, "update cell").await
}

pub async fn update_cell_text(
    pool: &MySqlPool,
    schema: &str,
    table: &str,
    pk_values: &[(String, JsonValue)],
    column: &str,
    new_value: Option<&str>,
    is_starrocks: bool,
) -> Result<(), AppError> {
    if is_starrocks {
        let where_clause = starrocks_pk_where_clause(pk_values);
        let value_sql = new_value.map(quote_literal).unwrap_or_else(|| "NULL".to_string());
        let sql = format!(
            "update {} set {} = {} where {}",
            quote_qualified(schema, table),
            quote_ident(column),
            value_sql,
            where_clause
        );
        return execute_single_row_update_starrocks(pool, &sql, "update cell").await;
    }

    let mut args = MySqlArguments::default();
    let _ = args.add(new_value.map(|s| s.to_string()));
    let where_clause = append_pk_params(&mut args, pk_values);

    let sql = format!(
        "update {} set {} = ? where {}",
        quote_qualified(schema, table),
        quote_ident(column),
        where_clause
    );

    execute_single_row_update(pool, &sql, args, "update cell").await
}

/// Inserts one row. Any column left out of `values` falls back to its
/// table default / null.
pub async fn insert_row(
    pool: &MySqlPool,
    schema: &str,
    table: &str,
    values: &[(String, String)],
    is_starrocks: bool,
) -> Result<(), AppError> {
    if values.is_empty() {
        return Err(AppError::new("Cannot insert a row with no columns."));
    }

    if is_starrocks {
        let column_names: Vec<String> = values.iter().map(|(c, _)| quote_ident(c)).collect();
        let literals: Vec<String> = values.iter().map(|(_, v)| quote_literal(v)).collect();
        let sql = format!(
            "insert into {} ({}) values ({});",
            quote_qualified(schema, table),
            column_names.join(", "),
            literals.join(", ")
        );
        sqlx::raw_sql(AssertSqlSafe(sql))
            .execute(pool)
            .await
            .map_err(|e| AppError::new(format!("Failed to insert row: {}", clean_mysql_error(&e))))?;
        return Ok(());
    }

    let mut args = MySqlArguments::default();
    let mut column_names = Vec::with_capacity(values.len());
    let mut placeholders = Vec::with_capacity(values.len());

    for (column, value) in values {
        let _ = args.add(value.clone());
        column_names.push(quote_ident(column));
        placeholders.push("?".to_string());
    }

    let sql = format!(
        "insert into {} ({}) values ({})",
        quote_qualified(schema, table),
        column_names.join(", "),
        placeholders.join(", ")
    );

    sqlx::query_with(AssertSqlSafe(sql), args)
        .execute(pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to insert row: {}", clean_mysql_error(&e))))?;

    Ok(())
}

pub async fn delete_rows(
    pool: &MySqlPool,
    schema: &str,
    table: &str,
    rows_pk_values: &[Vec<(String, JsonValue)>],
    is_starrocks: bool,
) -> Result<u64, AppError> {
    if rows_pk_values.is_empty() {
        return Ok(0);
    }

    if is_starrocks {
        let row_clauses: Vec<String> = rows_pk_values.iter().map(|pk| format!("({})", starrocks_pk_where_clause(pk))).collect();
        let where_clause = row_clauses.join(" or ");
        let sql = format!("delete from {} where {};", quote_qualified(schema, table), where_clause);
        let result = sqlx::raw_sql(AssertSqlSafe(sql))
            .execute(pool)
            .await
            .map_err(|e| AppError::new(format!("Failed to delete rows: {}", clean_mysql_error(&e))))?;
        let affected = result.rows_affected();
        let expected = rows_pk_values.len() as u64;
        if affected != expected {
            return Err(AppError::new(format!(
                "Expected to delete {expected} row(s) but {affected} matched — the data may have changed. Refresh and try again."
            )));
        }
        return Ok(affected);
    }

    let mut args = MySqlArguments::default();
    let mut row_clauses: Vec<String> = Vec::with_capacity(rows_pk_values.len());

    for pk_values in rows_pk_values {
        let mut clause_parts = Vec::with_capacity(pk_values.len());
        for (pk_col, pk_value) in pk_values {
            let _ = args.add(json_pk_to_text_param(pk_value));
            clause_parts.push(format!("cast({} as char) = ?", quote_ident(pk_col)));
        }
        row_clauses.push(format!("({})", clause_parts.join(" and ")));
    }

    let where_clause = row_clauses.join(" or ");

    let sql = format!(
        "delete from {} where {}",
        quote_qualified(schema, table),
        where_clause
    );

    let result = sqlx::query_with(AssertSqlSafe(sql), args)
        .execute(pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to delete rows: {}", clean_mysql_error(&e))))?;

    let affected = result.rows_affected();
    let expected = rows_pk_values.len() as u64;
    if affected != expected {
        return Err(AppError::new(format!(
            "Expected to delete {expected} row(s) but {affected} matched — the data may have changed. Refresh and try again."
        )));
    }

    Ok(affected)
}

fn append_pk_params(args: &mut MySqlArguments, pk_values: &[(String, JsonValue)]) -> String {
    let mut where_clause = String::new();
    for (i, (pk_col, pk_value)) in pk_values.iter().enumerate() {
        if i > 0 {
            where_clause.push_str(" and ");
        }
        let _ = args.add(json_pk_to_text_param(pk_value));
        where_clause.push_str(&format!("cast({} as char) = ?", quote_ident(pk_col)));
    }
    where_clause
}

fn starrocks_pk_where_clause(pk_values: &[(String, JsonValue)]) -> String {
    pk_values
        .iter()
        .map(|(pk_col, pk_value)| {
            let literal = json_pk_to_text_param(pk_value).map(|v| quote_literal(&v)).unwrap_or_else(|| "NULL".to_string());
            format!("cast({} as char) = {}", quote_ident(pk_col), literal)
        })
        .collect::<Vec<_>>()
        .join(" and ")
}

async fn execute_single_row_update(
    pool: &MySqlPool,
    sql: &str,
    args: MySqlArguments,
    action: &str,
) -> Result<(), AppError> {
    let result = sqlx::query_with(AssertSqlSafe(sql), args)
        .execute(pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to {action}: {}", clean_mysql_error(&e))))?;

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

async fn execute_single_row_update_starrocks(pool: &MySqlPool, sql: &str, action: &str) -> Result<(), AppError> {
    let result = sqlx::raw_sql(AssertSqlSafe(sql.to_string()))
        .execute(pool)
        .await
        .map_err(|e| AppError::new(format!("Failed to {action}: {}", clean_mysql_error(&e))))?;

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

fn quote_literal(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
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

fn quote_ident(ident: &str) -> String {
    format!("`{}`", ident.replace('`', "``"))
}

fn quote_qualified(_schema: &str, table: &str) -> String {
    quote_ident(table)
}

fn clean_mysql_error(err: &sqlx::Error) -> String {
    crate::error::describe_mysql_error(err)
}

fn mysql_value_to_json(row: &MySqlRow, idx: usize) -> JsonValue {
    let type_name = row.column(idx).type_info().name();

    macro_rules! try_get {
        ($t:ty) => {
            row.try_get::<Option<$t>, _>(idx).ok().flatten()
        };
    }

    match type_name {
        "BOOLEAN" => try_get!(bool).map(JsonValue::from).unwrap_or(JsonValue::Null),
        "TINYINT" | "TINYINT UNSIGNED" | "SMALLINT" | "SMALLINT UNSIGNED" | "MEDIUMINT"
        | "MEDIUMINT UNSIGNED" | "INT" => try_get!(i32).map(JsonValue::from).unwrap_or(JsonValue::Null),
        "INT UNSIGNED" | "BIGINT" => try_get!(i64).map(JsonValue::from).unwrap_or(JsonValue::Null),
        "BIGINT UNSIGNED" => try_get!(u64).map(JsonValue::from).unwrap_or(JsonValue::Null),
        "FLOAT" => try_get!(f32).map(JsonValue::from).unwrap_or(JsonValue::Null),
        "DOUBLE" => try_get!(f64).map(JsonValue::from).unwrap_or(JsonValue::Null),
        "DECIMAL" => try_get!(Decimal)
            .map(|d| JsonValue::String(d.to_string()))
            .unwrap_or(JsonValue::Null),
        "JSON" => try_get!(sqlx::types::Json<JsonValue>)
            .map(|j| j.0)
            .unwrap_or(JsonValue::Null),
        "DATE" => try_get!(chrono::NaiveDate)
            .map(|v| JsonValue::String(v.to_string()))
            .unwrap_or(JsonValue::Null),
        "TIME" => try_get!(chrono::NaiveTime)
            .map(|v| JsonValue::String(v.to_string()))
            .unwrap_or(JsonValue::Null),
        "DATETIME" => try_get!(chrono::NaiveDateTime)
            .map(|v| JsonValue::String(v.to_string()))
            .unwrap_or(JsonValue::Null),
        "TIMESTAMP" => try_get!(chrono::DateTime<chrono::Utc>)
            .map(|v| JsonValue::String(v.to_rfc3339()))
            .unwrap_or(JsonValue::Null),
        _ => try_get!(String).map(JsonValue::String).unwrap_or(JsonValue::Null),
    }
}
