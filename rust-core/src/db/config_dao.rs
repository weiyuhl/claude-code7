use super::connection::DbConnection;
use super::schema::ConfigEntity;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Result};

pub struct ConfigDao {
    conn: DbConnection,
}

impl ConfigDao {
    pub fn new(conn: DbConnection) -> Self {
        Self { conn }
    }

    pub fn set(
        &self,
        key: &str,
        value: &str,
        scope: &str,
        project_dir: Option<&str>,
    ) -> Result<()> {
        self.conn.with_connection_mut(|conn| {
            // Use COALESCE to normalize NULL to empty string for consistent PK matching.
            // SQLite treats NULL != NULL in UNIQUE/PK constraints, so INSERT OR REPLACE
            // would create duplicate rows instead of replacing.
            let dir = project_dir.unwrap_or("");
            conn.execute(
                "INSERT OR REPLACE INTO config (key, value, scope, project_dir, updated_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![key, value, scope, dir, Utc::now().to_rfc3339()],
            )?;
            Ok(())
        })
    }

    pub fn get(&self, key: &str, scope: &str, project_dir: Option<&str>) -> Result<Option<String>> {
        self.conn.with_connection(|conn| {
            let dir = project_dir.unwrap_or("");
            let mut stmt = conn.prepare(
                "SELECT value FROM config 
                 WHERE key = ?1 AND scope = ?2 AND project_dir = ?3",
            )?;

            let value = stmt
                .query_row(params![key, scope, dir], |row| row.get(0))
                .optional()?;

            Ok(value)
        })
    }

    pub fn get_all(&self, scope: &str, project_dir: Option<&str>) -> Result<Vec<ConfigEntity>> {
        self.conn.with_connection(|conn| {
            let dir = project_dir.unwrap_or("");
            let mut stmt = conn.prepare(
                "SELECT key, value, scope, project_dir, updated_at 
                 FROM config 
                 WHERE scope = ?1 AND project_dir = ?2",
            )?;

            let configs = stmt.query_map(params![scope, dir], |row| {
                Ok(ConfigEntity {
                    key: row.get(0)?,
                    value: row.get(1)?,
                    scope: row.get(2)?,
                    project_dir: {
                        let raw: String = row.get(3)?;
                        if raw.is_empty() {
                            None
                        } else {
                            Some(raw)
                        }
                    },
                    updated_at: row
                        .get::<_, String>(4)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?;

            Ok(configs.filter_map(|r| r.ok()).collect::<Vec<_>>())
        })
    }

    pub fn delete(&self, key: &str, scope: &str, project_dir: Option<&str>) -> Result<()> {
        self.conn.with_connection_mut(|conn| {
            let dir = project_dir.unwrap_or("");
            conn.execute(
                "DELETE FROM config WHERE key = ?1 AND scope = ?2 AND project_dir = ?3",
                params![key, scope, dir],
            )?;
            Ok(())
        })
    }

    pub fn delete_all(&self) -> Result<()> {
        self.conn.with_connection_mut(|conn| {
            conn.execute("DELETE FROM config", params![])?;
            Ok(())
        })
    }

    pub fn delete_scope(&self, scope: &str) -> Result<()> {
        self.conn.with_connection_mut(|conn| {
            conn.execute("DELETE FROM config WHERE scope = ?1", params![scope])?;
            Ok(())
        })
    }

    pub fn set_api_key(&self, provider: &str, key: &str) -> Result<()> {
        self.set(&format!("api_key.{}", provider), key, "global", None)
    }

    pub fn get_api_key(&self, provider: &str) -> Result<Option<String>> {
        self.get_global(&format!("api_key.{}", provider))
    }

    pub fn set_global_config(&self, key: &str, value: &str) -> Result<()> {
        self.set(key, value, "global", None)
    }

    pub fn get_global_config(&self, key: &str) -> Result<Option<String>> {
        self.get_global(key)
    }

    pub fn set_project_config(&self, key: &str, value: &str, project_dir: &str) -> Result<()> {
        self.set(key, value, "project", Some(project_dir))
    }

    pub fn get_project_config(&self, key: &str, project_dir: &str) -> Result<Option<String>> {
        self.get_project(key, project_dir)
    }

    fn get_global(&self, key: &str) -> Result<Option<String>> {
        self.get(key, "global", None)
    }

    fn get_project(&self, key: &str, project_dir: &str) -> Result<Option<String>> {
        self.get(key, "project", Some(project_dir))
    }
}
