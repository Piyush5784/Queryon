use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::domain::document::DocumentDriver;

struct ActiveDocumentConnection {
    driver: Arc<dyn DocumentDriver>,
    default_database: String,
}

#[derive(Default)]
pub struct DocumentConnectionRegistry {
    connections: Mutex<HashMap<String, ActiveDocumentConnection>>,
}

impl DocumentConnectionRegistry {
    pub fn insert(&self, id: String, driver: Arc<dyn DocumentDriver>, default_database: String) {
        self.connections
            .lock()
            .unwrap()
            .insert(id, ActiveDocumentConnection { driver, default_database });
    }

    pub fn get(&self, id: &str) -> Option<Arc<dyn DocumentDriver>> {
        self.connections
            .lock()
            .unwrap()
            .get(id)
            .map(|c| c.driver.clone())
    }

    pub fn default_database(&self, id: &str) -> Option<String> {
        self.connections
            .lock()
            .unwrap()
            .get(id)
            .map(|c| c.default_database.clone())
    }

    pub fn remove(&self, id: &str) {
        self.connections.lock().unwrap().remove(id);
    }

    pub fn ids(&self) -> Vec<String> {
        self.connections.lock().unwrap().keys().cloned().collect()
    }
}
