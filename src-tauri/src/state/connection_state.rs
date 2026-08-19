use std::collections::HashMap;
use std::sync::Mutex;

use deadpool_postgres::Pool;

#[derive(Default)]
pub struct ConnectionRegistry {
    pools: Mutex<HashMap<String, Pool>>,
}

impl ConnectionRegistry {
    pub fn insert(&self, id: String, pool: Pool) {
        self.pools.lock().unwrap().insert(id, pool);
    }

    pub fn get(&self, id: &str) -> Option<Pool> {
        self.pools.lock().unwrap().get(id).cloned()
    }

    pub fn remove(&self, id: &str) {
        self.pools.lock().unwrap().remove(id);
    }

    pub fn ids(&self) -> Vec<String> {
        self.pools.lock().unwrap().keys().cloned().collect()
    }
}
