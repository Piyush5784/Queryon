use tauri::{AppHandle, Wry};
use tauri_plugin_store::StoreExt;

use crate::domain::query::QueryHistoryEntry;
use crate::error::AppError;

const STORE_FILE: &str = "query-history.json";
const HISTORY_KEY: &str = "entries";
const MAX_ENTRIES: usize = 500;

fn read_all(app: &AppHandle) -> Result<Vec<QueryHistoryEntry>, AppError> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| AppError::new(format!("Could not open query history storage: {e}")))?;

    let Some(value) = store.get(HISTORY_KEY) else {
        return Ok(Vec::new());
    };

    serde_json::from_value(value)
        .map_err(|e| AppError::new(format!("Query history file is corrupted: {e}")))
}

fn write_all(app: &AppHandle, entries: &[QueryHistoryEntry]) -> Result<(), AppError> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| AppError::new(format!("Could not open query history storage: {e}")))?;

    let value = serde_json::to_value(entries)
        .map_err(|e| AppError::new(format!("Could not serialize query history: {e}")))?;
    store.set(HISTORY_KEY, value);
    store
        .save()
        .map_err(|e| AppError::new(format!("Could not save query history storage: {e}")))
}

pub fn list_for_connection(
    app: &AppHandle<Wry>,
    connection_id: &str,
) -> Result<Vec<QueryHistoryEntry>, AppError> {
    let mut entries = read_all(app)?;
    entries.retain(|e| e.connection_id == connection_id);
    Ok(entries)
}

pub fn append(app: &AppHandle<Wry>, entry: QueryHistoryEntry) -> Result<(), AppError> {
    let mut entries = read_all(app)?;
    apply_append(&mut entries, entry);
    write_all(app, &entries)
}

pub fn clear_for_connection(app: &AppHandle<Wry>, connection_id: &str) -> Result<(), AppError> {
    let mut entries = read_all(app)?;
    entries.retain(|e| e.connection_id != connection_id);
    write_all(app, &entries)
}

fn apply_append(entries: &mut Vec<QueryHistoryEntry>, entry: QueryHistoryEntry) {
    entries.insert(0, entry);
    entries.truncate(MAX_ENTRIES);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::query::QueryHistoryStatus;

    fn entry(id: &str, connection_id: &str) -> QueryHistoryEntry {
        QueryHistoryEntry {
            id: id.to_string(),
            connection_id: connection_id.to_string(),
            sql: "select 1".to_string(),
            status: QueryHistoryStatus::Success,
            error_message: None,
            row_count: Some(1),
            duration_ms: 5,
            ran_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn append_inserts_at_the_front() {
        let mut entries = vec![entry("a", "conn-1")];
        apply_append(&mut entries, entry("b", "conn-1"));
        assert_eq!(entries[0].id, "b");
        assert_eq!(entries[1].id, "a");
    }

    #[test]
    fn append_truncates_beyond_the_cap() {
        let mut entries: Vec<QueryHistoryEntry> =
            (0..MAX_ENTRIES).map(|i| entry(&i.to_string(), "conn-1")).collect();
        apply_append(&mut entries, entry("newest", "conn-1"));
        assert_eq!(entries.len(), MAX_ENTRIES);
        assert_eq!(entries[0].id, "newest");
    }
}
