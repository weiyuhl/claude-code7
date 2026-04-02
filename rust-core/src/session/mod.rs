mod error;
mod config;
mod storage;
mod config_manager;

pub use error::*;
pub use config::*;
pub use storage::{SessionStorage, PersistentSessionManager};
pub use config_manager::{ConfigManager, GlobalConfig, ProjectConfig, ConfigError};

use std::ffi::{c_char, c_void};
use std::sync::Arc;
use parking_lot::RwLock;

use crate::message::Message;

pub struct Session {
    pub id: String,
    pub messages: RwLock<Vec<Message>>,
    pub config: SessionConfig,
    pub provider: RwLock<Option<Arc<dyn crate::api::providers::Provider>>>,
}

unsafe impl Send for Session {}
unsafe impl Sync for Session {}

impl Session {
    pub fn add_message(&self, message: Message) -> Result<(), ClaudeError> {
        let mut messages = self.messages.write();
        messages.push(message);
        Ok(())
    }

    pub async fn list_models(&self) -> Result<Vec<crate::api::providers::ModelInfo>, ClaudeError> {
        let provider = self.provider.read();
        let provider = provider.as_ref().ok_or_else(|| ClaudeError::ConfigError {
            message: "No provider configured".to_string(),
        })?;
        provider.list_models().await
    }

    pub async fn get_balance(&self) -> Result<crate::api::providers::BalanceInfo, ClaudeError> {
        let provider = self.provider.read();
        let provider = provider.as_ref().ok_or_else(|| ClaudeError::ConfigError {
            message: "No provider configured".to_string(),
        })?;
        provider.get_balance().await
    }
}

pub struct SessionManager {
    sessions: RwLock<Vec<Arc<Session>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(Vec::new()),
        }
    }

    pub fn create_session(&self, config: SessionConfig) -> Result<Arc<Session>, ClaudeError> {
        let id = uuid::Uuid::new_v4().to_string();
        let session = Arc::new(Session {
            id,
            messages: RwLock::new(Vec::new()),
            config,
            provider: RwLock::new(None),
        });

        let mut sessions = self.sessions.write();
        sessions.push(Arc::clone(&session));

        Ok(session)
    }

    pub fn get_session(&self, id: &str) -> Option<Arc<Session>> {
        let sessions = self.sessions.read();
        sessions.iter().find(|s| s.id == id).map(Arc::clone)
    }

    pub fn remove_session(&self, id: &str) -> bool {
        let mut sessions = self.sessions.write();
        if let Some(pos) = sessions.iter().position(|s| s.id == id) {
            sessions.remove(pos);
            true
        } else {
            false
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

static SESSION_MANAGER: once_cell::sync::Lazy<SessionManager> =
    once_cell::sync::Lazy::new(SessionManager::new);

pub fn get_session_manager() -> &'static SessionManager {
    &SESSION_MANAGER
}

pub type StreamCallback = extern "C" fn(*const c_char, *mut c_void);
