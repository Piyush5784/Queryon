use deadpool_postgres::Client;

use crate::domain::schema::{
    ColumnInfo, ConstraintInfo, ConstraintKind, ForeignKeyAction, IndexInfo, TableRef,
};
use crate::error::{describe_pg_error, AppError};

/// `pg_get_constraintdef` renders a foreign key's referential actions as
/// literal ` ON UPDATE <action>`/` ON DELETE <action>` suffixes on the
/// definition text (e.g. `FOREIGN KEY (x) REFERENCES y(id) ON UPDATE SET
/// NULL ON DELETE CASCADE`), always in that order, entirely absent when
/// unset — confirmed against a live container rather than assumed from
/// docs. There is no separate metadata column for this, so parsing the
/// definition text is the only way to recover it.
fn parse_foreign_key_action(definition: &str, clause: &str) -> Option<ForeignKeyAction> {
    let upper = definition.to_uppercase();
    let marker = format!("ON {clause} ");
    let start = upper.find(&marker)? + marker.len();
    let rest = &upper[start..];
    if rest.starts_with("CASCADE") {
        Some(ForeignKeyAction::Cascade)
    } else if rest.starts_with("SET NULL") {
        Some(ForeignKeyAction::SetNull)
    } else if rest.starts_with("SET DEFAULT") {
        Some(ForeignKeyAction::SetDefault)
    } else if rest.starts_with("RESTRICT") {
        Some(ForeignKeyAction::Restrict)
    } else if rest.starts_with("NO ACTION") {
        Some(ForeignKeyAction::NoAction)
    } else {
        None
    }
}

