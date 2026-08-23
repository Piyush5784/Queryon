use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct ExportJobRegistry {
    cancel_flags: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl ExportJobRegistry {
    pub fn register(&self, job_id: String) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        self.cancel_flags.lock().unwrap().insert(job_id, flag.clone());
        flag
    }

    pub fn cancel(&self, job_id: &str) {
        if let Some(flag) = self.cancel_flags.lock().unwrap().get(job_id) {
            flag.store(true, Ordering::Relaxed);
        }
    }

    pub fn remove(&self, job_id: &str) {
        self.cancel_flags.lock().unwrap().remove(job_id);
    }
}
