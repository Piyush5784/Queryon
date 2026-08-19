use std::io::Write;

use serde_json::Value as JsonValue;

use crate::error::AppError;

use super::models::ExportFormat;

/// Incrementally writes rows to an open file in the chosen format.
/// Each `write_rows` call appends one chunk; `finish` closes out any
/// format-specific trailing syntax (only JSON needs one, for its `]`).
pub struct ExportWriter<W: Write> {
    format: ExportFormat,
    pretty_print: bool,
    writer: W,
    table_name: String,
    columns: Vec<String>,
    rows_written: u64,
}

impl<W: Write> ExportWriter<W> {
    pub fn new(
        mut writer: W,
        format: ExportFormat,
        pretty_print: bool,
        columns: Vec<String>,
        table_name: String,
    ) -> Result<Self, AppError> {
        match format {
            ExportFormat::Csv => {
                writeln!(writer, "{}", columns.iter().map(|c| csv_field(c)).collect::<Vec<_>>().join(","))
                    .map_err(io_err)?;
            }
            ExportFormat::Json => {
                write!(writer, "[").map_err(io_err)?;
            }
            ExportFormat::Sql => {}
        }

        Ok(Self {
            format,
            pretty_print,
            writer,
            table_name,
            columns,
            rows_written: 0,
        })
    }

    pub fn write_rows(&mut self, rows: &[Vec<JsonValue>]) -> Result<(), AppError> {
        for row in rows {
            match self.format {
                ExportFormat::Csv => self.write_csv_row(row)?,
                ExportFormat::Json => self.write_json_row(row)?,
                ExportFormat::Sql => self.write_sql_row(row)?,
            }
            self.rows_written += 1;
        }
        Ok(())
    }

    pub fn finish(mut self) -> Result<u64, AppError> {
        if self.format == ExportFormat::Json {
            write!(self.writer, "{}", if self.pretty_print { "\n]" } else { "]" }).map_err(io_err)?;
        }
        self.writer.flush().map_err(io_err)?;
        Ok(self.rows_written)
    }

    fn write_csv_row(&mut self, row: &[JsonValue]) -> Result<(), AppError> {
        let line = row.iter().map(|v| csv_field(&cell_display(v))).collect::<Vec<_>>().join(",");
        writeln!(self.writer, "{line}").map_err(io_err)
    }

    fn write_json_row(&mut self, row: &[JsonValue]) -> Result<(), AppError> {
        let mut obj = serde_json::Map::new();
        for (col, value) in self.columns.iter().zip(row) {
            obj.insert(col.clone(), value.clone());
        }
        let value = JsonValue::Object(obj);

        let text = if self.pretty_print {
            serde_json::to_string_pretty(&value)
        } else {
            serde_json::to_string(&value)
        }
        .map_err(|e| AppError::new(format!("Failed to encode row as JSON: {e}")))?;

        let indented = if self.pretty_print {
            text.lines().map(|l| format!("  {l}")).collect::<Vec<_>>().join("\n")
        } else {
            text
        };

        let prefix = if self.rows_written == 0 {
            if self.pretty_print { "\n" } else { "" }
        } else if self.pretty_print {
            ",\n"
        } else {
            ","
        };

        write!(self.writer, "{prefix}{indented}").map_err(io_err)
    }

    fn write_sql_row(&mut self, row: &[JsonValue]) -> Result<(), AppError> {
        let column_list = self.columns.iter().map(|c| quote_sql_ident(c)).collect::<Vec<_>>().join(", ");
        let values = row.iter().map(sql_literal).collect::<Vec<_>>().join(", ");
        writeln!(
            self.writer,
            "INSERT INTO {} ({column_list}) VALUES ({values});",
            quote_sql_ident(&self.table_name)
        )
        .map_err(io_err)
    }
}

fn io_err(e: std::io::Error) -> AppError {
    AppError::new(format!("Failed to write file: {e}"))
}

