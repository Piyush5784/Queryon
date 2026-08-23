use tauri::{AppHandle, Wry};
use tauri_plugin_store::StoreExt;

use crate::domain::query::SavedQuery;
use crate::error::AppError;

const STORE_FILE: &str = "saved-queries.json";
const QUERIES_KEY: &str = "queries";

fn read_all(app: &AppHandle) -> Result<Vec<SavedQuery>, AppError> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| AppError::new(format!("Could not open saved queries storage: {e}")))?;

    let Some(value) = store.get(QUERIES_KEY) else {
        return Ok(Vec::new());
    };

    serde_json::from_value(value)
        .map_err(|e| AppError::new(format!("Saved queries file is corrupted: {e}")))
}

fn write_all(app: &AppHandle, queries: &[SavedQuery]) -> Result<(), AppError> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| AppError::new(format!("Could not open saved queries storage: {e}")))?;

    let value = serde_json::to_value(queries)
        .map_err(|e| AppError::new(format!("Could not serialize saved queries: {e}")))?;
    store.set(QUERIES_KEY, value);
    store
        .save()
        .map_err(|e| AppError::new(format!("Could not save saved queries storage: {e}")))
}

pub fn list_for_connection(app: &AppHandle<Wry>, connection_id: &str) -> Result<Vec<SavedQuery>, AppError> {
    let mut queries = read_all(app)?;
    queries.retain(|q| q.connection_id == connection_id);
    Ok(queries)
}

pub fn upsert(app: &AppHandle<Wry>, query: SavedQuery) -> Result<(), AppError> {
    let mut queries = read_all(app)?;
    apply_upsert(&mut queries, query);
    write_all(app, &queries)
}

pub fn remove(app: &AppHandle<Wry>, query_id: &str) -> Result<(), AppError> {
    let mut queries = read_all(app)?;
    apply_remove(&mut queries, query_id);
    write_all(app, &queries)
}

fn apply_upsert(queries: &mut Vec<SavedQuery>, query: SavedQuery) {
    match queries.iter_mut().find(|q| q.id == query.id) {
        Some(existing) => *existing = query,
        None => queries.push(query),
    }
}

fn apply_remove(queries: &mut Vec<SavedQuery>, query_id: &str) {
    queries.retain(|q| q.id != query_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(id: &str, connection_id: &str, title: &str) -> SavedQuery {
        SavedQuery {
            id: id.to_string(),
            connection_id: connection_id.to_string(),
            title: title.to_string(),
            sql: "select 1".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn upsert_appends_a_new_query() {
        let mut queries = vec![query("a", "conn-1", "First")];
        apply_upsert(&mut queries, query("b", "conn-1", "Second"));
        assert_eq!(queries.len(), 2);
    }

    #[test]
    fn upsert_replaces_an_existing_query_by_id() {
        let mut queries = vec![query("a", "conn-1", "First")];
        apply_upsert(&mut queries, query("a", "conn-1", "Renamed"));
        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].title, "Renamed");
    }

    #[test]
    fn remove_deletes_the_matching_query() {
        let mut queries = vec![query("a", "conn-1", "First"), query("b", "conn-1", "Second")];
        apply_remove(&mut queries, "a");
        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].id, "b");
    }
}
