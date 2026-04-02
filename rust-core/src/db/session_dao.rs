use rusqlite::{Connection, Result, params, OptionalExtension};
use chrono::Utc;
use super::schema::{SessionEntity, MessageEntity};
use super::connection::DbConnection;

pub struct SessionDao {
    conn: DbConnection,
}

impl SessionDao {
    pub fn new(conn: DbConnection) -> Self {
        Self { conn }
    }

    pub fn insert(&self, session: &SessionEntity) -> Result<()> {
        self.conn.with_connection_mut(|conn| {
            conn.execute(
                "INSERT INTO sessions (id, title, created_at, updated_at, project_dir, branch, is_active, metadata) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    session.id,
                    session.title,
                    session.created_at.to_rfc3339(),
                    session.updated_at.to_rfc3339(),
                    session.project_dir,
                    session.branch,
                    if session.is_active { 1 } else { 0 },
                    session.metadata,
                ],
            )?;
            Ok(())
        })
    }

    pub fn get_by_id(&self, session_id: &str) -> Result<Option<SessionEntity>> {
        self.conn.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, title, created_at, updated_at, project_dir, branch, is_active, metadata 
                 FROM sessions WHERE id = ?1"
            )?;
            
            let session = stmt.query_row(params![session_id], |row| {
                Ok(SessionEntity {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get::<_, String>(2)?.parse().unwrap_or_else(|_| Utc::now()),
                    updated_at: row.get::<_, String>(3)?.parse().unwrap_or_else(|_| Utc::now()),
                    project_dir: row.get(4)?,
                    branch: row.get(5)?,
                    is_active: row.get::<_, i32>(6)? == 1,
                    metadata: row.get(7)?,
                })
            }).optional()?;
            
            Ok(session)
        })
    }

    pub fn get_all(&self) -> Result<Vec<SessionEntity>> {
        self.conn.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, title, created_at, updated_at, project_dir, branch, is_active, metadata 
                 FROM sessions ORDER BY updated_at DESC"
            )?;
            
            let sessions = stmt.query_map(params![], |row| {
                Ok(SessionEntity {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get::<_, String>(2)?.parse().unwrap_or_else(|_| Utc::now()),
                    updated_at: row.get::<_, String>(3)?.parse().unwrap_or_else(|_| Utc::now()),
                    project_dir: row.get(4)?,
                    branch: row.get(5)?,
                    is_active: row.get::<_, i32>(6)? == 1,
                    metadata: row.get(7)?,
                })
            })?;
            
            Ok(sessions.filter_map(|r| r.ok()).collect::<Vec<_>>())
        })
    }

    pub fn get_active(&self) -> Result<Option<SessionEntity>> {
        self.conn.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, title, created_at, updated_at, project_dir, branch, is_active, metadata 
                 FROM sessions WHERE is_active = 1 LIMIT 1"
            )?;
            
            let session = stmt.query_row(params![], |row| {
                Ok(SessionEntity {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get::<_, String>(2)?.parse().unwrap_or_else(|_| Utc::now()),
                    updated_at: row.get::<_, String>(3)?.parse().unwrap_or_else(|_| Utc::now()),
                    project_dir: row.get(4)?,
                    branch: row.get(5)?,
                    is_active: row.get::<_, i32>(6)? == 1,
                    metadata: row.get(7)?,
                })
            }).optional()?;
            
            Ok(session)
        })
    }

    pub fn update_title(&self, session_id: &str, title: &str) -> Result<()> {
        self.conn.with_connection_mut(|conn| {
            conn.execute(
                "UPDATE sessions SET title = ?1, updated_at = ?2 WHERE id = ?3",
                params![title, Utc::now().to_rfc3339(), session_id],
            )?;
            Ok(())
        })
    }

    pub fn update_metadata(&self, session_id: &str, metadata: &str) -> Result<()> {
        self.conn.with_connection_mut(|conn| {
            conn.execute(
                "UPDATE sessions SET metadata = ?1, updated_at = ?2 WHERE id = ?3",
                params![metadata, Utc::now().to_rfc3339(), session_id],
            )?;
            Ok(())
        })
    }

    pub fn set_active(&self, session_id: &str) -> Result<()> {
        self.conn.with_connection_mut(|conn| {
            let tx = conn.transaction()?;
            tx.execute("UPDATE sessions SET is_active = 0", params![])?;
            tx.execute(
                "UPDATE sessions SET is_active = 1, updated_at = ?1 WHERE id = ?2",
                params![Utc::now().to_rfc3339(), session_id],
            )?;
            tx.commit()?;
            Ok(())
        })
    }

    pub fn delete(&self, session_id: &str) -> Result<()> {
        self.conn.with_connection_mut(|conn| {
            conn.execute(
                "DELETE FROM sessions WHERE id = ?1",
                params![session_id],
            )?;
            Ok(())
        })
    }

    pub fn delete_all(&self) -> Result<()> {
        self.conn.with_connection_mut(|conn| {
            conn.execute("DELETE FROM sessions", params![])?;
            Ok(())
        })
    }
}
