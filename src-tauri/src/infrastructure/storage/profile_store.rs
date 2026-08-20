use tauri::{AppHandle, Wry};
use tauri_plugin_store::StoreExt;

use crate::domain::connection::SavedConnectionProfile;
use crate::error::AppError;

const STORE_FILE: &str = "connections.json";
const PROFILES_KEY: &str = "profiles";

fn read_all(app: &AppHandle) -> Result<Vec<SavedConnectionProfile>, AppError> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| AppError::new(format!("Could not open connection storage: {e}")))?;

    let Some(value) = store.get(PROFILES_KEY) else {
        return Ok(Vec::new());
    };

    serde_json::from_value(value)
        .map_err(|e| AppError::new(format!("Saved connections file is corrupted: {e}")))
}

fn write_all(app: &AppHandle, profiles: &[SavedConnectionProfile]) -> Result<(), AppError> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| AppError::new(format!("Could not open connection storage: {e}")))?;

    let value = serde_json::to_value(profiles)
        .map_err(|e| AppError::new(format!("Could not serialize connection profiles: {e}")))?;
    store.set(PROFILES_KEY, value);
    store
        .save()
        .map_err(|e| AppError::new(format!("Could not save connection storage: {e}")))
}

pub fn list(app: &AppHandle<Wry>) -> Result<Vec<SavedConnectionProfile>, AppError> {
    read_all(app)
}

pub fn upsert(app: &AppHandle<Wry>, profile: SavedConnectionProfile) -> Result<(), AppError> {
    let mut profiles = read_all(app)?;
    apply_upsert(&mut profiles, profile);
    write_all(app, &profiles)
}

pub fn remove(app: &AppHandle<Wry>, connection_id: &str) -> Result<(), AppError> {
    let mut profiles = read_all(app)?;
    apply_remove(&mut profiles, connection_id);
    write_all(app, &profiles)
}

/// Updates only a saved profile's display name — never touches the
/// password, which lives separately in `credential_vault`.
pub fn rename(app: &AppHandle<Wry>, connection_id: &str, name: &str) -> Result<(), AppError> {
    let mut profiles = read_all(app)?;
    apply_rename(&mut profiles, connection_id, name)?;
    write_all(app, &profiles)
}

fn apply_upsert(profiles: &mut Vec<SavedConnectionProfile>, profile: SavedConnectionProfile) {
    match profiles.iter_mut().find(|p| p.id == profile.id) {
        Some(existing) => *existing = profile,
        None => profiles.push(profile),
    }
}

fn apply_rename(
    profiles: &mut [SavedConnectionProfile],
    connection_id: &str,
    name: &str,
) -> Result<(), AppError> {
    let profile = profiles
        .iter_mut()
        .find(|p| p.id == connection_id)
        .ok_or_else(|| AppError::new("No saved connection found with this id."))?;
    profile.name = name.to_string();
    Ok(())
}

fn apply_remove(profiles: &mut Vec<SavedConnectionProfile>, connection_id: &str) {
    profiles.retain(|p| p.id != connection_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::connection::{Engine, SslMode};

    fn profile(id: &str, name: &str) -> SavedConnectionProfile {
        SavedConnectionProfile {
            id: id.to_string(),
            name: name.to_string(),
            engine: Engine::Postgres,
            host: "localhost".to_string(),
            port: 5432,
            database: "devdb".to_string(),
            user: "devuser".to_string(),
            ssl_mode: SslMode::Disable,
        }
    }

    #[test]
    fn upsert_appends_a_new_profile() {
        let mut profiles = vec![profile("a", "First")];
        apply_upsert(&mut profiles, profile("b", "Second"));
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[1].name, "Second");
    }

    #[test]
    fn upsert_replaces_an_existing_profile_by_id() {
        let mut profiles = vec![profile("a", "First")];
        apply_upsert(&mut profiles, profile("a", "Renamed"));
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "Renamed");
    }

    #[test]
    fn remove_deletes_the_matching_profile() {
        let mut profiles = vec![profile("a", "First"), profile("b", "Second")];
        apply_remove(&mut profiles, "a");
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].id, "b");
    }

    #[test]
    fn remove_is_a_no_op_for_an_unknown_id() {
        let mut profiles = vec![profile("a", "First")];
        apply_remove(&mut profiles, "does-not-exist");
        assert_eq!(profiles.len(), 1);
    }

    #[test]
    fn rename_updates_only_the_matching_profiles_name() {
        let mut profiles = vec![profile("a", "First"), profile("b", "Second")];
        apply_rename(&mut profiles, "a", "Renamed").expect("rename failed");
        assert_eq!(profiles[0].name, "Renamed");
        assert_eq!(profiles[1].name, "Second");
    }

    #[test]
    fn rename_errors_for_an_unknown_id() {
        let mut profiles = vec![profile("a", "First")];
        assert!(apply_rename(&mut profiles, "does-not-exist", "Renamed").is_err());
    }
}