fn csv_field(value: &str) -> String {
    if value.contains(['"', ',', '\r', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn cell_display(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => String::new(),
        JsonValue::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn quote_sql_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

fn sql_literal(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => "NULL".to_string(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
        JsonValue::String(s) => format!("'{}'", s.replace('\'', "''")),
        other => format!("'{}'", other.to_string().replace('\'', "''")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn run(format: ExportFormat, pretty_print: bool, rows: &[Vec<JsonValue>]) -> String {
        let mut buf: Vec<u8> = Vec::new();
        {
            let mut writer = ExportWriter::new(
                &mut buf,
                format,
                pretty_print,
                vec!["id".to_string(), "name".to_string()],
                "users".to_string(),
            )
            .unwrap();
            writer.write_rows(rows).unwrap();
            writer.finish().unwrap();
        }
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn csv_writes_header_and_rows() {
        let out = run(
            ExportFormat::Csv,
            false,
            &[vec![json!(1), json!("Alice")], vec![json!(2), json!("Bob")]],
        );
        assert_eq!(out, "id,name\n1,Alice\n2,Bob\n");
    }

    #[test]
    fn csv_quotes_fields_containing_commas_or_quotes() {
        let out = run(ExportFormat::Csv, false, &[vec![json!(1), json!("Smith, \"Bob\"")]]);
        assert_eq!(out, "id,name\n1,\"Smith, \"\"Bob\"\"\"\n");
    }

    #[test]
    fn csv_renders_null_as_empty_field() {
        let out = run(ExportFormat::Csv, false, &[vec![json!(1), JsonValue::Null]]);
        assert_eq!(out, "id,name\n1,\n");
    }

    #[test]
    fn json_produces_a_valid_array_of_objects() {
        let out = run(
            ExportFormat::Json,
            false,
            &[vec![json!(1), json!("Alice")], vec![json!(2), json!("Bob")]],
        );
        let parsed: JsonValue = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed, json!([{ "id": 1, "name": "Alice" }, { "id": 2, "name": "Bob" }]));
    }

    #[test]
    fn json_with_zero_rows_is_an_empty_array() {
        let out = run(ExportFormat::Json, false, &[]);
        assert_eq!(out, "[]");
    }

    #[test]
    fn json_pretty_print_produces_valid_json_too() {
        let out = run(ExportFormat::Json, true, &[vec![json!(1), json!("Alice")]]);
        let parsed: JsonValue = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed, json!([{ "id": 1, "name": "Alice" }]));
        assert!(out.contains('\n'), "pretty-printed output should contain newlines");
    }

    #[test]
    fn sql_produces_one_insert_statement_per_row() {
        let out = run(
            ExportFormat::Sql,
            false,
            &[vec![json!(1), json!("Alice")], vec![json!(2), json!(JsonValue::Null)]],
        );
        assert_eq!(
            out,
            "INSERT INTO \"users\" (\"id\", \"name\") VALUES (1, 'Alice');\nINSERT INTO \"users\" (\"id\", \"name\") VALUES (2, NULL);\n"
        );
    }

    #[test]
    fn sql_escapes_single_quotes_in_string_values() {
        let out = run(ExportFormat::Sql, false, &[vec![json!(1), json!("O'Brien")]]);
        assert!(out.contains("'O''Brien'"), "expected escaped quote, got: {out}");
    }

    #[test]
    fn write_rows_can_be_called_multiple_times_across_chunks() {
        let mut buf: Vec<u8> = Vec::new();
        {
            let mut writer = ExportWriter::new(
                &mut buf,
                ExportFormat::Csv,
                false,
                vec!["id".to_string(), "name".to_string()],
                "users".to_string(),
            )
            .unwrap();
            writer.write_rows(&[vec![json!(1), json!("Alice")]]).unwrap();
            writer.write_rows(&[vec![json!(2), json!("Bob")]]).unwrap();
            let total = writer.finish().unwrap();
            assert_eq!(total, 2);
        }
        assert_eq!(String::from_utf8(buf).unwrap(), "id,name\n1,Alice\n2,Bob\n");
    }
}
