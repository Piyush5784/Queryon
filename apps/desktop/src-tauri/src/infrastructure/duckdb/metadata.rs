use duckdb::Connection;

use crate::domain::schema::{ColumnInfo, ConstraintInfo, ConstraintKind, ForeignKeyAction, IndexInfo, TableRef};
use crate::error::AppError;

fn clean(e: duckdb::Error) -> AppError {
    AppError::new(e.to_string())
}

fn parse_fk_action(rule: &str) -> Option<ForeignKeyAction> {
    match rule.to_uppercase().as_str() {
        "CASCADE" => Some(ForeignKeyAction::Cascade),
        "SET NULL" => Some(ForeignKeyAction::SetNull),
        "SET DEFAULT" => Some(ForeignKeyAction::SetDefault),
        "RESTRICT" => Some(ForeignKeyAction::Restrict),
        "NO ACTION" => Some(ForeignKeyAction::NoAction),
        _ => None,
    }
}

pub fn list_tables(conn: &Connection) -> Result<Vec<TableRef>, AppError> {
    let mut stmt = conn
        .prepare("select schema_name, table_name, estimated_size from duckdb_tables order by table_name")
        .map_err(clean)?;

    let rows = stmt
        .query_map([], |row| {
            let schema: String = row.get(0)?;
            let name: String = row.get(1)?;
            let estimated_rows: Option<i64> = row.get(2)?;
            Ok(TableRef {
                schema,
                name,
                kind: "table".to_string(),
                estimated_rows: estimated_rows.unwrap_or(0) as f64,
            })
        })
        .map_err(clean)?;

    let mut tables = Vec::new();
    for row in rows {
        tables.push(row.map_err(clean)?);
    }

    let mut view_stmt = conn.prepare("select schema_name, view_name from duckdb_views order by view_name").map_err(clean)?;
    let view_rows = view_stmt
        .query_map([], |row| {
            let schema: String = row.get(0)?;
            let name: String = row.get(1)?;
            Ok(TableRef { schema, name, kind: "view".to_string(), estimated_rows: 0.0 })
        })
        .map_err(clean)?;
    for row in view_rows {
        tables.push(row.map_err(clean)?);
    }

    Ok(tables)
}

pub fn get_table_columns(conn: &Connection, table: &str) -> Result<Vec<ColumnInfo>, AppError> {
    let mut stmt = conn
        .prepare(
            "select column_name, column_index, data_type, is_nullable, column_default \
             from duckdb_columns where table_name = ? order by column_index",
        )
        .map_err(clean)?;

    let pk_columns = primary_key_columns(conn, table)?;

    let rows = stmt
        .query_map([table], |row| {
            let name: String = row.get(0)?;
            let ordinal_position: i32 = row.get(1)?;
            let data_type: String = row.get(2)?;
            let is_nullable: bool = row.get(3)?;
            let default: Option<String> = row.get(4)?;
            Ok((name, ordinal_position, data_type, is_nullable, default))
        })
        .map_err(clean)?;

    let mut columns = Vec::new();
    for row in rows {
        let (name, ordinal_position, data_type, is_nullable, default) = row.map_err(clean)?;
        let is_primary_key = pk_columns.contains(&name);
        columns.push(ColumnInfo { name, data_type, is_nullable, default, is_primary_key, ordinal_position });
    }
    Ok(columns)
}

fn primary_key_columns(conn: &Connection, table: &str) -> Result<Vec<String>, AppError> {
    let mut stmt = conn
        .prepare(
            "select constraint_column_names from duckdb_constraints \
             where table_name = ? and constraint_type = 'PRIMARY KEY'",
        )
        .map_err(clean)?;

    let rows = stmt
        .query_map([table], |row| {
            let names: duckdb::types::Value = row.get(0)?;
            Ok(names)
        })
        .map_err(clean)?;

    let mut columns = Vec::new();
    for row in rows {
        let value = row.map_err(clean)?;
        if let duckdb::types::Value::List(items) = value {
            for item in items {
                if let duckdb::types::Value::Text(name) = item {
                    columns.push(name);
                }
            }
        }
    }
    Ok(columns)
}

fn parse_index_columns(sql: &str) -> Vec<String> {
    let mut depth = 0i32;
    let mut start = None;
    let mut content = "";
    for (i, ch) in sql.char_indices() {
        match ch {
            '(' => {
                if depth == 0 {
                    start = Some(i + 1);
                }
                depth += 1;
            }
            ')' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(s) = start {
                        content = &sql[s..i];
                    }
                    break;
                }
            }
            _ => {}
        }
    }
    content.split(',').map(|c| c.trim().trim_matches('"').to_string()).filter(|c| !c.is_empty()).collect()
}

