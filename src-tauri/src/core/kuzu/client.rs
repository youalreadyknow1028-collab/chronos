//! KuzuDB Bitemporal Client (stub)
//! In-process graph database with bitemporal versioning
//! See EPIC_3 §3.1
//!
//! NOTE: kuzu crate not in Cargo.toml — stub implementation for compile check.
//! In production, add: kuzu = { version = "0.4", features = [] } to Cargo.toml

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

// Stub connection type — replace with actual kuzu::Connection when crate is available
pub struct StubConnection;

pub struct KuzuClient {
    // In production: conn: Arc<Mutex<Connection>>
    _conn: Arc<Mutex<StubConnection>>,
    db_path: String,
}

impl KuzuClient {
    /// Open or create KuzuDB at the given path
    pub fn new(db_path: &str) -> Result<Self, String> {
        info!("KuzuDB stub: would open at {}", db_path);
        Ok(Self {
            _conn: Arc::new(Mutex::new(StubConnection)),
            db_path: db_path.to_string(),
        })
    }

    /// Initialize schema (tables + relationships)
    pub async fn init(&self) -> Result<(), String> {
        info!("KuzuDB schema init (stub)");
        Ok(())
    }

    /// Get a locked connection for queries
    pub async fn conn(&self) -> Arc<Mutex<StubConnection>> {
        self._conn.clone()
    }

    /// Close the database
    pub async fn close(&self) {
        info!("KuzuDB closing: {}", self.db_path);
    }
}

impl Clone for KuzuClient {
    fn clone(&self) -> Self {
        Self {
            _conn: self._conn.clone(),
            db_path: self.db_path.clone(),
        }
    }
}
