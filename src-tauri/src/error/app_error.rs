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

/// Turns a query/execution-level tokio-postgres error into a message a
/// user can act on. Prefers the structured DbError (SQLSTATE + message)
/// Postgres sends back over the generic `Display` impl, which for many
/// errors is just a terse "db error" with no detail attached.
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
