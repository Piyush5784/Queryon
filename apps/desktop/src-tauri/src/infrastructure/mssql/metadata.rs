use tiberius::Query;

use crate::domain::schema::{
    ColumnInfo, ConstraintInfo, ConstraintKind, ForeignKeyAction, IndexInfo, TableRef,
};
use crate::error::AppError;

use super::pool::MssqlClient;

fn parse_foreign_key_action(rule: &str) -> Option<ForeignKeyAction> {
    match rule.to_uppercase().as_str() {
        "CASCADE" => Some(ForeignKeyAction::Cascade),
        "SET NULL" => Some(ForeignKeyAction::SetNull),
        "SET DEFAULT" => Some(ForeignKeyAction::SetDefault),
        "NO ACTION" => Some(ForeignKeyAction::NoAction),
        // SQL Server has no `RESTRICT` keyword at all for `ON UPDATE`/`ON
        // DELETE` — `NO ACTION` is its closest equivalent and what
        // `INFORMATION_SCHEMA.REFERENTIAL_CONSTRAINTS` reports when a
        // constraint is created with no explicit clause (confirmed live).
        _ => None,
    }
}

pub async fn list_tables(client: &mut MssqlClient) -> Result<Vec<TableRef>, AppError> {
    let rows = client
        .simple_query(
            "SELECT table_schema, table_name, table_type \
             FROM INFORMATION_SCHEMA.TABLES \
             WHERE table_type IN ('BASE TABLE', 'VIEW') \
             ORDER BY table_schema, table_name",
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to list tables: {e}")))?
        .into_first_result()
        .await
        .map_err(|e| AppError::new(format!("Failed to list tables: {e}")))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let table_type: &str = row.get("table_type").unwrap_or("BASE TABLE");
            TableRef {
                schema: row.get::<&str, _>("table_schema").unwrap_or_default().to_string(),
                name: row.get::<&str, _>("table_name").unwrap_or_default().to_string(),
                kind: if table_type == "VIEW" { "view" } else { "table" }.to_string(),
                estimated_rows: 0.0,
            }
        })
        .collect())
}

/// `INFORMATION_SCHEMA.COLUMNS` alone can't tell a real column apart
/// from a computed one (both just have a `data_type`) — this joins
/// `sys.computed_columns` to flag it, matching Beekeeper's own
/// `listTableColumns`. `column_default` comes back wrapped in SQL
/// Server's own literal parens (e.g. `((1))`, `(sysutcdatetime())`),
/// unlike Postgres/MySQL's bare defaults — confirmed live. That
/// wrapping is left as-is rather than stripped: it's valid SQL Server
/// syntax on its own (an `ALTER TABLE ... ADD DEFAULT` clause accepts
/// the parenthesized form directly), so round-tripping it through
/// `get_table_ddl`/DDL rendering needs no extra unwrapping logic.
pub async fn get_table_columns(
    client: &mut MssqlClient,
    schema: &str,
    table: &str,
) -> Result<Vec<ColumnInfo>, AppError> {
    let sql = "
        SELECT
            ic.column_name as column_name,
            CASE
                WHEN character_maximum_length IS NOT NULL AND ic.data_type NOT IN ('text', 'ntext')
                    THEN ic.data_type + '(' + CAST(
                        CASE WHEN character_maximum_length = -1 THEN 'max' ELSE CAST(character_maximum_length AS VARCHAR(16)) END
                    AS VARCHAR(16)) + ')'
                WHEN numeric_precision IS NOT NULL AND ic.data_type IN ('decimal', 'numeric')
                    THEN ic.data_type + '(' + CAST(numeric_precision AS VARCHAR(16)) + ',' + CAST(numeric_scale AS VARCHAR(16)) + ')'
                ELSE ic.data_type
            END as data_type,
            ic.is_nullable as is_nullable,
            ic.column_default as column_default,
            ic.ordinal_position as ordinal_position,
            CASE WHEN sc.definition IS NOT NULL THEN 1 ELSE 0 END as is_generated
        FROM INFORMATION_SCHEMA.COLUMNS ic
        LEFT JOIN sys.computed_columns sc
            ON OBJECT_ID(QUOTENAME(ic.TABLE_SCHEMA) + '.' + QUOTENAME(ic.TABLE_NAME)) = sc.object_id
            AND ic.COLUMN_NAME = sc.name
        WHERE ic.table_schema = @P1 AND ic.table_name = @P2
        ORDER BY ic.ordinal_position";

    let mut query = Query::new(sql);
    query.bind(schema);
    query.bind(table);

    let rows = query
        .query(client)
        .await
        .map_err(|e| AppError::new(format!("Failed to load columns: {e}")))?
        .into_first_result()
        .await
        .map_err(|e| AppError::new(format!("Failed to load columns: {e}")))?;

    let pk_columns = primary_key_columns(client, schema, table).await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let name: String = row.get::<&str, _>("column_name").unwrap_or_default().to_string();
            let is_nullable: &str = row.get("is_nullable").unwrap_or("YES");
            ColumnInfo {
                is_primary_key: pk_columns.contains(&name),
                name,
                data_type: row.get::<&str, _>("data_type").unwrap_or_default().to_string(),
                is_nullable: is_nullable == "YES",
                default: row.get::<&str, _>("column_default").map(|s| s.to_string()),
                ordinal_position: row.get::<i32, _>("ordinal_position").unwrap_or_default(),
            }
        })
        .collect())
}

