use serde::Serialize;
use specta::Type;

#[derive(Debug, Serialize, Type)]
#[serde(transparent)]
pub struct AppError(String);

impl AppError {
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AppError {}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for AppError {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

pub fn describe_pg_error(err: &tokio_postgres::Error) -> String {
    let Some(db_error) = err.as_db_error() else {
        return clean_postgres_error(&err.to_string());
    };

    use tokio_postgres::error::SqlState;

    let subject = db_error
        .column()
        .or(db_error.constraint())
        .map(|s| format!(" '{s}'"))
        .unwrap_or_default();

    match *db_error.code() {
        SqlState::NOT_NULL_VIOLATION => {
            format!("Column{subject} cannot be empty — it does not allow NULL.")
        }
        SqlState::FOREIGN_KEY_VIOLATION => format!(
            "This value{subject} doesn't match any row in the referenced table — check the related record exists."
        ),
        SqlState::UNIQUE_VIOLATION => {
            format!("A row with this value{subject} already exists — it must be unique.")
        }
        SqlState::CHECK_VIOLATION => {
            format!("This value violates a check constraint{subject}.")
        }
        SqlState::INVALID_TEXT_REPRESENTATION => {
            format!("This value isn't valid for the column's type{subject}.")
        }
        SqlState::NUMERIC_VALUE_OUT_OF_RANGE => {
            format!("This value is out of range for the column's type{subject}.")
        }
        SqlState::STRING_DATA_RIGHT_TRUNCATION => {
            format!("This value is too long for the column{subject}.")
        }
        _ => db_error.message().to_string(),
    }
}
pub fn describe_mysql_error(err: &sqlx::Error) -> String {
    let Some(db_error) = err.as_database_error() else {
        return clean_mysql_error(&err.to_string());
    };
    let Some(mysql_error) = db_error.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>() else {
        return clean_mysql_error(&db_error.to_string());
    };

    match mysql_error.number() {
        1048 => "Column cannot be empty — it does not allow NULL.".to_string(),
        1451 | 1452 => {
            "This value doesn't match any row in the referenced table — check the related record exists."
                .to_string()
        }
        1062 => "A row with this value already exists — it must be unique.".to_string(),
        3819 | 4025 => "This value violates a check constraint.".to_string(),
        1406 => "This value is too long for the column.".to_string(),
        1264 => "This value is out of range for the column's type.".to_string(),
        1366 => "This value isn't valid for the column's type.".to_string(),
        _ => mysql_error.message().to_string(),
    }
}

pub fn describe_sqlite_error(err: &sqlx::Error) -> String {
    let Some(db_error) = err.as_database_error() else {
        return clean_sqlite_error(&err.to_string());
    };
    use sqlx::error::DatabaseError;

    let Some(sqlite_error) = db_error.try_downcast_ref::<sqlx::sqlite::SqliteError>() else {
        return clean_sqlite_error(&db_error.to_string());
    };

    let code: i32 = sqlite_error
        .code()
        .and_then(|c| c.parse().ok())
        .unwrap_or(0);

    match code {
        2067 | 1555 => "A row with this value already exists — it must be unique.".to_string(),
        787 => {
            "This value doesn't match any row in the referenced table — check the related record exists."
                .to_string()
        }
        1299 => "Column cannot be empty — it does not allow NULL.".to_string(),
        275 => "This value violates a check constraint.".to_string(),
        _ => clean_sqlite_error(sqlite_error.message()),
    }
}

pub fn describe_libsql_error(err: &libsql::Error) -> String {
    clean_libsql_error(&err.to_string())
}

pub fn clean_libsql_error(raw: &str) -> String {
    if raw.contains("UNIQUE constraint failed") {
        "A row with this value already exists — it must be unique.".to_string()
    } else if raw.contains("FOREIGN KEY constraint failed") {
        "This value doesn't match any row in the referenced table — check the related record exists."
            .to_string()
    } else if raw.contains("NOT NULL constraint failed") {
        "Column cannot be empty — it does not allow NULL.".to_string()
    } else if raw.contains("CHECK constraint failed") {
        "This value violates a check constraint.".to_string()
    } else if raw.contains("Connection refused") || raw.contains("error sending request") {
        "Connection refused — check the server URL and that sqld/Turso is reachable.".to_string()
    } else if raw.contains("401") || raw.to_lowercase().contains("unauthorized") {
        "Authentication failed — check the auth token.".to_string()
    } else if raw.contains("timed out") {
        "Connection timed out — check the server URL and your network/firewall.".to_string()
    } else {
        raw.to_string()
    }
}

pub fn describe_mongodb_error(err: &mongodb::error::Error) -> String {
    clean_mongodb_error(&err.to_string())
}

pub fn clean_mongodb_error(raw: &str) -> String {
    if raw.contains("Authentication failed") || raw.contains("bad auth") {
        "Authentication failed — check your username and password.".to_string()
    } else if raw.contains("ServerSelectionTimeout") || raw.contains("server selection timeout") {
        "Could not reach the MongoDB server — check the connection string and your network/firewall."
            .to_string()
    } else if raw.contains("Connection refused") {
        "Connection refused — check the host and that the server is running.".to_string()
    } else if raw.contains("timed out") {
        "Connection timed out — check the connection string and your network/firewall.".to_string()
    } else if raw.contains("InvalidUri") || raw.contains("invalid uri") {
        "Invalid MongoDB connection string.".to_string()
    } else {
        raw.to_string()
    }
}

pub fn describe_trino_error(err: &trino_rust_client::error::Error) -> String {
    use trino_rust_client::error::Error as TrinoError;
    match err {
        TrinoError::Query(query_error) => query_error.message.clone(),
        TrinoError::Forbidden { message } => format!("Permission denied: {message}"),
        TrinoError::HttpError(_) | TrinoError::HttpNotOk(_, _) => {
            "Connection refused — check the host and port and that Trino is reachable.".to_string()
        }
        TrinoError::Transaction(message) => message.clone(),
        TrinoError::Protocol(message) => message.clone(),
        TrinoError::Decode(message) => format!("Failed to read the server's response: {message}"),
        other => other.to_string(),
    }
}

pub fn clean_sqlite_error(raw: &str) -> String {
    if raw.contains("unable to open database file") {
        "Could not open the database file — check the path exists and is readable.".to_string()
    } else if raw.contains("database is locked") {
        "Database is locked by another connection — close other programs using this file and try again."
            .to_string()
    } else if raw.contains("file is not a database") {
        "This file is not a valid SQLite database.".to_string()
    } else {
        raw.to_string()
    }
}

pub fn clean_mysql_error(raw: &str) -> String {
    if raw.contains("Access denied") {
        "Authentication failed — check your username and password.".to_string()
    } else if raw.contains("Connection refused") {
        "Connection refused — check the host and port, and that the server is running."
            .to_string()
    } else if raw.contains("timed out") {
        "Connection timed out — check the host and port, and your network/firewall.".to_string()
    } else if raw.contains("Unknown database") {
        "Database does not exist.".to_string()
    } else {
        raw.to_string()
    }
}

pub fn clean_postgres_error(raw: &str) -> String {
    if raw.contains("password authentication failed") {
        "Authentication failed — check your username and password.".to_string()
    } else if raw.contains("Connection refused") {
        "Connection refused — check the host and port, and that the server is running."
            .to_string()
    } else if raw.contains("timed out") {
        "Connection timed out — check the host and port, and your network/firewall.".to_string()
    } else if raw.contains("does not exist") && raw.contains("database") {
        "Database does not exist.".to_string()
    } else {
        raw.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_error_displays_its_message() {
        let err = AppError::new("something failed");
        assert_eq!(err.to_string(), "something failed");
    }

    #[test]
    fn app_error_serializes_as_plain_string() {
        let err = AppError::new("boom");
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, "\"boom\"");
    }

    #[test]
    fn app_error_from_string_and_str() {
        let a: AppError = "from &str".into();
        let b: AppError = String::from("from String").into();
        assert_eq!(a.to_string(), "from &str");
        assert_eq!(b.to_string(), "from String");
    }

    #[test]
    fn clean_postgres_error_recognizes_auth_failure() {
        let raw = "db error: FATAL: password authentication failed for user \"devuser\"";
        assert_eq!(
            clean_postgres_error(raw),
            "Authentication failed — check your username and password."
        );
    }

    #[test]
    fn clean_postgres_error_recognizes_connection_refused() {
        let raw = "error connecting to server: Connection refused (os error 111)";
        assert_eq!(
            clean_postgres_error(raw),
            "Connection refused — check the host and port, and that the server is running."
        );
    }

    #[test]
    fn clean_postgres_error_recognizes_timeout() {
        let raw = "db error: connection timed out";
        assert_eq!(
            clean_postgres_error(raw),
            "Connection timed out — check the host and port, and your network/firewall."
        );
    }

    #[test]
    fn clean_postgres_error_recognizes_missing_database() {
        let raw = "FATAL: database \"doesnotexist\" does not exist";
        assert_eq!(clean_postgres_error(raw), "Database does not exist.");
    }

    #[test]
    fn clean_postgres_error_falls_back_to_raw_message() {
        let raw = "some other unrecognized postgres error";
        assert_eq!(clean_postgres_error(raw), raw);
    }
}