pub async fn list_tables(client: &Client) -> Result<Vec<TableRef>, AppError> {
    let rows = client
        .query(
            r#"
            select
                n.nspname as schema,
                c.relname as name,
                case c.relkind
                    when 'r' then 'table'
                    when 'v' then 'view'
                    when 'm' then 'materialized_view'
                    else c.relkind::text
                end as kind,
                greatest(c.reltuples, 0)::float8 as estimated_rows
            from pg_catalog.pg_class c
            join pg_catalog.pg_namespace n on n.oid = c.relnamespace
            where c.relkind in ('r', 'v', 'm')
                and n.nspname not in ('pg_catalog', 'information_schema', 'pg_toast')
                and n.nspname not like 'pg_temp_%'
            order by n.nspname, c.relname
            "#,
            &[],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to list tables: {}", describe_pg_error(&e))))?;

    Ok(rows
        .into_iter()
        .map(|row| TableRef {
            schema: row.get("schema"),
            name: row.get("name"),
            kind: row.get("kind"),
            estimated_rows: row.get("estimated_rows"),
        })
        .collect())
}

pub async fn get_table_columns(
    client: &Client,
    schema: &str,
    table: &str,
) -> Result<Vec<ColumnInfo>, AppError> {
    let rows = client
        .query(
            r#"
            select
                col.column_name,
                col.data_type,
                (col.is_nullable = 'YES') as is_nullable,
                col.column_default,
                col.ordinal_position::text as ordinal_position_text,
                exists (
                    select 1
                    from information_schema.table_constraints tc
                    join information_schema.key_column_usage kcu
                        on kcu.constraint_name = tc.constraint_name
                        and kcu.table_schema = tc.table_schema
                        and kcu.table_name = tc.table_name
                    where tc.constraint_type = 'PRIMARY KEY'
                        and tc.table_schema = col.table_schema
                        and tc.table_name = col.table_name
                        and kcu.column_name = col.column_name
                ) as is_primary_key
            from information_schema.columns col
            where col.table_schema = $1 and col.table_name = $2
            order by col.ordinal_position
            "#,
            &[&schema, &table],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to load columns: {}", describe_pg_error(&e))))?;

    rows.into_iter()
        .map(|row| {
            // Real Postgres reports `ordinal_position::int` as `int4` on
            // the wire, CockroachDB's `information_schema` shim reports
            // it as `int8` even after the same cast, and GreengageDB
            // (GPDB6) reports plain `int4` again but tokio_postgres's i64
            // FromSql rejects it outright ("error deserializing column")
            // — no single fixed-width integer type decodes correctly
            // across all three. Casting to text and parsing in Rust is
            // wire-type-agnostic by construction and works for all of
            // them uniformly, the same fix used for the equivalent MySQL
            // cross-engine mismatch in infrastructure::mysql::metadata.
            let ordinal_position: String = row.get("ordinal_position_text");
            let ordinal_position: i32 = ordinal_position.parse().map_err(|_| {
                AppError::new(format!("Invalid ordinal_position: {ordinal_position}"))
            })?;
            Ok(ColumnInfo {
                name: row.get("column_name"),
                data_type: row.get("data_type"),
                is_nullable: row.get("is_nullable"),
                default: row.get("column_default"),
                is_primary_key: row.get("is_primary_key"),
                ordinal_position,
            })
        })
        .collect()
}

pub async fn list_indexes(
    client: &Client,
    schema: &str,
    table: &str,
) -> Result<Vec<IndexInfo>, AppError> {
    let rows = client
        .query(
            r#"
            select
                ic.relname as index_name,
                i.indisunique as is_unique,
                i.indisprimary as is_primary,
                array_agg(a.attname order by k.ord) as columns
            from pg_catalog.pg_index i
            join pg_catalog.pg_class tc on tc.oid = i.indrelid
            join pg_catalog.pg_class ic on ic.oid = i.indexrelid
            join pg_catalog.pg_namespace n on n.oid = tc.relnamespace
            join lateral unnest(i.indkey) with ordinality as k(attnum, ord) on true
            join pg_catalog.pg_attribute a on a.attrelid = tc.oid and a.attnum = k.attnum
            where n.nspname = $1 and tc.relname = $2
            group by ic.relname, i.indisunique, i.indisprimary
            order by ic.relname
            "#,
            &[&schema, &table],
        )
        .await
        .map_err(|e| AppError::new(format!("Failed to list indexes: {}", describe_pg_error(&e))))?;

    Ok(rows
        .into_iter()
        .map(|row| IndexInfo {
            name: row.get("index_name"),
            columns: row.get("columns"),
            is_unique: row.get("is_unique"),
            is_primary: row.get("is_primary"),
        })
        .collect())
}

pub async fn list_constraints(
    client: &Client,
    schema: &str,
    table: &str,
) -> Result<Vec<ConstraintInfo>, AppError> {
    let rows = client
        .query(
            r#"
            select
                con.conname as constraint_name,
                con.contype as constraint_type,
                array(
                    select attname from unnest(con.conkey) with ordinality as ck(attnum, ord)
                    join pg_catalog.pg_attribute a
                        on a.attrelid = con.conrelid and a.attnum = ck.attnum
                    order by ck.ord
                ) as columns,
                nf.nspname as referenced_schema,
                cf.relname as referenced_table,
                array(
                    select attname from unnest(con.confkey) with ordinality as fk(attnum, ord)
                    join pg_catalog.pg_attribute a
                        on a.attrelid = con.confrelid and a.attnum = fk.attnum
                    order by fk.ord
                ) as referenced_columns,
                pg_get_constraintdef(con.oid) as definition
            from pg_catalog.pg_constraint con
            join pg_catalog.pg_class c on c.oid = con.conrelid
            join pg_catalog.pg_namespace n on n.oid = c.relnamespace
            left join pg_catalog.pg_class cf on cf.oid = con.confrelid
            left join pg_catalog.pg_namespace nf on nf.oid = cf.relnamespace
            where n.nspname = $1 and c.relname = $2
                and con.contype in ('p', 'f', 'u', 'c')
            order by con.conname
            "#,
            &[&schema, &table],
        )
        .await
        .map_err(|e| {
            AppError::new(format!(
                "Failed to list constraints: {}",
                describe_pg_error(&e)
            ))
        })?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let constraint_type: i8 = row.get::<_, i8>("constraint_type");
            let kind = match constraint_type as u8 as char {
                'p' => ConstraintKind::PrimaryKey,
                'f' => ConstraintKind::ForeignKey,
                'u' => ConstraintKind::Unique,
                _ => ConstraintKind::Check,
            };
            let definition: String = row.get("definition");
            let is_fk = kind == ConstraintKind::ForeignKey;
            ConstraintInfo {
                name: row.get("constraint_name"),
                kind,
                columns: row.get("columns"),
                referenced_table: row.get("referenced_table"),
                referenced_columns: row.get("referenced_columns"),
                on_update: is_fk
                    .then(|| parse_foreign_key_action(&definition, "UPDATE"))
                    .flatten(),
                on_delete: is_fk
                    .then(|| parse_foreign_key_action(&definition, "DELETE"))
                    .flatten(),
                check_expression: if kind == ConstraintKind::Check {
                    Some(definition)
                } else {
                    None
                },
            }
        })
        .collect())
}

