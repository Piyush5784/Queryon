use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::domain::driver::DatabaseDriver;

#[derive(Default)]
pub struct ConnectionRegistry {
    drivers: Mutex<HashMap<String, Arc<dyn DatabaseDriver>>>,
}

impl ConnectionRegistry {
    pub fn insert(&self, id: String, driver: Arc<dyn DatabaseDriver>) {
        self.drivers.lock().unwrap().insert(id, driver);
    }

    pub fn get(&self, id: &str) -> Option<Arc<dyn DatabaseDriver>> {
        self.drivers.lock().unwrap().get(id).cloned()
    }

    pub fn remove(&self, id: &str) {
        self.drivers.lock().unwrap().remove(id);
    }

    pub fn ids(&self) -> Vec<String> {
        self.drivers.lock().unwrap().keys().cloned().collect()
    }
}