async fn primary_key_columns(
    client: &mut MssqlClient,
    schema: &str,
    table: &str,
) -> Result<Vec<String>, AppError> {
    let sql = "
        SELECT COLUMN_NAME
        FROM INFORMATION_SCHEMA.KEY_COLUMN_USAGE
        WHERE OBJECTPROPERTY(OBJECT_ID(CONSTRAINT_SCHEMA + '.' + QUOTENAME(CONSTRAINT_NAME)), 'IsPrimaryKey') = 1
            AND TABLE_NAME = @P1 AND TABLE_SCHEMA = @P2";
    let mut query = Query::new(sql);
    query.bind(table);
    query.bind(schema);

    let rows = query
        .query(client)
        .await
        .map_err(|e| AppError::new(format!("Failed to load primary key: {e}")))?
        .into_first_result()
        .await
        .map_err(|e| AppError::new(format!("Failed to load primary key: {e}")))?;

    Ok(rows
        .into_iter()
        .map(|row| row.get::<&str, _>("COLUMN_NAME").unwrap_or_default().to_string())
        .collect())
}

pub async fn list_indexes(
    client: &mut MssqlClient,
    schema: &str,
    table: &str,
) -> Result<Vec<IndexInfo>, AppError> {
    let sql = "
        SELECT
            ind.name as index_name,
            ic.index_column_id as column_id,
            col.name as column_name,
            ind.is_unique as is_unique,
            ind.is_primary_key as is_primary
        FROM sys.indexes ind
        INNER JOIN sys.index_columns ic ON ind.object_id = ic.object_id AND ind.index_id = ic.index_id
        INNER JOIN sys.columns col ON ic.object_id = col.object_id AND ic.column_id = col.column_id
        INNER JOIN sys.tables t ON ind.object_id = t.object_id
        INNER JOIN sys.schemas s ON t.schema_id = s.schema_id
        WHERE t.name = @P1 AND s.name = @P2 AND ind.name IS NOT NULL
        ORDER BY ind.index_id, ic.key_ordinal";

    let mut query = Query::new(sql);
    query.bind(table);
    query.bind(schema);

    let rows = query
        .query(client)
        .await
        .map_err(|e| AppError::new(format!("Failed to list indexes: {e}")))?
        .into_first_result()
        .await
        .map_err(|e| AppError::new(format!("Failed to list indexes: {e}")))?;

    let mut indexes: Vec<IndexInfo> = Vec::new();
    for row in rows {
        let name: String = row.get::<&str, _>("index_name").unwrap_or_default().to_string();
        let column: String = row.get::<&str, _>("column_name").unwrap_or_default().to_string();
        let is_unique: bool = row.get("is_unique").unwrap_or(false);
        let is_primary: bool = row.get("is_primary").unwrap_or(false);

        if let Some(existing) = indexes.iter_mut().find(|i| i.name == name) {
            existing.columns.push(column);
        } else {
            indexes.push(IndexInfo { name, columns: vec![column], is_unique, is_primary });
        }
    }
    Ok(indexes)
}

