use rusqlite::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEntity {
    pub id: String,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub project_dir: Option<String>,
    pub branch: Option<String>,
    pub is_active: bool,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEntity {
    pub id: String,
    pub session_id: String,
    pub parent_uuid: Option<String>,
    pub role: String,
    pub content: String,
    pub name: Option<String>,
    pub tool_call_id: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input: Option<String>,
    pub thinking: Option<String>,
    pub thinking_signature: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub is_compact_summary: bool,
    pub compact_boundary: Option<String>,
    pub order_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntity {
    pub key: String,
    pub value: String,
    pub scope: String,
    pub project_dir: Option<String>,
    pub updated_at: DateTime<Utc>,
}

pub struct DatabaseSchema;

impl DatabaseSchema {
    pub fn create_tables(conn: &rusqlite::Connection) -> Result<()> {
        conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                project_dir TEXT,
                branch TEXT,
                is_active INTEGER NOT NULL DEFAULT 0,
                metadata TEXT
            );
            
            CREATE INDEX IF NOT EXISTS idx_sessions_active ON sessions(is_active);
            CREATE INDEX IF NOT EXISTS idx_sessions_project ON sessions(project_dir);
            CREATE INDEX IF NOT EXISTS idx_sessions_created ON sessions(created_at);
            
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                parent_uuid TEXT,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                name TEXT,
                tool_call_id TEXT,
                tool_name TEXT,
                tool_input TEXT,
                thinking TEXT,
                thinking_signature TEXT,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                is_compact_summary INTEGER NOT NULL DEFAULT 0,
                compact_boundary TEXT,
                order_index INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );
            
            CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id);
            CREATE INDEX IF NOT EXISTS idx_messages_parent ON messages(parent_uuid);
            CREATE INDEX IF NOT EXISTS idx_messages_order ON messages(session_id, order_index);
            CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp);
            
            CREATE TABLE IF NOT EXISTS config (
                key TEXT NOT NULL,
                scope TEXT NOT NULL DEFAULT 'global',
                project_dir TEXT,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (key, scope, project_dir)
            );
            
            CREATE INDEX IF NOT EXISTS idx_config_scope ON config(scope);
            CREATE INDEX IF NOT EXISTS idx_config_project ON config(project_dir);
            
            CREATE TABLE IF NOT EXISTS api_keys (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                provider TEXT NOT NULL,
                key_encrypted TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_used_at TEXT
            );
            
            CREATE INDEX IF NOT EXISTS idx_api_keys_provider ON api_keys(provider);
            CREATE INDEX IF NOT EXISTS idx_api_keys_active ON api_keys(is_active);
            
            CREATE TABLE IF NOT EXISTS tool_results (
                id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                tool_name TEXT NOT NULL,
                result TEXT NOT NULL,
                error TEXT,
                duration_ms INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE
            );
            
            CREATE INDEX IF NOT EXISTS idx_tool_results_message ON tool_results(message_id);
            "
        )?;
        Ok(())
    }
}
