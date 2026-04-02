mod schema;
mod session_dao;
mod message_dao;
mod config_dao;
mod connection;

pub use schema::*;
pub use session_dao::SessionDao;
pub use message_dao::MessageDao;
pub use config_dao::ConfigDao;
pub use connection::DbConnection;

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::Arc;

static DB_CONNECTION: Lazy<RwLock<Option<Arc<DbConnection>>>> = Lazy::new(|| RwLock::new(None));

pub fn init_db(db_path: &str) -> Result<(), String> {
    let conn = DbConnection::new(db_path).map_err(|e| e.to_string())?;
    let mut connection = DB_CONNECTION.write();
    *connection = Some(Arc::new(conn));
    Ok(())
}

pub fn get_db() -> Option<Arc<DbConnection>> {
    DB_CONNECTION.read().clone()
}

pub fn get_session_dao() -> Result<SessionDao, String> {
    get_db()
        .map(|db| SessionDao::new((*db).clone()))
        .ok_or_else(|| "Database not initialized".to_string())
}

pub fn get_message_dao() -> Result<MessageDao, String> {
    get_db()
        .map(|db| MessageDao::new((*db).clone()))
        .ok_or_else(|| "Database not initialized".to_string())
}

pub fn get_config_dao() -> Result<ConfigDao, String> {
    get_db()
        .map(|db| ConfigDao::new((*db).clone()))
        .ok_or_else(|| "Database not initialized".to_string())
}
