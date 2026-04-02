use super::schema::DatabaseSchema;
use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

pub struct DbConnection {
    conn: Arc<Mutex<Connection>>,
}

impl DbConnection {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = 10000;
            PRAGMA temp_store = memory;
            PRAGMA wal_autocheckpoint = 1000;
            ",
        )?;

        DatabaseSchema::create_tables(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Checkpoint WAL to ensure all data is flushed to the main database file.
    pub fn checkpoint(&self) -> Result<()> {
        self.with_connection(|conn| {
            conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()))
        })
    }

    pub fn connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }

    pub fn with_connection<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let conn = self.conn.lock().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(format!("Mutex lock error: {}", e)),
            )
        })?;
        f(&conn)
    }

    pub fn with_connection_mut<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut Connection) -> Result<T>,
    {
        let mut conn = self.conn.lock().map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(format!("Mutex lock error: {}", e)),
            )
        })?;
        f(&mut conn)
    }
}

impl Clone for DbConnection {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
        }
    }
}

impl Drop for DbConnection {
    fn drop(&mut self) {
        // Checkpoint WAL before the last reference is dropped.
        // Arc::strong_count == 1 means this is the last clone.
        if Arc::strong_count(&self.conn) == 1 {
            if let Ok(conn) = self.conn.lock() {
                let _ = conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));
            }
        }
    }
}