pub async fn list_constraints(
    client: &mut MssqlClient,
    schema: &str,
    table: &str,
) -> Result<Vec<ConstraintInfo>, AppError> {
    let mut constraints = Vec::new();

    for index in list_indexes(client, schema, table).await? {
        if index.is_primary {
            constraints.push(ConstraintInfo {
                name: index.name,
                kind: ConstraintKind::PrimaryKey,
                columns: index.columns,
                referenced_table: None,
                referenced_columns: Vec::new(),
                on_update: None,
                on_delete: None,
                check_expression: None,
            });
        } else if index.is_unique {
            constraints.push(ConstraintInfo {
                name: index.name,
                kind: ConstraintKind::Unique,
                columns: index.columns,
                referenced_table: None,
                referenced_columns: Vec::new(),
                on_update: None,
                on_delete: None,
                check_expression: None,
            });
        }
    }

    let fk_sql = "
        SELECT
            FK.CONSTRAINT_NAME as name,
            CU.COLUMN_NAME as from_column,
            PK.TABLE_NAME as to_table,
            PT.COLUMN_NAME as to_column,
            C.UPDATE_RULE as on_update,
            C.DELETE_RULE as on_delete
        FROM INFORMATION_SCHEMA.REFERENTIAL_CONSTRAINTS C
        INNER JOIN INFORMATION_SCHEMA.TABLE_CONSTRAINTS FK ON C.CONSTRAINT_NAME = FK.CONSTRAINT_NAME
        INNER JOIN INFORMATION_SCHEMA.TABLE_CONSTRAINTS PK ON C.UNIQUE_CONSTRAINT_NAME = PK.CONSTRAINT_NAME
        INNER JOIN INFORMATION_SCHEMA.KEY_COLUMN_USAGE CU ON C.CONSTRAINT_NAME = CU.CONSTRAINT_NAME
        INNER JOIN (
            SELECT i1.TABLE_NAME, i2.COLUMN_NAME, i2.ORDINAL_POSITION
            FROM INFORMATION_SCHEMA.TABLE_CONSTRAINTS i1
            INNER JOIN INFORMATION_SCHEMA.KEY_COLUMN_USAGE i2 ON i1.CONSTRAINT_NAME = i2.CONSTRAINT_NAME
            WHERE i1.CONSTRAINT_TYPE = 'PRIMARY KEY'
        ) PT ON PT.TABLE_NAME = PK.TABLE_NAME AND CU.ORDINAL_POSITION = PT.ORDINAL_POSITION
        WHERE FK.TABLE_NAME = @P1 AND FK.TABLE_SCHEMA = @P2
        ORDER BY FK.CONSTRAINT_NAME, CU.ORDINAL_POSITION";
    let mut query = Query::new(fk_sql);
    query.bind(table);
    query.bind(schema);

    let fk_rows = query
        .query(client)
        .await
        .map_err(|e| AppError::new(format!("Failed to list foreign keys: {e}")))?
        .into_first_result()
        .await
        .map_err(|e| AppError::new(format!("Failed to list foreign keys: {e}")))?;

    let mut fk_constraints: Vec<ConstraintInfo> = Vec::new();
    for row in fk_rows {
        let name: String = row.get::<&str, _>("name").unwrap_or_default().to_string();
        let from_column: String = row.get::<&str, _>("from_column").unwrap_or_default().to_string();
        let to_column: String = row.get::<&str, _>("to_column").unwrap_or_default().to_string();
        let to_table: String = row.get::<&str, _>("to_table").unwrap_or_default().to_string();
        let on_update: Option<&str> = row.get("on_update");
        let on_delete: Option<&str> = row.get("on_delete");

        if let Some(existing) = fk_constraints.iter_mut().find(|c| c.name == name) {
            existing.columns.push(from_column);
            existing.referenced_columns.push(to_column);
        } else {
            fk_constraints.push(ConstraintInfo {
                name,
                kind: ConstraintKind::ForeignKey,
                columns: vec![from_column],
                referenced_table: Some(to_table),
                referenced_columns: vec![to_column],
                on_update: on_update.and_then(parse_foreign_key_action),
                on_delete: on_delete.and_then(parse_foreign_key_action),
                check_expression: None,
            });
        }
    }
    constraints.extend(fk_constraints);

    let check_sql = "
        SELECT con.name as constraint_name, con.definition as check_clause
        FROM sys.check_constraints con
        INNER JOIN sys.tables t ON con.parent_object_id = t.object_id
        INNER JOIN sys.schemas s ON t.schema_id = s.schema_id
        WHERE t.name = @P1 AND s.name = @P2";
    let mut query = Query::new(check_sql);
    query.bind(table);
    query.bind(schema);

    let check_rows = query
        .query(client)
        .await
        .map_err(|e| AppError::new(format!("Failed to list check constraints: {e}")))?
        .into_first_result()
        .await
        .map_err(|e| AppError::new(format!("Failed to list check constraints: {e}")))?;

    for row in check_rows {
        constraints.push(ConstraintInfo {
            name: row.get::<&str, _>("constraint_name").unwrap_or_default().to_string(),
            kind: ConstraintKind::Check,
            columns: Vec::new(),
            referenced_table: None,
            referenced_columns: Vec::new(),
            on_update: None,
            on_delete: None,
            check_expression: row.get::<&str, _>("check_clause").map(|s| s.to_string()),
        });
    }

    Ok(constraints)
}