pub fn list_indexes(conn: &Connection, table: &str) -> Result<Vec<IndexInfo>, AppError> {
    let mut stmt = conn
        .prepare("select index_name, is_unique, is_primary, sql from duckdb_indexes where table_name = ?")
        .map_err(clean)?;

    let rows = stmt
        .query_map([table], |row| {
            let name: String = row.get(0)?;
            let is_unique: bool = row.get(1)?;
            let is_primary: bool = row.get(2)?;
            let sql: String = row.get(3)?;
            Ok((name, is_unique, is_primary, sql))
        })
        .map_err(clean)?;

    let mut indexes = Vec::new();
    for row in rows {
        let (name, is_unique, is_primary, sql) = row.map_err(clean)?;
        indexes.push(IndexInfo { name, columns: parse_index_columns(&sql), is_unique, is_primary });
    }
    Ok(indexes)
}

pub fn list_constraints(conn: &Connection, table: &str) -> Result<Vec<ConstraintInfo>, AppError> {
    let mut constraints = Vec::new();

    let mut stmt = conn
        .prepare(
            "select constraint_type, constraint_column_names, expression from duckdb_constraints where table_name = ?",
        )
        .map_err(clean)?;
    let rows = stmt
        .query_map([table], |row| {
            let kind: String = row.get(0)?;
            let columns: duckdb::types::Value = row.get(1)?;
            let expression: Option<String> = row.get(2)?;
            Ok((kind, columns, expression))
        })
        .map_err(clean)?;

    for row in rows {
        let (kind, columns_value, expression) = row.map_err(clean)?;
        let columns = list_value_to_strings(columns_value);

        match kind.as_str() {
            "PRIMARY KEY" => constraints.push(ConstraintInfo {
                name: format!("{table}_pkey"),
                kind: ConstraintKind::PrimaryKey,
                columns,
                referenced_table: None,
                referenced_columns: Vec::new(),
                on_update: None,
                on_delete: None,
                check_expression: None,
            }),
            "UNIQUE" => constraints.push(ConstraintInfo {
                name: format!("{table}_{}_key", columns.join("_")),
                kind: ConstraintKind::Unique,
                columns,
                referenced_table: None,
                referenced_columns: Vec::new(),
                on_update: None,
                on_delete: None,
                check_expression: None,
            }),
            "CHECK" => constraints.push(ConstraintInfo {
                name: format!("{table}_{}_check", columns.join("_")),
                kind: ConstraintKind::Check,
                columns,
                referenced_table: None,
                referenced_columns: Vec::new(),
                on_update: None,
                on_delete: None,
                check_expression: expression,
            }),
            _ => {}
        }
    }

    let mut fk_stmt = conn
        .prepare(
            "select tc.constraint_name, kcu1.column_name, kcu2.table_name, kcu2.column_name, rc.update_rule, rc.delete_rule \
             from information_schema.table_constraints tc \
             join information_schema.referential_constraints rc on tc.constraint_name = rc.constraint_name \
             join information_schema.key_column_usage kcu1 on kcu1.constraint_name = tc.constraint_name \
             join information_schema.key_column_usage kcu2 on kcu2.constraint_name = rc.unique_constraint_name and kcu2.ordinal_position = kcu1.ordinal_position \
             where tc.table_name = ? and tc.constraint_type = 'FOREIGN KEY'",
        )
        .map_err(clean)?;
    let fk_rows = fk_stmt
        .query_map([table], |row| {
            let name: String = row.get(0)?;
            let from_col: String = row.get(1)?;
            let to_table: String = row.get(2)?;
            let to_col: String = row.get(3)?;
            let update_rule: String = row.get(4)?;
            let delete_rule: String = row.get(5)?;
            Ok((name, from_col, to_table, to_col, update_rule, delete_rule))
        })
        .map_err(clean)?;

    let mut grouped: std::collections::HashMap<String, ConstraintInfo> = std::collections::HashMap::new();
    for row in fk_rows {
        let (name, from_col, to_table, to_col, update_rule, delete_rule) = row.map_err(clean)?;
        let entry = grouped.entry(name.clone()).or_insert_with(|| ConstraintInfo {
            name: name.clone(),
            kind: ConstraintKind::ForeignKey,
            columns: Vec::new(),
            referenced_table: Some(to_table.clone()),
            referenced_columns: Vec::new(),
            on_update: parse_fk_action(&update_rule),
            on_delete: parse_fk_action(&delete_rule),
            check_expression: None,
        });
        entry.columns.push(from_col);
        entry.referenced_columns.push(to_col);
    }
    constraints.extend(grouped.into_values());

    Ok(constraints)
}

fn list_value_to_strings(value: duckdb::types::Value) -> Vec<String> {
    match value {
        duckdb::types::Value::List(items) => items
            .into_iter()
            .filter_map(|item| if let duckdb::types::Value::Text(s) = item { Some(s) } else { None })
            .collect(),
        duckdb::types::Value::Text(s) => vec![s],
        _ => Vec::new(),
    }
}

pub fn get_table_ddl(conn: &Connection, table: &str) -> Result<String, AppError> {
    let mut stmt = conn.prepare("select sql from duckdb_tables where table_name = ?").map_err(clean)?;
    let sql: String = stmt.query_row([table], |row| row.get(0)).map_err(clean)?;
    Ok(sql)
}