pub async fn get_table_ddl(client: &Client, schema: &str, table: &str) -> Result<String, AppError> {
    let columns = get_table_columns(client, schema, table).await?;
    if columns.is_empty() {
        return Err(AppError::new(format!(
            "Table '{schema}.{table}' not found."
        )));
    }

    let column_lines: Vec<String> = columns
        .iter()
        .map(|c| {
            let nullability = if c.is_nullable { "" } else { " not null" };
            let default = c
                .default
                .as_ref()
                .map(|d| format!(" default {d}"))
                .unwrap_or_default();
            format!(
                "    {} {}{}{}",
                quote_ident(&c.name),
                c.data_type,
                nullability,
                default
            )
        })
        .collect();

    let constraints = list_constraints(client, schema, table).await?;
    let constraint_lines: Vec<String> = constraints
        .iter()
        .map(|c| {
            let cols = c
                .columns
                .iter()
                .map(|col| quote_ident(col))
                .collect::<Vec<_>>()
                .join(", ");
            match c.kind {
                ConstraintKind::PrimaryKey => {
                    format!(
                        "    constraint {} primary key ({})",
                        quote_ident(&c.name),
                        cols
                    )
                }
                ConstraintKind::Unique => {
                    format!("    constraint {} unique ({})", quote_ident(&c.name), cols)
                }
                ConstraintKind::ForeignKey => {
                    let ref_table = c.referenced_table.as_deref().unwrap_or("?");
                    let ref_cols = c
                        .referenced_columns
                        .iter()
                        .map(|col| quote_ident(col))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(
                        "    constraint {} foreign key ({}) references {} ({})",
                        quote_ident(&c.name),
                        cols,
                        quote_ident(ref_table),
                        ref_cols
                    )
                }
                ConstraintKind::Check => {
                    format!(
                        "    constraint {} {}",
                        quote_ident(&c.name),
                        c.check_expression.clone().unwrap_or_default()
                    )
                }
            }
        })
        .collect();

    let mut all_lines = column_lines;
    all_lines.extend(constraint_lines);

    let indexes = list_indexes(client, schema, table).await?;
    let index_lines: Vec<String> = indexes
        .iter()
        .filter(|i| !i.is_primary)
        .map(|i| {
            let unique = if i.is_unique { "unique " } else { "" };
            let cols = i
                .columns
                .iter()
                .map(|c| quote_ident(c))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "create {unique}index {} on {}.{} ({});",
                quote_ident(&i.name),
                quote_ident(schema),
                quote_ident(table),
                cols
            )
        })
        .collect();

    let create_table = format!(
        "create table {}.{} (\n{}\n);",
        quote_ident(schema),
        quote_ident(table),
        all_lines.join(",\n")
    );

    if index_lines.is_empty() {
        Ok(create_table)
    } else {
        Ok(format!("{}\n\n{}", create_table, index_lines.join("\n")))
    }
}

fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}