/// The name of the `DEFAULT` constraint on `column`, if any — SQL Server
/// names every default as its own separate object (unlike Postgres/
/// MySQL where a default is just a column attribute), and both `ALTER
/// COLUMN` and `DROP COLUMN` fail outright if a default constraint still
/// references the column ("The object ... is dependent on column ...",
/// confirmed live) — so this is looked up before either operation runs.
pub async fn default_constraint_name(
    client: &mut MssqlClient,
    table: &str,
    column: &str,
) -> Result<Option<String>, AppError> {
    let sql = "
        SELECT dc.name
        FROM sys.default_constraints dc
        JOIN sys.columns c ON dc.parent_object_id = c.object_id AND dc.parent_column_id = c.column_id
        JOIN sys.tables t ON dc.parent_object_id = t.object_id
        WHERE t.name = @P1 AND c.name = @P2";
    let mut query = Query::new(sql);
    query.bind(table);
    query.bind(column);

    let row = query
        .query(client)
        .await
        .map_err(|e| AppError::new(format!("Failed to look up default constraint: {e}")))?
        .into_row()
        .await
        .map_err(|e| AppError::new(format!("Failed to look up default constraint: {e}")))?;

    Ok(row.and_then(|r| r.get::<&str, _>("name").map(|s| s.to_string())))
}

/// Every unique/primary-key index name on `column` — `DropColumn` fails
/// the same way `AlterColumn` does when one of these still references
/// the column (confirmed live), so they need dropping first too.
/// One index that references `column`, plus whether SQL Server considers
/// it a constraint object rather than a plain index. This distinction
/// matters for how it has to be dropped: a `UNIQUE`/`PRIMARY KEY`
/// constraint's backing index rejects a plain `DROP INDEX` outright
/// ("An explicit DROP INDEX is not allowed on index ... It is being used
/// for UNIQUE KEY constraint enforcement", confirmed live) and must go
/// through `ALTER TABLE ... DROP CONSTRAINT` instead — a manually
/// `CREATE INDEX`-created index has no such restriction and only accepts
/// `DROP INDEX`. `sys.indexes.is_unique_constraint`/`is_primary_key` are
/// the flags that tell the two apart; a plain `unique` column's index
/// (`is_unique = 1` but `is_unique_constraint = 0`) is NOT a constraint
/// object at all in SQL Server the way it is in Postgres/MySQL/SQLite —
/// `UNIQUE` inline on a column declaration still creates a genuine named
/// constraint here (confirmed live via `sys.indexes` on the seeded
/// schema's `sku nvarchar(100) not null unique` column), so this ends up
/// covering exactly the cases the generic `IndexInfo.is_unique` alone
/// can't distinguish.
pub struct ColumnIndexRef {
    pub name: String,
    pub is_constraint: bool,
}

