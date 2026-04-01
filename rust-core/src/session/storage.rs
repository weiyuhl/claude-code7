use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::message::Message;
use crate::session::{Session, SessionConfig, ClaudeError};

pub struct SessionStorage {
    storage_dir: PathBuf,
}

impl SessionStorage {
    pub fn new(storage_dir: &str) -> Self {
        let path = PathBuf::from(storage_dir);
        if !path.exists() {
            fs::create_dir_all(&path).unwrap_or_else(|_| {
                eprintln!("Failed to create storage directory: {}", storage_dir);
            });
        }
        Self {
            storage_dir: path,
        }
    }

    pub fn save_session(&self, session: &Session) -> Result<(), ClaudeError> {
        let file_path = self.storage_dir.join(format!("{}.json", session.id));
        
        let messages = session.messages.read();
        let session_data = serde_json::json!({
            "id": session.id,
            "config": session.config,
            "messages": *messages,
        });
        
        let json_str = serde_json::to_string_pretty(&session_data)
            .map_err(|e| ClaudeError::IoError {
                path: file_path.to_string_lossy().to_string(),
                message: e.to_string(),
            })?;
        
        fs::write(&file_path, json_str)
            .map_err(|e| ClaudeError::IoError {
                path: file_path.to_string_lossy().to_string(),
                message: e.to_string(),
            })?;
        
        Ok(())
    }

    pub fn load_session(&self, session_id: &str) -> Result<Option<Session>, ClaudeError> {
        let file_path = self.storage_dir.join(format!("{}.json", session_id));
        
        if !file_path.exists() {
            return Ok(None);
        }
        
        let json_str = fs::read_to_string(&file_path)
            .map_err(|e| ClaudeError::IoError {
                path: file_path.to_string_lossy().to_string(),
                message: e.to_string(),
            })?;
        
        let session_data: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| ClaudeError::IoError {
                path: file_path.to_string_lossy().to_string(),
                message: e.to_string(),
            })?;
        
        let config: SessionConfig = serde_json::from_value(
            session_data.get("config").cloned().unwrap_or_default()
        ).map_err(|e| ClaudeError::IoError {
            path: file_path.to_string_lossy().to_string(),
            message: e.to_string(),
        })?;
        
        let messages: Vec<Message> = serde_json::from_value(
            session_data.get("messages").cloned().unwrap_or_default()
        ).map_err(|e| ClaudeError::IoError {
            path: file_path.to_string_lossy().to_string(),
            message: e.to_string(),
        })?;
        
        Ok(Some(Session {
            id: session_id.to_string(),
            messages: RwLock::new(messages),
            config,
            provider: RwLock::new(None),
        }))
    }

    pub fn delete_session(&self, session_id: &str) -> Result<(), ClaudeError> {
        let file_path = self.storage_dir.join(format!("{}.json", session_id));
        
        if file_path.exists() {
            fs::remove_file(&file_path)
                .map_err(|e| ClaudeError::IoError {
                    path: file_path.to_string_lossy().to_string(),
                    message: e.to_string(),
                })?;
        }
        
        Ok(())
    }

    pub fn list_sessions(&self) -> Result<Vec<String>, ClaudeError> {
        let mut sessions = Vec::new();
        
        if let Ok(entries) = fs::read_dir(&self.storage_dir) {
            for entry in entries.flatten() {
                if let Some(extension) = entry.path().extension() {
                    if extension == "json" {
                        if let Some(stem) = entry.path().file_stem() {
                            if let Some(name) = stem.to_str() {
                                sessions.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        Ok(sessions)
    }

    pub fn clear_all(&self) -> Result<(), ClaudeError> {
        if self.storage_dir.exists() {
            fs::remove_dir_all(&self.storage_dir)
                .map_err(|e| ClaudeError::IoError {
                    path: self.storage_dir.to_string_lossy().to_string(),
                    message: e.to_string(),
                })?;
            fs::create_dir_all(&self.storage_dir)
                .map_err(|e| ClaudeError::IoError {
                    path: self.storage_dir.to_string_lossy().to_string(),
                    message: e.to_string(),
                })?;
        }
        Ok(())
    }
}

pub struct PersistentSessionManager {
    sessions: RwLock<Vec<Arc<Session>>>,
    storage: SessionStorage,
}

impl PersistentSessionManager {
    pub fn new(storage_dir: &str) -> Self {
        let storage = SessionStorage::new(storage_dir);
        Self {
            sessions: RwLock::new(Vec::new()),
            storage,
        }
    }

    pub fn create_session(&self, config: SessionConfig) -> Result<Arc<Session>, ClaudeError> {
        let id = uuid::Uuid::new_v4().to_string();
        let session = Arc::new(Session {
            id: id.clone(),
            messages: RwLock::new(Vec::new()),
            config,
            provider: RwLock::new(None),
        });

        self.storage.save_session(&session)?;

        let mut sessions = self.sessions.write();
        sessions.push(Arc::clone(&session));

        Ok(session)
    }

    pub fn load_session(&self, session_id: &str) -> Result<Option<Arc<Session>>, ClaudeError> {
        {
            let sessions = self.sessions.read();
            if let Some(session) = sessions.iter().find(|s| s.id == session_id) {
                return Ok(Some(Arc::clone(session)));
            }
        }

        if let Some(session) = self.storage.load_session(session_id)? {
            let arc_session = Arc::new(session);
            let mut sessions = self.sessions.write();
            sessions.push(Arc::clone(&arc_session));
            return Ok(Some(arc_session));
        }

        Ok(None)
    }

    pub fn save_session(&self, session: &Arc<Session>) -> Result<(), ClaudeError> {
        self.storage.save_session(session)
    }

    pub fn remove_session(&self, session_id: &str) -> Result<bool, ClaudeError> {
        {
            let mut sessions = self.sessions.write();
            if let Some(pos) = sessions.iter().position(|s| s.id == session_id) {
                sessions.remove(pos);
            }
        }
        
        self.storage.delete_session(session_id)?;
        Ok(true)
    }

    pub fn list_sessions(&self) -> Result<Vec<String>, ClaudeError> {
        self.storage.list_sessions()
    }

    pub fn clear_all(&self) -> Result<(), ClaudeError> {
        let mut sessions = self.sessions.write();
        sessions.clear();
        self.storage.clear_all()
    }
}

impl Default for PersistentSessionManager {
    fn default() -> Self {
        Self::new("./sessions")
    }
}
