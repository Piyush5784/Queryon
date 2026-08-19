use keyring::Entry;

use crate::error::AppError;

const SERVICE: &str = "com.piyush.queryon";

fn entry(connection_id: &str) -> Result<Entry, AppError> {
    Entry::new(SERVICE, connection_id)
        .map_err(|e| AppError::new(format!("Could not access the system keychain: {e}")))
}

pub fn save_password(connection_id: &str, password: &str) -> Result<(), AppError> {
    entry(connection_id)?
        .set_password(password)
        .map_err(|e| AppError::new(format!("Could not save password to the system keychain: {e}")))
}

pub fn load_password(connection_id: &str) -> Result<String, AppError> {
    entry(connection_id)?.get_password().map_err(|e| match e {
        keyring::Error::NoEntry => AppError::new(
            "No saved password found for this connection — reconnect and save it again.",
        ),
        other => AppError::new(format!("Could not read password from the system keychain: {other}")),
    })
}

pub fn delete_password(connection_id: &str) -> Result<(), AppError> {
    match entry(connection_id)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::new(format!(
            "Could not delete password from the system keychain: {e}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_id(suffix: &str) -> String {
        format!("queryon-test-{suffix}-{}", std::process::id())
    }

    #[test]
    fn saves_and_loads_a_password() {
        let id = test_id("save-load");
        save_password(&id, "s3cret").unwrap();
        assert_eq!(load_password(&id).unwrap(), "s3cret");
        delete_password(&id).unwrap();
    }

    #[test]
    fn overwrites_an_existing_password() {
        let id = test_id("overwrite");
        save_password(&id, "first").unwrap();
        save_password(&id, "second").unwrap();
        assert_eq!(load_password(&id).unwrap(), "second");
        delete_password(&id).unwrap();
    }

    #[test]
    fn loading_a_missing_password_gives_a_clean_error() {
        let id = test_id("missing");
        let err = load_password(&id).unwrap_err();
        assert!(err.to_string().contains("No saved password found"));
    }

    #[test]
    fn delete_is_idempotent_for_a_missing_entry() {
        let id = test_id("delete-missing");
        assert!(delete_password(&id).is_ok());
        assert!(delete_password(&id).is_ok());
    }

    #[test]
    fn delete_removes_the_password_for_later_loads() {
        let id = test_id("delete-then-load");
        save_password(&id, "temp").unwrap();
        delete_password(&id).unwrap();
        assert!(load_password(&id).is_err());
    }
}