pub async fn indexes_on_column(
    client: &mut MssqlClient,
    schema: &str,
    table: &str,
    column: &str,
) -> Result<Vec<ColumnIndexRef>, AppError> {
    let sql = "
        SELECT DISTINCT ind.name as index_name, ind.is_unique_constraint, ind.is_primary_key
        FROM sys.indexes ind
        INNER JOIN sys.index_columns ic ON ind.object_id = ic.object_id AND ind.index_id = ic.index_id
        INNER JOIN sys.columns col ON ic.object_id = col.object_id AND ic.column_id = col.column_id
        INNER JOIN sys.tables t ON ind.object_id = t.object_id
        INNER JOIN sys.schemas s ON t.schema_id = s.schema_id
        WHERE t.name = @P1 AND s.name = @P2 AND col.name = @P3
            AND ind.name IS NOT NULL AND ind.is_primary_key = 0";
    let mut query = Query::new(sql);
    query.bind(table);
    query.bind(schema);
    query.bind(column);

    let rows = query
        .query(client)
        .await
        .map_err(|e| AppError::new(format!("Failed to look up indexes on column: {e}")))?
        .into_first_result()
        .await
        .map_err(|e| AppError::new(format!("Failed to look up indexes on column: {e}")))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let is_unique_constraint: bool = row.get("is_unique_constraint").unwrap_or(false);
            ColumnIndexRef {
                name: row.get::<&str, _>("index_name").unwrap_or_default().to_string(),
                is_constraint: is_unique_constraint,
            }
        })
        .collect())
}

pub async fn get_table_ddl(
    client: &mut MssqlClient,
    schema: &str,
    table: &str,
) -> Result<String, AppError> {
    let columns = get_table_columns(client, schema, table).await?;
    if columns.is_empty() {
        return Err(AppError::new(format!("Table '{schema}.{table}' not found.")));
    }

    let column_lines: Vec<String> = columns
        .iter()
        .map(|c| {
            let nullability = if c.is_nullable { "" } else { " not null" };
            let default = c.default.as_ref().map(|d| format!(" default {d}")).unwrap_or_default();
            format!("    {} {}{}{}", quote_ident(&c.name), c.data_type, nullability, default)
        })
        .collect();

    let mut all_lines = column_lines;
    let constraints = list_constraints(client, schema, table).await?;
    for c in &constraints {
        let cols = c.columns.iter().map(|col| quote_ident(col)).collect::<Vec<_>>().join(", ");
        let line = match c.kind {
            ConstraintKind::PrimaryKey => format!("    constraint {} primary key ({})", quote_ident(&c.name), cols),
            ConstraintKind::Unique => format!("    constraint {} unique ({})", quote_ident(&c.name), cols),
            ConstraintKind::ForeignKey => {
                let ref_table = c.referenced_table.as_deref().unwrap_or("?");
                let ref_cols = c.referenced_columns.iter().map(|col| quote_ident(col)).collect::<Vec<_>>().join(", ");
                format!(
                    "    constraint {} foreign key ({}) references {} ({})",
                    quote_ident(&c.name),
                    cols,
                    quote_ident(ref_table),
                    ref_cols
                )
            }
            ConstraintKind::Check => format!(
                "    constraint {} check {}",
                quote_ident(&c.name),
                c.check_expression.clone().unwrap_or_default()
            ),
        };
        all_lines.push(line);
    }

    Ok(format!("create table {}.{} (\n{}\n);", quote_ident(schema), quote_ident(table), all_lines.join(",\n")))
}

pub fn quote_ident(ident: &str) -> String {
    format!("[{}]", ident.replace(']', "]]"))
}
