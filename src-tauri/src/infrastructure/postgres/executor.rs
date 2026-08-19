use deadpool_postgres::Client;
use serde_json::Value as JsonValue;
use tokio_postgres::types::{ToSql, Type as PgType};
use tokio_postgres::Row;

use crate::domain::query::{encode_cell, RawQueryResult};
use crate::domain::table::TableRowsResult;
use crate::error::{describe_pg_error, AppError};

pub async fn fetch_rows(
    client: &Client,
    schema: &str,
    table: &str,
    limit: i64,
    offset: i64,
) -> Result<TableRowsResult, AppError> {
    let sql = format!(
        "select * from {}.{} limit $1 offset $2",
        quote_ident(schema),
        quote_ident(table)
    );

    let rows = client
        .query(&sql, &[&limit, &offset])
        .await
        .map_err(|e| AppError::new(format!("Failed to fetch rows: {}", describe_pg_error(&e))))?;

    let columns = rows
        .first()
        .map(|r| {
            r.columns()
                .iter()
                .map(|c| c.name().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let row_count = rows.len();
    let has_more = row_count as i64 == limit;

    let encoded_rows = rows
        .iter()
        .map(|row| {
            (0..row.len())
                .map(|i| encode_cell(row_value_to_json(row, i)))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let columns = if columns.is_empty() {
        describe_columns(client, schema, table).await?
    } else {
        columns
    };

    Ok(TableRowsResult {
        columns,
        rows: encoded_rows,
        row_count: row_count as u32,
        has_more,
    })
}

pub async fn execute_query(client: &Client, sql: &str, max_rows: usize) -> Result<RawQueryResult, AppError> {
    let stmt = client
        .prepare(sql)
        .await
        .map_err(|e| AppError::new(describe_pg_error(&e)))?;

    if stmt.columns().is_empty() {
        let affected = client
            .execute(&stmt, &[])
            .await
            .map_err(|e| AppError::new(describe_pg_error(&e)))?;
        return Ok(RawQueryResult::Affected { row_count: affected });
    }

    let columns = stmt.columns().iter().map(|c| c.name().to_string()).collect();

    let rows = client
        .query(&stmt, &[])
        .await
        .map_err(|e| AppError::new(describe_pg_error(&e)))?;

    let row_count = rows.len();
    let json_rows = rows
        .iter()
        .take(max_rows)
        .map(|row| (0..row.len()).map(|i| row_value_to_json(row, i)).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    Ok(RawQueryResult::Rows {
        columns,
        rows: json_rows,
        row_count,
    })
}

pub async fn update_json_cell(
    client: &Client,
    schema: &str,
    table: &str,
    pk_values: &[(String, JsonValue)],
    column: &str,
    value: &JsonValue,
) -> Result<(), AppError> {
    let mut params: Vec<Box<dyn ToSql + Sync + Send>> = Vec::with_capacity(pk_values.len() + 1);
    params.push(Box::new(value.clone()));

    let where_clause = append_pk_params(&mut params, pk_values);

    let sql = format!(
        "update {}.{} set {} = $1::jsonb where {}",
        quote_ident(schema),
        quote_ident(table),
        quote_ident(column),
        where_clause
    );

    execute_single_row_update(client, &sql, &params, "update cell").await
}

pub async fn update_cell_text(
    client: &Client,
    schema: &str,
    table: &str,
    pk_values: &[(String, JsonValue)],
    column: &str,
    column_type: &str,
    new_value: Option<&str>,
) -> Result<(), AppError> {
    let mut params: Vec<Box<dyn ToSql + Sync + Send>> = Vec::with_capacity(pk_values.len() + 1);
    params.push(Box::new(new_value.map(|s| s.to_string())));

    let where_clause = append_pk_params(&mut params, pk_values);

    let sql = format!(
        "update {}.{} set {} = $1::text::{} where {}",
        quote_ident(schema),
        quote_ident(table),
        quote_ident(column),
        quote_pg_type(column_type),
        where_clause
    );

    execute_single_row_update(client, &sql, &params, "update cell").await
}

pub async fn delete_rows(
    client: &Client,
    schema: &str,
    table: &str,
    rows_pk_values: &[Vec<(String, JsonValue)>],
) -> Result<u64, AppError> {
    if rows_pk_values.is_empty() {
        return Ok(0);
    }

    let mut params: Vec<Box<dyn ToSql + Sync + Send>> = Vec::new();
    let mut row_clauses: Vec<String> = Vec::with_capacity(rows_pk_values.len());

    for pk_values in rows_pk_values {
        let mut clause_parts = Vec::with_capacity(pk_values.len());
        for (pk_col, pk_value) in pk_values {
            params.push(json_pk_to_text_param(pk_value));
            clause_parts.push(format!("{}::text = ${}::text", quote_ident(pk_col), params.len()));
        }
        row_clauses.push(format!("({})", clause_parts.join(" and ")));
    }

    let where_clause = row_clauses.join(" or ");

    let sql = format!(
        "delete from {}.{} where {}",
        quote_ident(schema),
        quote_ident(table),
        where_clause
    );

    let param_refs: Vec<&(dyn ToSql + Sync)> = params
        .iter()
        .map(|p| p.as_ref() as &(dyn ToSql + Sync))
        .collect();

    let affected = client
        .execute(&sql, &param_refs)
        .await
        .map_err(|e| AppError::new(format!("Failed to delete rows: {}", describe_pg_error(&e))))?;

    let expected = rows_pk_values.len() as u64;
    if affected != expected {
        return Err(AppError::new(format!(
            "Expected to delete {expected} row(s) but {affected} matched — the data may have changed. Refresh and try again."
        )));
    }

    Ok(affected)
}

fn append_pk_params(
    params: &mut Vec<Box<dyn ToSql + Sync + Send>>,
    pk_values: &[(String, JsonValue)],
) -> String {
    let mut where_clause = String::new();
    for (i, (pk_col, pk_value)) in pk_values.iter().enumerate() {
        if i > 0 {
            where_clause.push_str(" and ");
        }
        params.push(json_pk_to_text_param(pk_value));
        where_clause.push_str(&format!(
            "{}::text = ${}::text",
            quote_ident(pk_col),
            params.len()
        ));
    }
    where_clause
}

async fn execute_single_row_update(
    client: &Client,
    sql: &str,
    params: &[Box<dyn ToSql + Sync + Send>],
    action: &str,
) -> Result<(), AppError> {
    let param_refs: Vec<&(dyn ToSql + Sync)> = params
        .iter()
        .map(|p| p.as_ref() as &(dyn ToSql + Sync))
        .collect();

    let affected = client
        .execute(sql, &param_refs)
        .await
        .map_err(|e| AppError::new(format!("Failed to {action}: {}", describe_pg_error(&e))))?;

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

/// Only allows identifier-shaped Postgres type names (covers every builtin
/// scalar type: text, integer, boolean, numeric, timestamptz, uuid, etc.)
/// so `column_type` — sourced from information_schema, not raw user input —
/// can never be used to break out of the generated SQL even if that
/// assumption were ever violated.
fn quote_pg_type(type_name: &str) -> &str {
    let is_safe = !type_name.is_empty()
        && type_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ' ');
    if is_safe {
        type_name
    } else {
        "text"
    }
}

/// Converts a JSON primary-key value into its text representation, since
/// `update_cell`'s WHERE clause always compares as `col::text = $n::text`
/// (works uniformly across bigint/uuid/text PKs without per-type binding).
fn json_pk_to_text_param(value: &JsonValue) -> Box<dyn ToSql + Sync + Send> {
    let text = match value {
        JsonValue::String(s) => s.clone(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Null => return Box::new(Option::<String>::None),
        _ => value.to_string(),
    };
    Box::new(text)
}

async fn describe_columns(client: &Client, schema: &str, table: &str) -> Result<Vec<String>, AppError> {
    let sql = format!(
        "select * from {}.{} limit 0",
        quote_ident(schema),
        quote_ident(table)
    );
    let stmt = client
        .prepare(&sql)
        .await
        .map_err(|e| AppError::new(format!("Failed to describe table: {}", describe_pg_error(&e))))?;
    Ok(stmt.columns().iter().map(|c| c.name().to_string()).collect())
}

fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

fn row_value_to_json(row: &Row, idx: usize) -> JsonValue {
    let col_type = row.columns()[idx].type_();

    macro_rules! try_get {
        ($t:ty) => {
            row.try_get::<_, Option<$t>>(idx).ok().flatten()
        };
    }

    match *col_type {
        PgType::BOOL => try_get!(bool).map(JsonValue::from).unwrap_or(JsonValue::Null),
        PgType::INT2 => try_get!(i16).map(JsonValue::from).unwrap_or(JsonValue::Null),
        PgType::INT4 => try_get!(i32).map(JsonValue::from).unwrap_or(JsonValue::Null),
        PgType::INT8 => try_get!(i64).map(JsonValue::from).unwrap_or(JsonValue::Null),
        PgType::FLOAT4 => try_get!(f32).map(JsonValue::from).unwrap_or(JsonValue::Null),
        PgType::FLOAT8 => try_get!(f64).map(JsonValue::from).unwrap_or(JsonValue::Null),
        PgType::NUMERIC => row
            .try_get::<_, Option<PgNumeric>>(idx)
            .ok()
            .flatten()
            .map(|d| JsonValue::String(d.0))
            .unwrap_or(JsonValue::Null),
        PgType::JSON | PgType::JSONB => {
            try_get!(JsonValue).unwrap_or(JsonValue::Null)
        }
        PgType::TIMESTAMP => try_get!(chrono::NaiveDateTime)
            .map(|v| JsonValue::String(v.to_string()))
            .unwrap_or(JsonValue::Null),
        PgType::TIMESTAMPTZ => try_get!(chrono::DateTime<chrono::Utc>)
            .map(|v| JsonValue::String(v.to_rfc3339()))
            .unwrap_or(JsonValue::Null),
        PgType::DATE => try_get!(chrono::NaiveDate)
            .map(|v| JsonValue::String(v.to_string()))
            .unwrap_or(JsonValue::Null),
        PgType::UUID => try_get!(uuid::Uuid)
            .map(|v| JsonValue::String(v.to_string()))
            .unwrap_or(JsonValue::Null),
        PgType::TEXT | PgType::VARCHAR | PgType::BPCHAR | PgType::NAME => {
            try_get!(String).map(JsonValue::String).unwrap_or(JsonValue::Null)
        }
        _ => try_get!(String)
            .map(JsonValue::String)
            .unwrap_or(JsonValue::Null),
    }
}

struct PgNumeric(String);

impl<'a> tokio_postgres::types::FromSql<'a> for PgNumeric {
    fn from_sql(
        _ty: &PgType,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        if raw.len() < 8 {
            return Err("invalid numeric: too short".into());
        }

        let ndigits = i16::from_be_bytes([raw[0], raw[1]]) as usize;
        let weight = i16::from_be_bytes([raw[2], raw[3]]);
        let sign = u16::from_be_bytes([raw[4], raw[5]]);
        let dscale = i16::from_be_bytes([raw[6], raw[7]]) as usize;

        const NUMERIC_NAN: u16 = 0xC000;
        const NUMERIC_PINF: u16 = 0xD000;
        const NUMERIC_NINF: u16 = 0xF000;
        if sign == NUMERIC_NAN {
            return Ok(PgNumeric("NaN".to_string()));
        }
        if sign == NUMERIC_PINF {
            return Ok(PgNumeric("Infinity".to_string()));
        }
        if sign == NUMERIC_NINF {
            return Ok(PgNumeric("-Infinity".to_string()));
        }

        let mut digits = Vec::with_capacity(ndigits);
        for i in 0..ndigits {
            let offset = 8 + i * 2;
            if raw.len() < offset + 2 {
                return Err("invalid numeric: truncated digits".into());
            }
            digits.push(i16::from_be_bytes([raw[offset], raw[offset + 1]]));
        }

        let mut int_part = String::new();
        let mut frac_part = String::new();

        for (i, d) in digits.iter().enumerate() {
            let exp = weight as i32 - i as i32;
            let group = format!("{:04}", d);
            if exp >= 0 {
                int_part.push_str(&group);
            } else {
                frac_part.push_str(&group);
            }
        }

        if int_part.is_empty() {
            int_part.push('0');
        }
        let int_part = int_part.trim_start_matches('0');
        let int_part = if int_part.is_empty() { "0" } else { int_part };

        let mut result = String::new();
        if sign == 0x4000 {
            result.push('-');
        }
        result.push_str(int_part);

        if dscale > 0 {
            frac_part.truncate(frac_part.len().max(dscale));
            while frac_part.len() < dscale {
                frac_part.push('0');
            }
            frac_part.truncate(dscale);
            result.push('.');
            result.push_str(&frac_part);
        }

        Ok(PgNumeric(result))
    }

    fn accepts(ty: &PgType) -> bool {
        *ty == PgType::NUMERIC
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_postgres::types::FromSql;

    fn build_numeric_wire(
        digits: &[i16],
        weight: i16,
        sign: u16,
        dscale: i16,
    ) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(digits.len() as i16).to_be_bytes());
        buf.extend_from_slice(&weight.to_be_bytes());
        buf.extend_from_slice(&sign.to_be_bytes());
        buf.extend_from_slice(&dscale.to_be_bytes());
        for d in digits {
            buf.extend_from_slice(&d.to_be_bytes());
        }
        buf
    }

    fn decode(raw: &[u8]) -> String {
        PgNumeric::from_sql(&PgType::NUMERIC, raw).unwrap().0
    }

    #[test]
    fn decodes_positive_integer() {
        // 123 => one base-10000 digit group [123], weight 0, no scale
        let raw = build_numeric_wire(&[123], 0, 0x0000, 0);
        assert_eq!(decode(&raw), "123");
    }

    #[test]
    fn decodes_negative_integer() {
        let raw = build_numeric_wire(&[42], 0, 0x4000, 0);
        assert_eq!(decode(&raw), "-42");
    }

    #[test]
    fn decodes_decimal_value() {
        // 123.45 => digit groups [123, 4500], weight 0, dscale 2
        let raw = build_numeric_wire(&[123, 4500], 0, 0x0000, 2);
        assert_eq!(decode(&raw), "123.45");
    }

    #[test]
    fn decodes_zero() {
        let raw = build_numeric_wire(&[], 0, 0x0000, 0);
        assert_eq!(decode(&raw), "0");
    }

    #[test]
    fn decodes_small_fraction_with_leading_zero() {
        // 0.05 => digit group [500] at weight -1, dscale 2
        let raw = build_numeric_wire(&[500], -1, 0x0000, 2);
        assert_eq!(decode(&raw), "0.05");
    }

    #[test]
    fn decodes_nan() {
        let raw = build_numeric_wire(&[], 0, 0xC000, 0);
        assert_eq!(decode(&raw), "NaN");
    }

    #[test]
    fn decodes_positive_infinity() {
        let raw = build_numeric_wire(&[], 0, 0xD000, 0);
        assert_eq!(decode(&raw), "Infinity");
    }

    #[test]
    fn decodes_negative_infinity() {
        let raw = build_numeric_wire(&[], 0, 0xF000, 0);
        assert_eq!(decode(&raw), "-Infinity");
    }

    #[test]
    fn rejects_truncated_header() {
        let raw = vec![0u8; 4];
        assert!(PgNumeric::from_sql(&PgType::NUMERIC, &raw).is_err());
    }

    #[test]
    fn rejects_truncated_digits() {
        // Header claims 2 digit groups but only provides bytes for one.
        let mut raw = build_numeric_wire(&[123], 0, 0x0000, 0);
        raw[1] = 2;
        assert!(PgNumeric::from_sql(&PgType::NUMERIC, &raw).is_err());
    }

    #[test]
    fn quote_ident_wraps_and_escapes_double_quotes() {
        assert_eq!(quote_ident("users"), "\"users\"");
        assert_eq!(quote_ident("weird\"name"), "\"weird\"\"name\"");
    }

    #[test]
    fn json_pk_to_text_param_handles_each_json_type() {
        // These should not panic and should produce a boxed ToSql value
        // for every JSON variant a primary key column might have.
        let _ = json_pk_to_text_param(&JsonValue::String("abc".into()));
        let _ = json_pk_to_text_param(&JsonValue::from(42i64));
        let _ = json_pk_to_text_param(&JsonValue::from(3.15_f64));
        let _ = json_pk_to_text_param(&JsonValue::Bool(true));
        let _ = json_pk_to_text_param(&JsonValue::Null);
    }
}
