use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::domain::driver::DatabaseDriver;
use crate::error::AppError;
use crate::infrastructure::ssh::SshTunnel;

struct ActiveConnection {
    driver: Arc<dyn DatabaseDriver>,
    read_only: bool,
    /// Kept alive only to hold its forwarding task open for as long as
    /// this connection is active — never read after `insert`. Dropped
    /// (and the tunnel torn down with it) when this entry is removed,
    /// e.g. via `remove` on disconnect.
    _ssh_tunnel: Option<SshTunnel>,
}

#[derive(Default)]
pub struct ConnectionRegistry {
    connections: Mutex<HashMap<String, ActiveConnection>>,
}

impl ConnectionRegistry {
    pub fn insert(&self, id: String, driver: Arc<dyn DatabaseDriver>, read_only: bool) {
        self.connections
            .lock()
            .unwrap()
            .insert(id, ActiveConnection { driver, read_only, _ssh_tunnel: None });
    }

    /// Like `insert`, but also keeps `ssh_tunnel`'s forwarding task alive
    /// for the life of this connection — see `ActiveConnection::_ssh_tunnel`.
    pub fn insert_with_tunnel(
        &self,
        id: String,
        driver: Arc<dyn DatabaseDriver>,
        read_only: bool,
        ssh_tunnel: SshTunnel,
    ) {
        self.connections
            .lock()
            .unwrap()
            .insert(id, ActiveConnection { driver, read_only, _ssh_tunnel: Some(ssh_tunnel) });
    }

    pub fn get(&self, id: &str) -> Option<Arc<dyn DatabaseDriver>> {
        self.connections
            .lock()
            .unwrap()
            .get(id)
            .map(|c| c.driver.clone())
    }

    /// Looks up the driver for `id` and rejects if the connection is
    /// missing or was opened read-only. Every write command (schema DDL,
    /// row insert/update/delete) must go through this instead of `get`
    /// — a single choke point means a new write command can't forget the
    /// read-only check the way it could if each command re-implemented
    /// its own `is_read_only` branch.
    pub fn require_writable(&self, id: &str) -> Result<Arc<dyn DatabaseDriver>, AppError> {
        let connections = self.connections.lock().unwrap();
        let connection = connections
            .get(id)
            .ok_or_else(|| AppError::new("Not connected — reconnect and try again."))?;
        if connection.read_only {
            return Err(AppError::new(
                "This connection is read-only — writes are disabled. Turn off read-only mode in the connection's settings to make changes.",
            ));
        }
        Ok(connection.driver.clone())
    }

    pub fn remove(&self, id: &str) {
        self.connections.lock().unwrap().remove(id);
    }

    pub fn ids(&self) -> Vec<String> {
        self.connections.lock().unwrap().keys().cloned().collect()
    }
}
