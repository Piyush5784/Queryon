use std::collections::HashMap;
use std::sync::Mutex;

/// The SQL text last run on a tab, kept so a later
/// `db_fetch_query_result_page` call knows what to re-wrap and re-run
/// with a different `OFFSET` — pagination is always a fresh query
/// against the database, never a slice of rows held in memory. Replaced
/// whenever the tab's query is rerun, and dropped when the tab closes.
#[derive(Default)]
pub struct QueryResultCache {
    last_sql: Mutex<HashMap<String, String>>,
}

impl QueryResultCache {
    pub fn store(&self, tab_id: String, sql: String) {
        self.last_sql.lock().unwrap().insert(tab_id, sql);
    }

    pub fn get(&self, tab_id: &str) -> Option<String> {
        self.last_sql.lock().unwrap().get(tab_id).cloned()
    }

    pub fn remove(&self, tab_id: &str) {
        self.last_sql.lock().unwrap().remove(tab_id);
    }
}
